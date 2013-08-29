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
    varying vec2      f_texcoord;
    
    void main(void) {
      vec2 texcoord = f_texcoord;
      vec4 color    = texture2D(fbo_texture, texcoord);
      float gray    =  0.2126 * color.r + 0.7152 * color.g + 0.0722 * color.b;
      gl_FragColor  = vec4(gray, gray, gray, color.a);
    }";

/// Post processing effect which turns everything in grayscales.
pub struct Grayscales {
    priv vshader:      GLuint,
    priv fshader:      GLuint,
    priv program:      GLuint,
    priv time:         f64,
    priv fbo_texture:  GLuint,
    priv fbo_vertices: GLuint,
    priv v_coord:      GLint
}

impl Grayscales {
    /// Creates a new Grayscales post processing effect.
    pub fn new() -> Grayscales {
        unsafe {
            /* Global */
            let vbo_fbo_vertices: GLuint = 0;;
            /* init_resources */
            let fbo_vertices: [GLfloat, ..8] = [
                -1.0, -1.0,
                1.0, -1.0,
                -1.0,  1.0,
                1.0,  1.0];

            verify!(gl::GenBuffers(1, &vbo_fbo_vertices));
            verify!(gl::BindBuffer(gl::ARRAY_BUFFER, vbo_fbo_vertices));
            verify!(gl::BufferData(
                gl::ARRAY_BUFFER,
                (fbo_vertices.len() * sys::size_of::<GLfloat>()) as GLsizeiptr,
                cast::transmute(&fbo_vertices[0]),
                gl::STATIC_DRAW));
            verify!(gl::BindBuffer(gl::ARRAY_BUFFER, 0));

            let (program, vshader, fshader) =
                ShadersManager::load_shader_program(VERTEX_SHADER, FRAGMENT_SHADER);

            verify!(gl::UseProgram(program));

            let v_coord = gl::GetAttribLocation(program, "v_coord".to_c_str().unwrap());
            verify!(gl::EnableVertexAttribArray(v_coord as GLuint));

            Grayscales {
                vshader:      vshader,
                fshader:      fshader,
                program:      program,
                time:         0.0,
                fbo_texture:  gl::GetUniformLocation(program, "fbo_texture".to_c_str().unwrap()) as GLuint,
                fbo_vertices: vbo_fbo_vertices,
                v_coord:      v_coord

            }
        }
    }
}

impl PostProcessingEffect for Grayscales {
    fn update(&mut self, dt: f64) {
        self.time = self.time + dt;
    }

    fn draw(&self, shaders_manager: &mut ShadersManager, fbo_texture: GLuint) {
        shaders_manager.select(Other);

        /*
         * Finalize draw
         */
        verify!(gl::ClearColor(0.0, 0.0, 0.0, 1.0));
        verify!(gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT));

        verify!(gl::UseProgram(self.program));
        verify!(gl::BindTexture(gl::TEXTURE_2D, fbo_texture));
        verify!(gl::Uniform1i(self.fbo_texture as GLint, /* gl::TEXTURE*/0));

        verify!(gl::BindBuffer(gl::ARRAY_BUFFER, self.fbo_vertices));
        unsafe {
            gl::VertexAttribPointer(
                self.v_coord as GLuint,
                2,
                gl::FLOAT,
                gl::FALSE as u8,
                0,
                ptr::null());
        }
        verify!(gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4));
    }
}

impl Drop for Grayscales {
    fn drop(&self) {
        gl::DeleteProgram(self.program);
        gl::DeleteShader(self.vshader);
        gl::DeleteShader(self.fshader);
        unsafe { gl::DeleteBuffers(1, &self.fbo_vertices); }
    }
}
