//! A renderer for Conrod primitives.

use conrod::position::Rect;
use conrod::text::GlyphCache;
use conrod::{render::PrimitiveKind, Ui};
use conrod_core as conrod;
use kiss3d::context::{Context, Texture};
use kiss3d::resource::{
    AllocationType, BufferType, Effect, GPUVec, ShaderAttribute, ShaderUniform,
};
use kiss3d::text::Font;
use nalgebra::{Point2, Point3, Point4, Vector2};
use rusttype::gpu_cache::Cache;
use std::rc::Rc;

macro_rules! verify(
    ($e: expr) => {
        {
            let res = $e;
            #[cfg(not(any(target_arch = "wasm32", target_arch = "asmjs")))]
            { assert_eq!(kiss3d::context::Context::get().get_error(), 0); }
            res
        }
    }
);

#[derive(Copy, Clone, Debug, PartialEq)]
enum RenderMode {
    Image {
        color: Point4<f32>,
        texture: conrod::image::Id,
    },
    Shape,
    Text {
        color: Point4<f32>,
    },
    Unknown,
}

/// Structure which manages the display of short-living points.
pub struct ConrodRenderer {
    ui: Ui,
    triangle_shader: Effect,
    triangle_window_size: ShaderUniform<Vector2<f32>>,
    triangle_pos: ShaderAttribute<Point2<f32>>,
    triangle_color: ShaderAttribute<Point4<f32>>,
    text_shader: Effect,
    text_window_size: ShaderUniform<Vector2<f32>>,
    text_color: ShaderUniform<Point4<f32>>,
    text_pos: ShaderAttribute<Point2<f32>>,
    text_uvs: ShaderAttribute<Point2<f32>>,
    text_texture: ShaderUniform<i32>,
    image_shader: Effect,
    image_window_size: ShaderUniform<Vector2<f32>>,
    image_color: ShaderUniform<Point4<f32>>,
    image_pos: ShaderAttribute<Point2<f32>>,
    image_uvs: ShaderAttribute<Point2<f32>>,
    image_texture: ShaderUniform<i32>,
    points: GPUVec<f32>,
    indices: GPUVec<Point3<u16>>,
    cache: GlyphCache<'static>,
    texture: Texture,
    resized_once: bool,
}

impl ConrodRenderer {
    /// Creates a new points manager.
    pub fn new(width: f64, height: f64) -> ConrodRenderer {
        //
        // Create shaders.
        //
        let triangle_shader = Effect::new_from_str(TRIANGLES_VERTEX_SRC, TRIANGLES_FRAGMENT_SRC);
        let text_shader = Effect::new_from_str(TEXT_VERTEX_SRC, TEXT_FRAGMENT_SRC);
        let image_shader = Effect::new_from_str(IMAGE_VERTEX_SRC, IMAGE_FRAGMENT_SRC);

        //
        // Initialize UI with the default font.
        //
        let mut ui = conrod::UiBuilder::new([width, height]).build();
        let _ = ui.fonts.insert(Font::default().font().clone());

        //
        // Create cache or text.
        //
        let atlas_width = 1024;
        let atlas_height = 1024;
        let cache = Cache::builder()
            .dimensions(atlas_width, atlas_height)
            .build();

        //
        // Create texture for text
        //
        let ctxt = Context::get();

        /* We're using 1 byte alignment buffering. */
        verify!(ctxt.pixel_storei(Context::UNPACK_ALIGNMENT, 1));

        let texture = verify!(ctxt
            .create_texture()
            .expect("Font texture creation failed."));
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

        ConrodRenderer {
            ui,
            points: GPUVec::new(Vec::new(), BufferType::Array, AllocationType::StreamDraw),
            indices: GPUVec::new(
                Vec::new(),
                BufferType::ElementArray,
                AllocationType::StreamDraw,
            ),
            triangle_window_size: triangle_shader.get_uniform("window_size").unwrap(),
            triangle_pos: triangle_shader.get_attrib("position").unwrap(),
            triangle_color: triangle_shader.get_attrib("color").unwrap(),
            triangle_shader,
            text_window_size: text_shader
                .get_uniform::<Vector2<f32>>("window_size")
                .unwrap(),
            text_color: text_shader.get_uniform::<Point4<f32>>("color").unwrap(),
            text_pos: text_shader.get_attrib("pos").unwrap(),
            text_uvs: text_shader.get_attrib("uvs").unwrap(),
            text_texture: text_shader.get_uniform("tex0").unwrap(),
            text_shader,
            image_window_size: image_shader
                .get_uniform::<Vector2<f32>>("window_size")
                .unwrap(),
            image_color: image_shader.get_uniform::<Point4<f32>>("color").unwrap(),
            image_pos: image_shader.get_attrib("pos").unwrap(),
            image_uvs: image_shader.get_attrib("uvs").unwrap(),
            image_texture: image_shader.get_uniform("tex0").unwrap(),
            image_shader,
            cache,
            texture,
            resized_once: false,
        }
    }

    /// The mutable UI to be displayed.
    pub fn ui_mut(&mut self) -> &mut Ui {
        &mut self.ui
    }

    /// The UI to be displayed.
    pub fn ui(&self) -> &Ui {
        &self.ui
    }

    /// Actually draws the points.
    pub fn render(
        &mut self,
        width: f32,
        height: f32,
        hidpi_factor: f32,
        texture_map: &conrod::image::Map<(Rc<Texture>, (u32, u32))>,
    ) {
        // NOTE: this seems necessary for WASM.
        if !self.resized_once {
            self.ui.handle_event(conrod::event::Input::Resize(
                width as f64 / hidpi_factor as f64,
                height as f64 / hidpi_factor as f64,
            ));
            self.resized_once = true;
        }

        let mut primitives = self.ui.draw();
        let ctxt = Context::get();
        let mut mode = RenderMode::Unknown;
        let mut curr_scizzor = Rect::from_corners([0.0, 0.0], [0.0, 0.0]);

        let mut vid = 0;

        verify!(ctxt.disable(Context::CULL_FACE));
        let _ = verify!(ctxt.polygon_mode(Context::FRONT_AND_BACK, Context::FILL));
        verify!(ctxt.enable(Context::BLEND));
        verify!(ctxt.blend_func_separate(
            Context::SRC_ALPHA,
            Context::ONE_MINUS_SRC_ALPHA,
            Context::ONE,
            Context::ONE_MINUS_SRC_ALPHA,
        ));
        verify!(ctxt.disable(Context::DEPTH_TEST));
        verify!(ctxt.enable(Context::SCISSOR_TEST));

        let rect_to_gl_rect = |rect: Rect| {
            let (w, h) = rect.w_h();
            let l = rect.left() as f32 * hidpi_factor + width / 2.0;
            let b = rect.bottom() as f32 * hidpi_factor + height / 2.0;
            let w = w as f32 * hidpi_factor;
            let h = h as f32 * hidpi_factor;

            (l.max(0.0), b.max(0.0), w.min(width), h.min(height))
        };

        loop {
            let primitive = primitives.next();

            let render = if let Some(ref primitive) = primitive {
                curr_scizzor != primitive.scizzor
                    || match primitive.kind {
                        PrimitiveKind::TrianglesSingleColor { .. } => mode != RenderMode::Shape,
                        PrimitiveKind::TrianglesMultiColor { .. } => mode != RenderMode::Shape,
                        PrimitiveKind::Rectangle { .. } => mode != RenderMode::Shape,
                        PrimitiveKind::Image {
                            color, image_id, ..
                        } => {
                            let rgba = color.unwrap_or(conrod::color::WHITE).to_rgb();
                            mode != RenderMode::Image {
                                color: Point4::new(rgba.0, rgba.1, rgba.2, rgba.3),
                                texture: image_id,
                            }
                        }
                        PrimitiveKind::Text { color, .. } => {
                            let rgba = color.to_rgb();
                            mode != RenderMode::Text {
                                color: Point4::new(rgba.0, rgba.1, rgba.2, rgba.3),
                            }
                        }
                        PrimitiveKind::Other(_) => false,
                    }
            } else {
                true
            };

            if render {
                let (x, y, w, h) = rect_to_gl_rect(curr_scizzor);
                ctxt.scissor(x as i32, y as i32, w as i32, h as i32);
                match mode {
                    RenderMode::Shape => {
                        self.triangle_shader.use_program();
                        self.triangle_pos.enable();
                        self.triangle_color.enable();

                        self.triangle_window_size
                            .upload(&Vector2::new(width, height));
                        unsafe {
                            self.triangle_color
                                .bind_sub_buffer_generic(&mut self.points, 5, 2)
                        };
                        unsafe {
                            self.triangle_pos
                                .bind_sub_buffer_generic(&mut self.points, 5, 0)
                        };
                        self.indices.bind();

                        verify!(ctxt.draw_elements(
                            Context::TRIANGLES,
                            self.indices.len() as i32 * 3,
                            Context::UNSIGNED_SHORT,
                            0
                        ));

                        self.triangle_pos.disable();
                        self.triangle_color.disable();
                    }
                    RenderMode::Text { color } => {
                        self.text_shader.use_program();
                        self.text_pos.enable();
                        self.text_uvs.enable();
                        self.text_texture.upload(&0);
                        self.text_color.upload(&color);
                        self.text_window_size.upload(&Vector2::new(width, height));
                        verify!(ctxt.bind_texture(Context::TEXTURE_2D, Some(&self.texture)));
                        unsafe {
                            self.text_pos
                                .bind_sub_buffer_generic(&mut self.points, 3, 0)
                        };
                        unsafe {
                            self.text_uvs
                                .bind_sub_buffer_generic(&mut self.points, 3, 2)
                        };

                        verify!(ctxt.draw_arrays(
                            Context::TRIANGLES,
                            0,
                            (self.points.len() / 4) as i32
                        ));

                        self.text_pos.disable();
                        self.text_uvs.disable();
                    }
                    RenderMode::Image { color, texture } => {
                        if let Some(texture) = texture_map.get(&texture) {
                            self.image_shader.use_program();
                            self.image_pos.enable();
                            self.image_uvs.enable();

                            self.image_texture.upload(&0);
                            self.image_color.upload(&color);
                            self.image_window_size.upload(&Vector2::new(width, height));
                            verify!(ctxt.bind_texture(Context::TEXTURE_2D, Some(&texture.0)));
                            unsafe {
                                self.image_pos
                                    .bind_sub_buffer_generic(&mut self.points, 3, 0)
                            };
                            unsafe {
                                self.image_uvs
                                    .bind_sub_buffer_generic(&mut self.points, 3, 2)
                            };

                            verify!(ctxt.draw_arrays(
                                Context::TRIANGLES,
                                0,
                                (self.points.len() / 4) as i32
                            ));

                            self.image_pos.disable();
                            self.image_uvs.disable();
                        }
                    }
                    RenderMode::Unknown => {}
                }

                vid = 0;
                mode = RenderMode::Unknown;
                self.points.data_mut().as_mut().unwrap().clear();
                self.indices.data_mut().as_mut().unwrap().clear();
            }

            if primitive.is_none() {
                break;
            }

            let primitive = primitive.unwrap();
            let vertices = self.points.data_mut().as_mut().unwrap();
            let indices = self.indices.data_mut().as_mut().unwrap();
            curr_scizzor = primitive.scizzor;

            match primitive.kind {
                PrimitiveKind::Rectangle { color } => {
                    mode = RenderMode::Shape;

                    let color = color.to_rgb();
                    let tl = (primitive.rect.x.start, primitive.rect.y.end);
                    let bl = (primitive.rect.x.start, primitive.rect.y.start);
                    let br = (primitive.rect.x.end, primitive.rect.y.start);
                    let tr = (primitive.rect.x.end, primitive.rect.y.end);

                    vertices.extend_from_slice(&[
                        tl.0 as f32 * hidpi_factor,
                        tl.1 as f32 * hidpi_factor,
                        color.0,
                        color.1,
                        color.2,
                        color.3,
                        bl.0 as f32 * hidpi_factor,
                        bl.1 as f32 * hidpi_factor,
                        color.0,
                        color.1,
                        color.2,
                        color.3,
                        br.0 as f32 * hidpi_factor,
                        br.1 as f32 * hidpi_factor,
                        color.0,
                        color.1,
                        color.2,
                        color.3,
                        tr.0 as f32 * hidpi_factor,
                        tr.1 as f32 * hidpi_factor,
                        color.0,
                        color.1,
                        color.2,
                        color.3,
                    ]);

                    indices.push(Point3::new(vid + 0, vid + 1, vid + 2));
                    indices.push(Point3::new(vid + 2, vid + 3, vid + 0));

                    vid += 4;
                }
                PrimitiveKind::TrianglesSingleColor { color, triangles } => {
                    mode = RenderMode::Shape;

                    for triangle in triangles {
                        let pts = triangle.points();

                        vertices.extend_from_slice(&[
                            pts[0][0] as f32 * hidpi_factor,
                            pts[0][1] as f32 * hidpi_factor,
                            color.0,
                            color.1,
                            color.2,
                            color.3,
                            pts[1][0] as f32 * hidpi_factor,
                            pts[1][1] as f32 * hidpi_factor,
                            color.0,
                            color.1,
                            color.2,
                            color.3,
                            pts[2][0] as f32 * hidpi_factor,
                            pts[2][1] as f32 * hidpi_factor,
                            color.0,
                            color.1,
                            color.2,
                            color.3,
                        ]);
                        indices.push(Point3::new(vid + 0, vid + 1, vid + 2));

                        vid += 3;
                    }
                }
                PrimitiveKind::TrianglesMultiColor { triangles } => {
                    mode = RenderMode::Shape;

                    for triangle in triangles {
                        let ((a, ca), (b, cb), (c, cc)) =
                            (triangle.0[0], triangle.0[1], triangle.0[2]);
                        vertices.extend_from_slice(&[
                            a[0] as f32 * hidpi_factor,
                            a[1] as f32 * hidpi_factor,
                            ca.0,
                            ca.1,
                            ca.2,
                            ca.3,
                            b[0] as f32 * hidpi_factor,
                            b[1] as f32 * hidpi_factor,
                            cb.0,
                            cb.1,
                            cb.2,
                            cb.3,
                            c[0] as f32 * hidpi_factor,
                            c[1] as f32 * hidpi_factor,
                            cc.0,
                            cc.1,
                            cc.2,
                            cc.3,
                        ]);
                        indices.push(Point3::new(vid + 0, vid + 1, vid + 2));

                        vid += 3;
                    }
                }
                PrimitiveKind::Image {
                    image_id,
                    color,
                    source_rect,
                } => {
                    if let Some(texture) = texture_map.get(&image_id) {
                        let color = color.unwrap_or(conrod::color::WHITE).to_rgb();
                        mode = RenderMode::Image {
                            color: Point4::new(color.0, color.1, color.2, color.3),
                            texture: image_id,
                        };

                        let min_px = primitive.rect.x.start as f32 * hidpi_factor;
                        let min_py = primitive.rect.y.start as f32 * hidpi_factor;
                        let max_px = primitive.rect.x.end as f32 * hidpi_factor;
                        let max_py = primitive.rect.y.end as f32 * hidpi_factor;

                        let w = (texture.1).0 as f64;
                        let h = (texture.1).1 as f64;
                        let mut tex = source_rect
                            .unwrap_or(conrod::position::Rect::from_corners([0.0, 0.0], [w, h]));
                        tex.x.start /= w;
                        tex.x.end /= w;
                        // Because opengl textures are loaded upside down.
                        tex.y.start = (h - tex.y.start) / h;
                        tex.y.end = (h - tex.y.end) / h;

                        vertices.extend_from_slice(&[
                            min_px,
                            min_py,
                            tex.x.start as f32,
                            tex.y.start as f32,
                            min_px,
                            max_py,
                            tex.x.start as f32,
                            tex.y.end as f32,
                            max_px,
                            min_py,
                            tex.x.end as f32,
                            tex.y.start as f32,
                            max_px,
                            min_py,
                            tex.x.end as f32,
                            tex.y.start as f32,
                            min_px,
                            max_py,
                            tex.x.start as f32,
                            tex.y.end as f32,
                            max_px,
                            max_py,
                            tex.x.end as f32,
                            tex.y.end as f32,
                        ]);
                    }
                }
                PrimitiveKind::Other(_) => {}
                PrimitiveKind::Text {
                    color,
                    text,
                    font_id,
                } => {
                    let rgba = color.to_rgb();
                    mode = RenderMode::Text {
                        color: Point4::new(rgba.0, rgba.1, rgba.2, rgba.3),
                    };

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

                    /*
                     * Update the text image.
                     */
                    let positioned_glyphs = text.positioned_glyphs(hidpi_factor);
                    for glyph in positioned_glyphs.iter() {
                        self.cache.queue_glyph(font_id.index(), glyph.clone());
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

                    /*
                     * Build the vertex buffer.
                     */
                    for glyph in positioned_glyphs {
                        if let Some(Some((tex, rect))) =
                            self.cache.rect_for(font_id.index(), &glyph).ok()
                        {
                            let min_px = rect.min.x as f32;
                            let min_py = rect.min.y as f32;
                            let max_px = rect.max.x as f32;
                            let max_py = rect.max.y as f32;

                            vertices.extend_from_slice(&[
                                min_px, min_py, tex.min.x, tex.min.y, min_px, max_py, tex.min.x,
                                tex.max.y, max_px, min_py, tex.max.x, tex.min.y, max_px, min_py,
                                tex.max.x, tex.min.y, min_px, max_py, tex.min.x, tex.max.y, max_px,
                                max_py, tex.max.x, tex.max.y,
                            ]);
                        }
                    }
                }
            }
        }

        verify!(ctxt.enable(Context::DEPTH_TEST));
        verify!(ctxt.disable(Context::BLEND));
        ctxt.scissor(0, 0, width as i32, height as i32);
    }
}

static TRIANGLES_VERTEX_SRC: &'static str = "#version 100
attribute vec2 position;
attribute vec4 color;

uniform vec2 window_size;

varying vec4 v_color;

void main(){
    gl_Position = vec4(position / window_size * 2.0, 0.0, 1.0);
    v_color = color;
}";

static TRIANGLES_FRAGMENT_SRC: &'static str = "#version 100
#ifdef GL_FRAGMENT_PRECISION_HIGH
   precision highp float;
#else
   precision mediump float;
#endif

varying vec4 v_color;

void main() {
  gl_FragColor = v_color;
}";

const TEXT_VERTEX_SRC: &'static str = "
#version 100

uniform vec2 window_size;
uniform vec4 color;

attribute vec2 pos;
attribute vec2 uvs;

varying vec2 v_uvs;
varying vec4 v_color;

void main() {
    gl_Position = vec4((pos.x / window_size.x - 0.5) * 2.0, (0.5 - pos.y / window_size.y) * 2.0, 0.0, 1.0);
    v_uvs       = uvs;
    v_color     = color;
}
";

const TEXT_FRAGMENT_SRC: &'static str = "
#version 100

#ifdef GL_FRAGMENT_PRECISION_HIGH
   precision highp float;
#else
   precision mediump float;
#endif

uniform sampler2D tex0;

varying vec2 v_uvs;
varying vec4 v_color;

void main() {
    gl_FragColor = vec4(v_color.rgb, v_color.a * texture2D(tex0, v_uvs).r);
}
";

const IMAGE_VERTEX_SRC: &'static str = "
#version 100

uniform vec2 window_size;
uniform vec4 color;

attribute vec2 pos;
attribute vec2 uvs;

varying vec2 v_uvs;
varying vec4 v_color;

void main() {
    gl_Position = vec4(pos / window_size * 2.0, 0.0, 1.0);
    v_uvs       = uvs;
    v_color     = color;
}
";

const IMAGE_FRAGMENT_SRC: &'static str = "
#version 100

#ifdef GL_FRAGMENT_PRECISION_HIGH
   precision highp float;
#else
   precision mediump float;
#endif

uniform sampler2D tex0;

varying vec2 v_uvs;
varying vec4 v_color;

void main() {
    gl_FragColor = texture2D(tex0, v_uvs) * v_color;
}
";
