// This whole file is inspired by: https://github.com/jeaye/q3/blob/master/src/client/ui/ttf/renderer.rs
// available under the BSD-3 licence.
// It has been modified to work with gl-rs, nalgebra, and rust-freetype

use na::{Point2, Point3, Vector2};
use rusttype;
use rusttype::gpu_cache::{Cache, CacheBuilder};
use std::rc::Rc;

use context::{Context, Texture};
use resource::{AllocationType, BufferType, Effect, GPUVec, ShaderAttribute, ShaderUniform};
use text::Font;

#[path = "../error.rs"]
mod error;

struct TextRenderContext {
    len: usize,
    scale: f32,
    color: Point3<f32>,
    pos: Point2<f32>,
    font: Rc<Font>,
}

/// A ttf text renderer.
pub struct TextRenderer {
    text: String,
    texture: Texture,
    shader: Effect,
    cache: Cache<'static>,
    invsz: ShaderUniform<Vector2<f32>>,
    tex: ShaderUniform<i32>,
    color: ShaderUniform<Point3<f32>>,
    pos: ShaderAttribute<Point2<f32>>,
    uvs: ShaderAttribute<Point2<f32>>,
    contexts: Vec<TextRenderContext>,
    coords: GPUVec<Point2<f32>>,
}

impl TextRenderer {
    /// Creates a new text renderer with `font` as the default font.
    pub fn new() -> TextRenderer {
        //
        // Create cache.
        //
        let atlas_width = 1024;
        let atlas_height = 1024;
        let cache = CacheBuilder {
            width: atlas_width,
            height: atlas_height,
            ..CacheBuilder::default()
        }.build();

        //
        // Create texture.
        //
        let ctxt = Context::get();

        /* We're using 1 byte alignment buffering. */
        verify!(ctxt.pixel_storei(Context::UNPACK_ALIGNMENT, 1));

        let texture = verify!(
            ctxt.create_texture()
                .expect("Font texture creation failed.")
        );
        verify!(ctxt.bind_texture(Context::TEXTURE_2D, Some(&texture)));
        verify!(ctxt.tex_image2d(
            Context::TEXTURE_2D,
            0,
            Context::RED as i32,
            atlas_width as i32,
            atlas_height as i32,
            0,
            Context::RED,
            None
        ));

        /* Clamp to the edge to avoid artifacts when scaling. */
        verify!(ctxt.tex_parameteri(
            Context::TEXTURE_2D,
            Context::TEXTURE_WRAP_S,
            Context::CLAMP_TO_EDGE as i32
        ));
        verify!(ctxt.tex_parameteri(
            Context::TEXTURE_2D,
            Context::TEXTURE_WRAP_T,
            Context::CLAMP_TO_EDGE as i32
        ));

        /* Linear filtering usually looks best for text. */
        verify!(ctxt.tex_parameteri(
            Context::TEXTURE_2D,
            Context::TEXTURE_MIN_FILTER,
            Context::LINEAR as i32
        ));
        verify!(ctxt.tex_parameteri(
            Context::TEXTURE_2D,
            Context::TEXTURE_MAG_FILTER,
            Context::LINEAR as i32
        ));

        //
        // Create shader.
        //
        let mut shader = Effect::new_from_str(TEXT_VERTEX_SRC, TEXT_FRAGMENT_SRC);
        shader.use_program();

        TextRenderer {
            text: String::new(),
            cache,
            texture,
            invsz: shader.get_uniform("invsz").expect("Could not find invsz"),
            tex: shader.get_uniform("tex0").expect("Could not find tex0"),
            color: shader.get_uniform("color").expect("Could not find color"),
            pos: shader.get_attrib("pos").expect("Could not find pos"),
            uvs: shader.get_attrib("uvs").expect("Could not find uvs"),
            shader: shader,
            contexts: Vec::new(),
            coords: GPUVec::new(Vec::new(), BufferType::Array, AllocationType::StreamDraw),
        }
    }

    /// Adds a piece of text to be drawn during the next frame. The text is not persistent between
    /// frames. This method must be called for each text to draw, and at each update loop
    /// iteration.
    pub fn draw_text(
        &mut self,
        text: &str,
        pos: &Point2<f32>,
        scale: f32,
        font: &Rc<Font>,
        color: &Point3<f32>,
    ) {
        self.text.push_str(text);
        self.contexts.push(TextRenderContext {
            len: text.len(),
            scale,
            color: *color,
            pos: *pos,
            font: font.clone(),
        })
    }

    /// Actually draws the text.
    pub fn render(&mut self, width: f32, height: f32) {
        if self.contexts.len() == 0 {
            return;
        }

        let ctxt = Context::get();
        self.shader.use_program();

        let _ = verify!(ctxt.polygon_mode(Context::FRONT_AND_BACK, Context::FILL));
        verify!(ctxt.enable(Context::BLEND));
        verify!(ctxt.blend_func(Context::SRC_ALPHA, Context::ONE_MINUS_SRC_ALPHA));
        verify!(ctxt.disable(Context::DEPTH_TEST));

        self.pos.enable();
        self.uvs.enable();
        self.tex.upload(&0);
        self.invsz.upload(&Vector2::new(1.0 / width, -1.0 / height));

        verify!(ctxt.bind_texture(Context::TEXTURE_2D, Some(&self.texture)));
        verify!(ctxt.tex_parameteri(
            Context::TEXTURE_2D,
            Context::TEXTURE_WRAP_S,
            Context::CLAMP_TO_EDGE as i32
        ));
        verify!(ctxt.tex_parameteri(
            Context::TEXTURE_2D,
            Context::TEXTURE_WRAP_T,
            Context::CLAMP_TO_EDGE as i32
        ));

        let mut pos = 0;

        for context in self.contexts.iter() {
            let scale = rusttype::Scale::uniform(context.scale);
            let vmetrics = context.font.font().v_metrics(scale);
            let line_height = vmetrics.ascent - vmetrics.descent;
            let text = &self.text[pos..pos + context.len];
            let font_uid = Font::uid(&context.font);
            let mut vshift = 0.0;

            for line in text.lines() {
                let orig = rusttype::Point {
                    x: context.pos.x,
                    y: context.pos.y + vshift,
                };

                vshift += line_height as f32;
                let layout = context.font.font().layout(line, scale, orig);

                for glyph in layout {
                    let gly: rusttype::PositionedGlyph<'static> = context
                        .font
                        .font()
                        .glyph(glyph.id())
                        .scaled(glyph.scale())
                        .positioned(glyph.position());
                    self.cache.queue_glyph(font_uid, gly); // FIXME: is the call to `.standalone()` costly?
                }

                let _ = self.cache.cache_queued(|rect, data| {
                    verify!(ctxt.tex_sub_image2d(
                        Context::TEXTURE_2D,
                        0,
                        rect.min.x as i32,
                        rect.min.y as i32,
                        rect.width() as i32,
                        rect.height() as i32,
                        Context::RED,
                        Some(&data)
                    ));
                });

                let layout = context.font.font().layout(line, scale, orig);

                {
                    let coords = self.coords.data_mut().as_mut().unwrap();
                    for glyph in layout {
                        if let Some(Some((tex, rect))) = self.cache.rect_for(font_uid, &glyph).ok()
                        {
                            let min_px = rect.min.x as f32;
                            let min_py = rect.min.y as f32 + vmetrics.ascent;
                            let max_px = rect.max.x as f32;
                            let max_py = rect.max.y as f32 + vmetrics.ascent;

                            coords.push(Point2::new(min_px, min_py));
                            coords.push(Point2::new(tex.min.x, tex.min.y));

                            coords.push(Point2::new(min_px, max_py));
                            coords.push(Point2::new(tex.min.x, tex.max.y));

                            coords.push(Point2::new(max_px, min_py));
                            coords.push(Point2::new(tex.max.x, tex.min.y));

                            coords.push(Point2::new(max_px, min_py));
                            coords.push(Point2::new(tex.max.x, tex.min.y));

                            coords.push(Point2::new(min_px, max_py));
                            coords.push(Point2::new(tex.min.x, tex.max.y));

                            coords.push(Point2::new(max_px, max_py));
                            coords.push(Point2::new(tex.max.x, tex.max.y));
                        }
                    }
                }

                self.pos.bind_sub_buffer(&mut self.coords, 1, 0);
                self.uvs.bind_sub_buffer(&mut self.coords, 1, 1);
                self.color.upload(&context.color);

                verify!(ctxt.draw_arrays(Context::TRIANGLES, 0, (self.coords.len() / 2) as i32));
            }
            pos += context.len;

            self.coords.data_mut().as_mut().unwrap().clear();
        }

        self.pos.disable();
        self.uvs.enable();

        verify!(ctxt.enable(Context::DEPTH_TEST));
        verify!(ctxt.disable(Context::BLEND));

        self.contexts.clear();
        self.text.clear();
    }
}

/// Vertex shader used by the material to display line.
pub static TEXT_VERTEX_SRC: &'static str = A_VERY_LONG_STRING;
/// Fragment shader used by the material to display line.
pub static TEXT_FRAGMENT_SRC: &'static str = ANOTHER_VERY_LONG_STRING;

const A_VERY_LONG_STRING: &'static str = "
#version 100

uniform vec2 invsz;
uniform vec3 color;

attribute vec2 pos;
attribute vec2 uvs;

varying vec2 tex;
varying vec3 Color;

void main() {
    gl_Position = vec4(pos.x * invsz.x - 1.0, pos.y * invsz.y + 1.0, -1.0, 1.0);
    tex         = uvs;
    Color       = color;
}
";

const ANOTHER_VERY_LONG_STRING: &'static str = "
#version 100

#ifdef GL_FRAGMENT_PRECISION_HIGH
   precision highp float;
#else
   precision mediump float;
#endif

uniform sampler2D tex0;

varying vec2 tex;
varying vec3 Color;

void main() {
    gl_FragColor = vec4(Color, texture2D(tex0, tex).r);
}
";
