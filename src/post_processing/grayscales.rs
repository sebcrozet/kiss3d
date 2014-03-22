//! Post-processing effect to draw everything in grey-levels.

use gl;
use gl::types::*;
use nalgebra::na::Vec2;
use resource::{Shader, ShaderUniform, ShaderAttribute, RenderTarget, GPUVector, ArrayBuffer, StaticDraw};
use post_processing::post_processing_effect::PostProcessingEffect;

#[path = "../error.rs"]
mod error;

/// Post processing effect which turns everything in gray scales.
pub struct Grayscales {
    priv shader:       Shader,
    priv fbo_texture:  ShaderUniform<GLint>,
    priv v_coord:      ShaderAttribute<Vec2<f32>>,
    priv fbo_vertices: GPUVector<Vec2<GLfloat>>,
}

impl Grayscales {
    /// Creates a new `Grayscales` post processing effect.
    pub fn new() -> Grayscales {
        let fbo_vertices: Vec<Vec2<GLfloat>>  = vec!(
            Vec2::new(-1.0, -1.0),
            Vec2::new(1.0, -1.0),
            Vec2::new(-1.0,  1.0),
            Vec2::new(1.0,  1.0));

        let mut fbo_vertices = GPUVector::new(fbo_vertices, ArrayBuffer, StaticDraw);
        fbo_vertices.load_to_gpu();
        fbo_vertices.unload_from_ram();

        let mut shader = Shader::new_from_str(VERTEX_SHADER, FRAGMENT_SHADER);

        shader.use_program();

        Grayscales {
            fbo_texture:  shader.get_uniform("fbo_texture").unwrap(),
            v_coord:      shader.get_attrib("v_coord").unwrap(),
            fbo_vertices: fbo_vertices,
            shader:       shader
        }
    }
}

impl PostProcessingEffect for Grayscales {
    fn update(&mut self, _: f32, _: f32, _: f32, _: f32, _: f32) {
    }

    fn draw(&mut self, target: &RenderTarget) {
        self.v_coord.enable();

        /*
         * Finalize draw
         */
        self.shader.use_program();
        verify!(gl::ClearColor(0.0, 0.0, 0.0, 1.0));
        verify!(gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT));
        verify!(gl::BindTexture(gl::TEXTURE_2D, target.texture_id()));

        self.fbo_texture.upload(&0);
        self.v_coord.bind(&mut self.fbo_vertices);

        verify!(gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4));

        self.v_coord.disable();
    }
}

static VERTEX_SHADER: &'static str =
    "#version 120
    attribute vec2    v_coord;
    uniform sampler2D fbo_texture;
    varying vec2      f_texcoord;
     
    void main(void) {
      gl_Position = vec4(v_coord, 0.0, 1.0);
      f_texcoord  = (v_coord + 1.0) / 2.0;
    }";

static FRAGMENT_SHADER: &'static str =
    "#version 120
    uniform sampler2D fbo_texture;
    varying vec2      f_texcoord;
    
    void main(void) {
      vec2 texcoord = f_texcoord;
      vec4 color    = texture2D(fbo_texture, texcoord);
      float gray    =  0.2126 * color.r + 0.7152 * color.g + 0.0722 * color.b;
      gl_FragColor  = vec4(gray, gray, gray, color.a);
    }";

