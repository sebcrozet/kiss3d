//! Post processing effect to support the Oculus Rift.

use std::rt::io;
use std::cast;
use std::ptr;
use std::mem;
use std::rt::io::file::FileInfo;
use std::rt::io::Reader;
use std::str;
use gl;
use gl::types::*;
use resources::framebuffers_manager::RenderTarget;
use resources::shaders_manager::{ShadersManager, Other};
use post_processing::post_processing_effect::PostProcessingEffect;

#[path = "../error.rs"]
mod error;

fn load_file(path: &str) -> ~str {
    let s = Path::new(path).open_reader(io::Open).expect("Cannot open the file: " + path).read_to_end();
    str::from_utf8_owned(s)
}

/// An post-processing effect to support the oculus rift.
pub struct OculusStereo {
    priv vshader:      GLuint,
    priv fshader:      GLuint,
    priv program:      GLuint,
    priv time:         f32,
    priv fbo_texture:  GLuint,
    priv fbo_vertices: GLuint,
    priv v_coord:      GLint,
    priv kappa_0:      GLuint,
    priv kappa_1:      GLuint,
    priv kappa_2:      GLuint,
    priv kappa_3:      GLuint,
    priv scale:        GLuint,
    priv scale_in:     GLuint,
    priv w:            f32,
    priv h:            f32
}

impl OculusStereo {
    /// Creates a new OculusStereo post processing effect.
    pub fn new() -> OculusStereo {
        unsafe {
            /* Global */
            let mut vbo_fbo_vertices: GLuint = 0;;
            /* init_resources */
            let fbo_vertices: [GLfloat, ..8] = [
                -1.0, -1.0,
                1.0, -1.0,
                -1.0,  1.0,
                1.0,  1.0,
                ];

            gl::GenBuffers(1, &mut vbo_fbo_vertices);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo_fbo_vertices);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (fbo_vertices.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                cast::transmute(&fbo_vertices[0]),
                gl::STATIC_DRAW);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);

            let (program, vshader, fshader) =
                ShadersManager::load_shader_program(
                    load_file("oculus_vertex_shader.glsl"),
                    load_file("oculus_fragment_shader.glsl"));

            verify!(gl::UseProgram(program));

            let v_coord = gl::GetAttribLocation(program, "v_coord".to_c_str().unwrap());

            OculusStereo {
                vshader:      vshader,
                fshader:      fshader,
                program:      program,
                time:         0.0,
                fbo_texture:  gl::GetUniformLocation(program, "fbo_texture".to_c_str().unwrap()) as GLuint,
                fbo_vertices: vbo_fbo_vertices,
                v_coord:      v_coord,
                kappa_0:  gl::GetUniformLocation(program, "kappa_0".to_c_str().unwrap()) as GLuint,
                kappa_1:  gl::GetUniformLocation(program, "kappa_1".to_c_str().unwrap()) as GLuint,
                kappa_2:  gl::GetUniformLocation(program, "kappa_2".to_c_str().unwrap()) as GLuint,
                kappa_3:  gl::GetUniformLocation(program, "kappa_3".to_c_str().unwrap()) as GLuint,
                scale:    gl::GetUniformLocation(program, "Scale".to_c_str().unwrap()) as GLuint,
                scale_in: gl::GetUniformLocation(program, "ScaleIn".to_c_str().unwrap()) as GLuint,
                h:  1f32, // will be updated in the first update
                w:  1f32, // ditto
            }
        }
    }
}

impl PostProcessingEffect for OculusStereo {
    fn update(&mut self, _: f32, w: f32, h: f32, _: f32, _: f32) {
        self.w = w;
        self.h = h;
    }

    fn draw(&self, shaders_manager: &mut ShadersManager, target: &RenderTarget) {
        shaders_manager.select(Other);
        let scaleFactor = 0.9f32; // firebox: in Oculus SDK example it's "1.0f/Distortion.Scale"
        let aspect = (self.w / 2.0f32) / (self.h); // firebox: rift's "half screen aspect ratio"

        let scale = [0.5f32, aspect];
        let scale_in = [2.0f32 * scaleFactor, 1.0f32 / aspect * scaleFactor];

        verify!(gl::EnableVertexAttribArray(self.v_coord as GLuint));
        /*
         * Configure the post-process effect.
         */
        gl::UseProgram(self.program);
        let kappa = [1.0, 1.7, 0.7, 15.0];
        gl::Uniform1f(self.kappa_0 as GLint, kappa[0]);
        gl::Uniform1f(self.kappa_1 as GLint, kappa[1]);
        gl::Uniform1f(self.kappa_2 as GLint, kappa[2]);
        gl::Uniform1f(self.kappa_3 as GLint, kappa[3]);
        gl::Uniform2f(self.scale as GLint, scale[0], scale[1]);
        gl::Uniform2f(self.scale_in as GLint, scale_in[0], scale_in[1]);

        /*
         * Finalize draw
         */
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

        gl::UseProgram(self.program);
        gl::BindTexture(gl::TEXTURE_2D, target.texture_id());
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
        verify!(gl::DisableVertexAttribArray(self.v_coord as GLuint));
    }
}

impl Drop for OculusStereo {
    fn drop(&mut self) {
        gl::DeleteProgram(self.program);
        gl::DeleteShader(self.vshader);
        gl::DeleteShader(self.fshader);
        unsafe { gl::DeleteBuffers(1, &self.fbo_vertices); }
    }
}
