//! An useless post-process effect: waves deformations.

// This a simple post-process. I do this only to learn how works post-processing; so it might be
// useless for anybody else.
// This is inspired _a lot_ by: http://en.wikibooks.org/wiki/Opengl::Programming/Post-Processing

use gl;
use gl::types::*;
use na::Vector2;
use resource::{BufferType, AllocationType, Shader, ShaderUniform, ShaderAttribute, RenderTarget, GPUVec};
use post_processing::post_processing_effect::PostProcessingEffect;

#[path = "../error.rs"]
mod error;

/// An useless post-processing effect mainly to test that everything works correctly.
///
/// It deforms the displayed scene with a wave effect.
pub struct Waves {
    shader:       Shader,
    time:         f32,
    offset:       ShaderUniform<GLfloat>,
    fbo_texture:  ShaderUniform<GLint>,
    v_coord:      ShaderAttribute<Vector2<f32>>,
    fbo_vertices: GPUVec<Vector2<GLfloat>>
}

impl Waves {
    /// Creates a new Waves post processing effect.
    pub fn new() -> Waves {
        let fbo_vertices: Vec<Vector2<GLfloat>>  = vec!(
            Vector2::new(-1.0, -1.0),
            Vector2::new(1.0, -1.0),
            Vector2::new(-1.0,  1.0),
            Vector2::new(1.0,  1.0));

        let mut fbo_vertices = GPUVec::new(fbo_vertices, BufferType::Array, AllocationType::StaticDraw);
        fbo_vertices.load_to_gpu();
        fbo_vertices.unload_from_ram();

        let mut shader = Shader::new_from_str(VERTEX_SHADER, FRAGMENT_SHADER);

        shader.use_program();

        Waves {
            time:         0.0,
            offset:       shader.get_uniform("offset").unwrap(),
            fbo_texture:  shader.get_uniform("fbo_texture").unwrap(),
            v_coord:      shader.get_attrib("v_coord").unwrap(),
            fbo_vertices: fbo_vertices,
            shader:       shader
        }
    }
}

impl PostProcessingEffect for Waves {
    fn update(&mut self, dt: f32, _: f32, _: f32, _: f32, _: f32) {
        self.time = self.time + dt;
    }

    fn draw(&mut self, target: &RenderTarget) {
        /*
         * Configure the post-process effect.
         */
        self.shader.use_program();

        let move_amount = self.time * 2.0 * 3.14159 * 0.75;  // 3/4 of a wave cycle per second

        self.offset.upload(&move_amount);

        /*
         * Finalize draw
         */
        verify!(gl::ClearColor(0.0, 0.0, 0.0, 1.0));
        verify!(gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT));
        verify!(gl::BindTexture(gl::TEXTURE_2D, target.texture_id()));

        self.fbo_texture.upload(&0);
        self.v_coord.enable();
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
    uniform float     offset;
    varying vec2      f_texcoord;
    
    void main(void) {
      vec2 texcoord =  f_texcoord;
      texcoord.x    += sin(texcoord.y * 4 * 2 * 3.14159 + offset) / 100;
      gl_FragColor  =  texture2D(fbo_texture, texcoord);
    }";
