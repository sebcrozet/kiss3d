use std::cast;
use std::ptr;
use std::sys;
use gl;
use gl::types::*;
use resources::shaders_manager::{ShadersManager, Other};
use post_processing::post_processing_effect::PostProcessingEffect;

#[path = "../error.rs"]
mod error;

static VERTEX_SHADER: &'static str =
    "#version 120
    attribute vec2    v_coord;
    uniform sampler2D fbo_depth;
    uniform sampler2D fbo_texture;
    uniform float     nx;
    uniform float     ny;
    uniform float     znear;
    uniform float     zfar;
    uniform float     threshold;
    varying vec2      f_texcoord;
     
    void main(void) {
        gl_Position = vec4(v_coord, 0.0, 1.0);
        f_texcoord  = (v_coord + 1.0) / 2.0;
    }";

static FRAGMENT_SHADER: &'static str =
    "#version 120
    uniform sampler2D fbo_depth;
    uniform sampler2D fbo_texture;
    uniform float     nx;
    uniform float     ny;
    uniform float     znear;
    uniform float     zfar;
    uniform float     threshold;
    varying vec2      f_texcoord;

    float lin_depth(vec2 uv) {
        float nlin_depth = texture2D(fbo_depth, uv).x;

        return znear * zfar / ((nlin_depth * (zfar - znear)) - zfar);
    }
    
    void main(void) {
        vec2 texcoord  = f_texcoord;

        float KX[9] = float[](1, 0, -1,
                              2, 0, -2,
                              1, 0, -1);

        float gx = 0;

        for (int i = -1; i < 2; ++i) {
            for (int j = -1; j < 2; ++j) {
                int off = (i + 1) * 3 + j + 1;
                gx += KX[off] * lin_depth(vec2(f_texcoord.x + i * nx, f_texcoord.y + j * ny));
            }
        }

        float KY[9] = float[](1,  2,  1,
                              0,  0,  0,
                              -1, -2, -1);

        float gy = 0;

        for (int i = -1; i < 2; ++i) {
            for (int j = -1; j < 2; ++j) {
                int off = (i + 1) * 3 + j + 1;
                gy += KY[off] * lin_depth(vec2(f_texcoord.x + i * nx, f_texcoord.y + j * ny));
            }
        }

        float gradient = sqrt(gx * gx + gy * gy);

        float edge;

        if (gradient > threshold) {
            edge = 0.0;
        }
        else {
            edge = 1.0 - gradient / threshold;
        }

        vec4 color = texture2D(fbo_texture, texcoord);

        gl_FragColor = vec4(edge * color.xyz, 1.0);
    }";

/// Post processing effect which turns everything in grayscales.
pub struct SobelEdgeHighlight {
    priv shiftx:          f64,
    priv shifty:          f64,
    priv zn:              f64,
    priv zf:              f64,
    priv threshold:       f64,
    priv gl_vshader:      GLuint,
    priv gl_fshader:      GLuint,
    priv gl_program:      GLuint,
    priv gl_nx:           GLuint,
    priv gl_ny:           GLuint,
    priv gl_fbo_depth:    GLuint,
    priv gl_fbo_texture:  GLuint,
    priv gl_fbo_vertices: GLuint,
    priv gl_znear:        GLuint,
    priv gl_zfar:         GLuint,
    priv gl_threshold:    GLuint,
    priv gl_v_coord:      GLint
}

impl SobelEdgeHighlight {
    /// Creates a new SobelEdgeHighlight post processing effect.
    pub fn new(threshold: f64) -> SobelEdgeHighlight {
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

            SobelEdgeHighlight {
                gl_vshader:      vshader,
                gl_fshader:      fshader,
                gl_program:      program,
                shiftx:          0.0,
                shifty:          0.0,
                zn:              0.0,
                zf:              0.0,
                threshold:       threshold,
                gl_nx:           gl::GetUniformLocation(program, "nx".to_c_str().unwrap()) as GLuint,
                gl_ny:           gl::GetUniformLocation(program, "ny".to_c_str().unwrap()) as GLuint,
                gl_fbo_depth:    gl::GetUniformLocation(program, "fbo_depth".to_c_str().unwrap()) as GLuint,
                gl_fbo_texture:  gl::GetUniformLocation(program, "fbo_texture".to_c_str().unwrap()) as GLuint,
                gl_znear:        gl::GetUniformLocation(program, "znear".to_c_str().unwrap()) as GLuint,
                gl_zfar:         gl::GetUniformLocation(program, "zfar".to_c_str().unwrap()) as GLuint,
                gl_threshold:    gl::GetUniformLocation(program, "threshold".to_c_str().unwrap()) as GLuint,
                gl_fbo_vertices: vbo_fbo_vertices,
                gl_v_coord:      v_coord
            }
        }
    }
}

impl PostProcessingEffect for SobelEdgeHighlight {
    fn update(&mut self, _: f64, w: f64, h: f64, znear: f64, zfar: f64) {
        self.shiftx = 2.0 / w;
        self.shifty = 2.0 / h;
        self.zn     = znear;
        self.zf     = zfar;
    }

    fn draw(&self,
            shaders_manager: &mut ShadersManager,
            fbo_texture:     GLuint,
            fbo_depth:       GLuint) {
        shaders_manager.select(Other);

        verify!(gl::EnableVertexAttribArray(self.gl_v_coord as GLuint));
        /*
         * Finalize draw
         */
        verify!(gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT));

        verify!(gl::UseProgram(self.gl_program));

        verify!(gl::Uniform1f(self.gl_threshold as GLint, self.threshold as GLfloat));
        verify!(gl::Uniform1f(self.gl_nx as GLint, self.shiftx as GLfloat));
        verify!(gl::Uniform1f(self.gl_ny as GLint, self.shifty as GLfloat));
        verify!(gl::Uniform1f(self.gl_znear as GLint, self.zn  as GLfloat));
        verify!(gl::Uniform1f(self.gl_zfar  as GLint, self.zf  as GLfloat));

        verify!(gl::ActiveTexture(gl::TEXTURE0));
        verify!(gl::BindTexture(gl::TEXTURE_2D, fbo_texture));
        verify!(gl::Uniform1i(self.gl_fbo_texture as GLint, 0));

        verify!(gl::ActiveTexture(gl::TEXTURE1));
        verify!(gl::BindTexture(gl::TEXTURE_2D, fbo_depth));
        verify!(gl::Uniform1i(self.gl_fbo_depth as GLint, 1));


        verify!(gl::BindBuffer(gl::ARRAY_BUFFER, self.gl_fbo_vertices));

        unsafe {
            gl::VertexAttribPointer(
                self.gl_v_coord as GLuint,
                2,
                gl::FLOAT,
                gl::FALSE as u8,
                0,
                ptr::null());
        }

        verify!(gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4));
        verify!(gl::DisableVertexAttribArray(self.gl_v_coord as GLuint));
    }
}

impl Drop for SobelEdgeHighlight {
    fn drop(&self) {
        gl::DeleteProgram(self.gl_program);
        gl::DeleteShader(self.gl_vshader);
        gl::DeleteShader(self.gl_fshader);
        unsafe { gl::DeleteBuffers(1, &self.gl_fbo_vertices); }
    }
}
