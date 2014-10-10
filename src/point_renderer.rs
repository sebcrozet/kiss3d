//! A batched point renderer.

use gl;
use gl::types::*;
use na::{Pnt3, Mat4};
use resource::{GPUVector, ArrayBuffer, StreamDraw, Shader, ShaderAttribute, ShaderUniform};
use camera::Camera;

#[path = "error.rs"]
mod error;

/// Structure which manages the display of short-living points.
pub struct PointRenderer {
    shader:     Shader,
    pos:        ShaderAttribute<Pnt3<f32>>,
    color:      ShaderAttribute<Pnt3<f32>>,
    view:       ShaderUniform<Mat4<f32>>,
    points:     GPUVector<Pnt3<GLfloat>>
}

impl PointRenderer {
    /// Creates a new points manager.
    pub fn new() -> PointRenderer {
        let mut shader = Shader::new_from_str(POINTS_VERTEX_SRC, POINTS_FRAGMENT_SRC);

        shader.use_program();

        PointRenderer {
            points:      GPUVector::new(Vec::new(), ArrayBuffer, StreamDraw),
            pos:        shader.get_attrib::<Pnt3<f32>>("position").unwrap(),
            color:      shader.get_attrib::<Pnt3<f32>>("color").unwrap(),
            view:       shader.get_uniform::<Mat4<f32>>("view").unwrap(),
            shader:     shader
        }
    }
 
    /// Indicates whether some points have to be drawn.
    pub fn needs_rendering(&self) -> bool {
        self.points.len() != 0
    }

    /// Adds a line to be drawn during the next frame. Lines are not persistent between frames.
    /// This method must be called for each line to draw, and at each update loop iteration.
    pub fn draw_point(&mut self, pt: Pnt3<GLfloat>, color: Pnt3<GLfloat>) {
        for points in self.points.data_mut().iter_mut() {
            points.push(pt);
            points.push(color);
        }
    }

    /// Actually draws the points.
    pub fn render(&mut self, pass: uint, camera: &mut Camera) {
        if self.points.len() == 0 { return }

        self.shader.use_program();
        self.pos.enable();
        self.color.enable();

        camera.upload(pass, &mut self.view);

        self.color.bind_sub_buffer(&mut self.points, 1, 1);
        self.pos.bind_sub_buffer(&mut self.points, 1, 0);

        verify!(gl::DrawArrays(gl::POINTS, 0, (self.points.len() / 2) as i32));

        self.pos.disable();
        self.color.disable();

        for points in self.points.data_mut().iter_mut() {
            points.clear()
        }
    }
}

/// Vertex shader used by the material to display line.
pub static POINTS_VERTEX_SRC:   &'static str = A_VERY_LONG_STRING;
/// Fragment shader used by the material to display line.
pub static POINTS_FRAGMENT_SRC: &'static str = ANOTHER_VERY_LONG_STRING;

const A_VERY_LONG_STRING: &'static str =
   "#version 120
    attribute vec3 position;
    attribute vec3 color;
    varying   vec3 Color;
    uniform   mat4   view;
    void main() {
        gl_Position = view * vec4(position, 1.0);
        Color = color;
    }";

const ANOTHER_VERY_LONG_STRING: &'static str =
   "#version 120
    varying vec3 Color;
    void main() {
        gl_FragColor = vec4(Color, 1.0);
    }";
