//! A batched line renderer.

use crate::camera::Camera;
use crate::context::Context;
use crate::renderer::Renderer;
use crate::resource::{AllocationType, BufferType, Effect, GPUVec, ShaderAttribute, ShaderUniform};
use na::{Matrix4, Point3};

#[path = "../error.rs"]
mod error;

/// Structure which manages the display of short-living lines.
pub struct LineRenderer {
    shader: Effect,
    pos: ShaderAttribute<Point3<f32>>,
    color: ShaderAttribute<Point3<f32>>,
    view: ShaderUniform<Matrix4<f32>>,
    proj: ShaderUniform<Matrix4<f32>>,
    lines: GPUVec<Point3<f32>>,
    line_width: u32,
}

impl LineRenderer {
    /// Creates a new lines manager.
    pub fn new() -> LineRenderer {
        let mut shader = Effect::new_from_str(LINES_VERTEX_SRC, LINES_FRAGMENT_SRC);

        shader.use_program();

        LineRenderer {
            lines: GPUVec::new(Vec::new(), BufferType::Array, AllocationType::StreamDraw),
            pos: shader
                .get_attrib::<Point3<f32>>("position")
                .expect("Failed to get shader attribute."),
            color: shader
                .get_attrib::<Point3<f32>>("color")
                .expect("Failed to get shader attribute."),
            view: shader
                .get_uniform::<Matrix4<f32>>("view")
                .expect("Failed to get shader uniform."),
            proj: shader
                .get_uniform::<Matrix4<f32>>("proj")
                .expect("Failed to get shader uniform."),
            shader: shader,
        }
    }

    /// Indicates whether some lines have to be drawn.
    pub fn needs_rendering(&self) -> bool {
        self.lines.len() != 0
    }

    /// Adds a line to be drawn during the next frame. Lines are not persistent between frames.
    /// This method must be called for each line to draw, and at each update loop iteration.
    pub fn draw_line(&mut self, a: Point3<f32>, b: Point3<f32>, color: Point3<f32>) {
        for lines in self.lines.data_mut().iter_mut() {
            lines.push(a);
            lines.push(color);
            lines.push(b);
            lines.push(color);
        }
    }

    /// Sets the line width for the rendered lines.
    pub fn set_line_width(&mut self, line_width: f32) {
        self.line_width = line_width;
    }
}

impl Renderer for LineRenderer {
    /// Actually draws the lines.
    fn render(&mut self, pass: usize, camera: &mut dyn Camera) {
        if self.lines.len() == 0 {
            return;
        }

        self.shader.use_program();
        self.pos.enable();
        self.color.enable();

        camera.upload(pass, &mut self.proj, &mut self.view);

        self.color.bind_sub_buffer(&mut self.lines, 1, 1);
        self.pos.bind_sub_buffer(&mut self.lines, 1, 0);

        let ctxt = Context::get();
        verify!(ctxt.draw_arrays(Context::LINES, 0, (self.lines.len() / 2) as i32));
        verify!(ctxt.line_width(self.line_width));

        self.pos.disable();
        self.color.disable();

        for lines in self.lines.data_mut().iter_mut() {
            lines.clear()
        }
    }
}

/// Vertex shader used by the material to display line.
pub static LINES_VERTEX_SRC: &'static str = A_VERY_LONG_STRING;
/// Fragment shader used by the material to display line.
pub static LINES_FRAGMENT_SRC: &'static str = ANOTHER_VERY_LONG_STRING;

const A_VERY_LONG_STRING: &'static str = "#version 100
    attribute vec3 position;
    attribute vec3 color;
    varying   vec3 vColor;
    uniform   mat4 proj;
    uniform   mat4 view;
    void main() {
        gl_Position = proj * view * vec4(position, 1.0);
        vColor = color;
    }";

const ANOTHER_VERY_LONG_STRING: &'static str = "#version 100
#ifdef GL_FRAGMENT_PRECISION_HIGH
   precision highp float;
#else
   precision mediump float;
#endif

    varying vec3 vColor;
    void main() {
        gl_FragColor = vec4(vColor, 1.0);
    }";
