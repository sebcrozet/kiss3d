use std::util::NonCopyable;
use nalgebra::na::{Vec3, Mat4};
use resource::{Shader, ShaderAttribute, ShaderUniform};

#[path = "../error.rs"]
mod error;

/// Material used to display lines.
pub struct LinesMaterial {
    #[doc(hidden)]
    shader:    Shader,
    #[doc(hidden)]
    pos:       ShaderAttribute<Vec3<f32>>,
    #[doc(hidden)]
    color:     ShaderAttribute<Vec3<f32>>,
    #[doc(hidden)]
    view:      ShaderUniform<Mat4<f32>>,
    #[doc(hidden)]
    ncopy:     NonCopyable
}

impl LinesMaterial {
    /// Creates a new `LinesMaterial`.
    pub fn new() -> LinesMaterial {
        let mut shader = Shader::new_from_str(LINES_VERTEX_SRC, LINES_FRAGMENT_SRC);

        shader.use_program();

        LinesMaterial {
            pos:    shader.get_attrib::<Vec3<f32>>("position").unwrap(),
            color:  shader.get_attrib::<Vec3<f32>>("color").unwrap(),
            view:   shader.get_uniform::<Mat4<f32>>("view").unwrap(),
            shader: shader,
            ncopy:  NonCopyable
        }
    }

    /// Makes active the shader program used by this material.
    pub fn activate(&mut self) {
        self.shader.use_program();
        self.pos.enable();
        self.color.enable();
    }

    /// Makes inactive the shader program used by this material.
    pub fn deactivate(&mut self) {
        self.pos.disable();
        self.color.disable();
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

static ANOTHER_VERY_LONG_STRING: &'static str =
   "#version 120
    varying vec3 Color;
    void main() {
        gl_FragColor = vec4(Color, 1.0);
    }";
