use std::util::NonCopyable;
use gl;
use gl::types::*;
use resource;

#[path = "../error.rs"]
mod error;

/// Material used to display lines.
pub struct LinesMaterial {
    #[doc(hidden)]
    program:   GLuint,
    #[doc(hidden)]
    vshader:   GLuint,
    #[doc(hidden)]
    fshader:   GLuint,
    #[doc(hidden)]
    pos:       GLuint,
    #[doc(hidden)]
    color:     GLuint,
    #[doc(hidden)]
    view:      GLint,
    #[doc(hidden)]
    ncopy:     NonCopyable
}

impl LinesMaterial {
    /// Creates a new `LinesMaterial`.
    pub fn new() -> LinesMaterial {
        unsafe {
            // load the shader
            let (program, vshader, fshader) =
                resource::load_shader_program(LINES_VERTEX_SRC, LINES_FRAGMENT_SRC);

            verify!(gl::UseProgram(program));

            LinesMaterial {
                program: program,
                vshader: vshader,
                fshader: fshader,
                pos:     gl::GetAttribLocation(program,  "position".to_c_str().unwrap()) as GLuint,
                color:   gl::GetAttribLocation(program,  "color".to_c_str().unwrap()) as GLuint,
                view:    gl::GetUniformLocation(program, "view".to_c_str().unwrap()),
                ncopy:   NonCopyable
            }
        }
    }

    /// Makes active the shader program used by this material.
    pub fn activate(&mut self) {
        verify!(gl::UseProgram(self.program));
        verify!(gl::EnableVertexAttribArray(self.pos));
        verify!(gl::EnableVertexAttribArray(self.color));
    }

    /// Makes inactive the shader program used by this material.
    pub fn deactivate(&mut self) {
        verify!(gl::DisableVertexAttribArray(self.pos));
        verify!(gl::DisableVertexAttribArray(self.color));
    }
}

impl Drop for LinesMaterial {
    fn drop(&mut self) {
        gl::DeleteProgram(self.program);
        gl::DeleteShader(self.fshader);
        gl::DeleteShader(self.vshader);
    }
}

/// Vertex shader used by the material to display line.
pub static LINES_VERTEX_SRC:   &'static str = A_VERY_LONG_STRING;
/// Fragment shader used by the material to display line.
pub static LINES_FRAGMENT_SRC: &'static str = ANOTHER_VERY_LONG_STRING;

static A_VERY_LONG_STRING: &'static str =
   "#version 120
    attribute vec3 position;
    attribute vec3 color;
    varying   vec3 Color;
    uniform   mat4   view;
    void main() {
        gl_Position = view * vec4(position, 1.0);
        Color = color;
    }";

// phong lighting (heavily) inspired
// by http://www.opengl.org/sdk/docs/tutorials/ClockworkCoders/lighting.php
static ANOTHER_VERY_LONG_STRING: &'static str =
   "#version 120
    varying vec3 Color;
    void main() {
      gl_FragColor = vec4(Color, 1.0);
    }";
