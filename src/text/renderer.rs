// This whole file is inspired by: https://github.com/jeaye/q3/blob/master/src/client/ui/ttf/renderer.rs
// available under the BSD-3 licence.
// It has been modified to work with gl-rs, nalgebra, and rust-freetype

use std::rc::Rc;
use gl;
use gl::types::*;
use na::{Vec2, Pnt2, Pnt3};
use text::Font;
use camera::Camera;
use resource::{BufferType, AllocationType, Shader, ShaderUniform, ShaderAttribute, GPUVector};

#[path = "../error.rs"]
mod error;

struct TextRenderContext {
    color: Pnt3<f32>,
    font:  Rc<Font>,
    begin: usize,
    size:  usize
}

impl TextRenderContext {
    pub fn new(color: Pnt3<f32>, font: Rc<Font>, begin: usize, size: usize) -> TextRenderContext {
        TextRenderContext {
            color:   color,
            font:    font,
            begin:   begin,
            size:    size
        }
    }
}

/// A ttf text renderer.
pub struct TextRenderer {
    shader:   Shader,
    invsz:    ShaderUniform<Vec2<f32>>,
    tex:      ShaderUniform<GLint>,
    color:    ShaderUniform<Pnt3<f32>>,
    pos:      ShaderAttribute<Pnt2<f32>>,
    uvs:      ShaderAttribute<Pnt2<f32>>,
    contexts: Vec<TextRenderContext>,
    coords:   GPUVector<Pnt2<f32>>,
}

impl TextRenderer {
    /// Creates a new text renderer with `font` as the default font.
    pub fn new() -> TextRenderer {
        let mut shader = Shader::new_from_str(TEXT_VERTEX_SRC, TEXT_FRAGMENT_SRC);

        shader.use_program();

        TextRenderer {
            invsz:    shader.get_uniform("invsz").expect("Could not find invsz"),
            tex:      shader.get_uniform("tex0").expect("Could not find tex0"),
            color:    shader.get_uniform("color").expect("Could not find color"),
            pos:      shader.get_attrib("pos").expect("Could not find pos"),
            uvs:      shader.get_attrib("uvs").expect("Could not find uvs"),
            shader:   shader,
            contexts: Vec::new(),
            coords:   GPUVector::new(Vec::new(), BufferType::Array, AllocationType::StreamDraw)
        }
    }

    /// Adds a piece of text to be drawn during the next frame. The text is not persistent between
    /// frames. This method must be called for each text to draw, and at each update loop
    /// iteration.
    pub fn draw_text(&mut self, text: &str, pos: &Pnt2<f32>, font: &Rc<Font>, color: &Pnt3<f32>) {
        for coords in self.coords.data_mut().iter_mut() {
            let begin = coords.len();

            for (line_count, line) in text.lines().enumerate() {
                let mut temp_pos = pos.clone();
                temp_pos.y       = temp_pos.y + (font.height() as usize * (line_count + 1)) as f32;

                for curr in line.chars() {
                    // XXX: do _not_ use a hashmap!
                    let glyph = match font.glyphs()[curr as usize] {
                        Some(ref g) => g,
                        None        => continue,
                    };

                    let end_x = temp_pos.x + glyph.offset.x;
                    let end_y = -temp_pos.y - (glyph.dimensions.y - glyph.offset.y);
                    let end_w = glyph.dimensions.x;
                    let end_h = glyph.dimensions.y;

                    temp_pos.x = temp_pos.x + glyph.advance.x; 
                    temp_pos.y = temp_pos.y + glyph.advance.y; 

                    // Skip empty glyphs.
                    if end_w <= 0.1 || end_h <= 0.1 {
                        continue;
                    }

                    let adimx = font.atlas_dimensions().x as f32;
                    let adimy = font.atlas_dimensions().y as f32;

                    coords.push(Pnt2::new(end_x, -end_y - end_h));
                    coords.push(Pnt2::new(glyph.tex.x, glyph.tex.y));

                    coords.push(Pnt2::new(end_x, -end_y));
                    coords.push(Pnt2::new(glyph.tex.x, glyph.tex.y + (end_h / adimy)));

                    coords.push(Pnt2::new(end_x + end_w, -end_y));
                    coords.push(Pnt2::new(glyph.tex.x + (end_w / adimx), glyph.tex.y + (end_h / adimy)));

                    coords.push(Pnt2::new(end_x, -end_y - end_h));
                    coords.push(Pnt2::new(glyph.tex.x, glyph.tex.y));

                    coords.push(Pnt2::new(end_x + end_w, -end_y));
                    coords.push(Pnt2::new(glyph.tex.x + (end_w / adimx), glyph.tex.y + (end_h / adimy)));

                    coords.push(Pnt2::new(end_x + end_w, -end_y - end_h));
                    coords.push(Pnt2::new(glyph.tex.x + (end_w / adimx), glyph.tex.y));
                }
            }

            let size = coords.len() - begin;

            if size > 0 {
                self.contexts.push(TextRenderContext::new(color.clone(), font.clone(), begin, size));
            }
        }
    }

    /// Actually draws the text.
    pub fn render(&mut self, width: f32, height: f32) {
        if self.coords.len() == 0 { return }

        self.shader.use_program();

        verify!(gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL));
        verify!(gl::Enable(gl::BLEND));
        verify!(gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA));
        verify!(gl::Disable(gl::DEPTH_TEST));

        self.pos.enable();
        self.uvs.enable();
        self.tex.upload(&0);
        self.invsz.upload(&Vec2::new(1.0 / width, -1.0 / height));

        for ctxt in self.contexts.iter() {
            verify!(gl::BindTexture(gl::TEXTURE_2D, ctxt.font.texture_atlas()));
            verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32));
            verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32));

            self.pos.bind_sub_buffer(&mut self.coords, 1, ctxt.begin + 0);
            self.uvs.bind_sub_buffer(&mut self.coords, 1, ctxt.begin + 1);
            self.color.upload(&ctxt.color);

            verify!(gl::DrawArrays(gl::TRIANGLES, 0, (ctxt.size / 2) as i32));
        }

        self.pos.disable();
        self.uvs.enable();

        verify!(gl::Enable(gl::DEPTH_TEST));
        verify!(gl::Disable(gl::BLEND));

        for coords in self.coords.data_mut().iter_mut() {
            coords.clear()
        }

        self.contexts.clear();
    }
}

/// Vertex shader used by the material to display line.
pub static TEXT_VERTEX_SRC:   &'static str = A_VERY_LONG_STRING;
/// Fragment shader used by the material to display line.
pub static TEXT_FRAGMENT_SRC: &'static str = ANOTHER_VERY_LONG_STRING;

const A_VERY_LONG_STRING: &'static str =
"
#version 120

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

const ANOTHER_VERY_LONG_STRING: &'static str =
"
#version 120

uniform sampler2D tex0;

varying vec2 tex;
varying vec3 Color; 

void main() {
    gl_FragColor = vec4(Color, texture2D(tex0, tex).r); 
}
";
