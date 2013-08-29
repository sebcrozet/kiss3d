// This a simple post-process. I do this only to learn how works post-processing; so it might be
// useless for anybody else.
// This is inspired _a lot_ by: http://en.wikibooks.org/wiki/Opengl::Programming/Post-Processing

use std::cast;
use std::ptr;
use std::sys;
use gl;
use gl::types::*;
use shaders_manager::{ShadersManager, Other};
use post_processing::post_processing_effect::PostProcessingEffect;

#[path = "../error.rs"]
mod error;

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

pub struct Waves {
    vshader:      GLuint,
    fshader:      GLuint,
    program:      GLuint,
    time:         f64,
    offset:       GLuint,
    fbo_texture:  GLuint,
    fbo_vertices: GLuint,
    v_coord:      GLint
}

impl Waves {
    pub fn new() -> Waves {
        unsafe {
            /* Global */
            let vbo_fbo_vertices: GLuint = 0;;
            /* init_resources */
            let fbo_vertices: [GLfloat, ..8] = [
                -1.0, -1.0,
                1.0, -1.0,
                -1.0,  1.0,
                1.0,  1.0,
                ];

            gl::GenBuffers(1, &vbo_fbo_vertices);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo_fbo_vertices);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (fbo_vertices.len() * sys::size_of::<GLfloat>()) as GLsizeiptr,
                cast::transmute(&fbo_vertices[0]),
                gl::STATIC_DRAW);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);

            let (program, vshader, fshader) =
                ShadersManager::load_shader_program(VERTEX_SHADER, FRAGMENT_SHADER);

            verify!(gl::UseProgram(program));

            let v_coord = gl::GetAttribLocation(program, "v_coord".to_c_str().unwrap());
            verify!(gl::EnableVertexAttribArray(v_coord as GLuint));

            Waves {
                vshader:      vshader,
                fshader:      fshader,
                program:      program,
                time:         0.0,
                offset:       gl::GetUniformLocation(program, "offset".to_c_str().unwrap()) as GLuint,
                fbo_texture:  gl::GetUniformLocation(program, "fbo_texture".to_c_str().unwrap()) as GLuint,
                fbo_vertices: vbo_fbo_vertices,
                v_coord:      v_coord
            }
        }
    }
}

impl PostProcessingEffect for Waves {
    fn update(&mut self, dt: f64) {
        self.time = self.time + dt;
    }

    fn draw(&self, shaders_manager: &mut ShadersManager, fbo_texture: GLuint) {
        shaders_manager.select(Other);
        /*
         * Configure the post-process effect.
         */
        gl::UseProgram(self.program);
        let move = self.time * 2.0 * 3.14159 * 0.75;  // 3/4 of a wave cycle per second
        gl::Uniform1f(self.offset as GLint, move as f32);

        /*
         * Finalize draw
         */
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

        gl::UseProgram(self.program);
        gl::BindTexture(gl::TEXTURE_2D, fbo_texture);
        gl::Uniform1i(self.fbo_texture as GLint, /* gl::TEXTURE*/0);

        gl::BindBuffer(gl::ARRAY_BUFFER, self.fbo_vertices);
        unsafe {
            gl::VertexAttribPointer(
                self.v_coord as GLuint,
                2,
                gl::FLOAT,
                gl::FALSE as u8,
                0,
                ptr::null());
        }
        gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
    }
}

impl Drop for Waves {
    fn drop(&self) {
        println("FIXME: release resources used by the `Wave` post-processing effect.")
    }
}
