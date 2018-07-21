//! A batched line renderer.

use context::Context;
use na::{Matrix3, Point2, Point3};
use planar_camera::PlanarCamera;
use resource::{AllocationType, BufferType, Effect, GPUVec, ShaderAttribute, ShaderUniform};

#[path = "error.rs"]
mod error;

/// Structure which manages the display of short-living lines.
pub struct PlanarLineRenderer {
    shader: Effect,
    pos: ShaderAttribute<Point2<f32>>,
    color: ShaderAttribute<Point3<f32>>,
    view: ShaderUniform<Matrix3<f32>>,
    proj: ShaderUniform<Matrix3<f32>>,
    colors: GPUVec<Point3<f32>>,
    lines: GPUVec<Point2<f32>>,
}

impl PlanarLineRenderer {
    /// Creates a new lines manager.
    pub fn new() -> PlanarLineRenderer {
        let mut shader = Effect::new_from_str(LINES_VERTEX_SRC, LINES_FRAGMENT_SRC);

        shader.use_program();

        PlanarLineRenderer {
            lines: GPUVec::new(Vec::new(), BufferType::Array, AllocationType::StreamDraw),
            colors: GPUVec::new(Vec::new(), BufferType::Array, AllocationType::StreamDraw),
            pos: shader
                .get_attrib::<Point2<f32>>("position")
                .expect("Failed to get shader attribute."),
            color: shader
                .get_attrib::<Point3<f32>>("color")
                .expect("Failed to get shader attribute."),
            view: shader
                .get_uniform::<Matrix3<f32>>("view")
                .expect("Failed to get shader uniform."),
            proj: shader
                .get_uniform::<Matrix3<f32>>("proj")
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
    pub fn draw_line(&mut self, a: Point2<f32>, b: Point2<f32>, color: Point3<f32>) {
        for lines in self.lines.data_mut().iter_mut() {
            lines.push(a);
            lines.push(b);
        }
        for colors in self.colors.data_mut().iter_mut() {
            colors.push(color);
            colors.push(color);
        }
    }

    /// Actually draws the lines.
    pub fn render(&mut self, camera: &mut PlanarCamera) {
        if self.lines.len() == 0 {
            return;
        }

        self.shader.use_program();
        self.pos.enable();
        self.color.enable();

        camera.upload(&mut self.proj, &mut self.view);

        self.color.bind_sub_buffer(&mut self.colors, 0, 0);
        self.pos.bind_sub_buffer(&mut self.lines, 0, 0);

        let ctxt = Context::get();
        verify!(ctxt.draw_arrays(Context::LINES, 0, self.lines.len() as i32));

        self.pos.disable();
        self.color.disable();

        for lines in self.lines.data_mut().iter_mut() {
            lines.clear()
        }

        for colors in self.colors.data_mut().iter_mut() {
            colors.clear()
        }
    }
}

/// Vertex shader used by the material to display line.
static LINES_VERTEX_SRC: &'static str = A_VERY_LONG_STRING;
/// Fragment shader used by the material to display line.
static LINES_FRAGMENT_SRC: &'static str = ANOTHER_VERY_LONG_STRING;

const A_VERY_LONG_STRING: &'static str = "#version 100
    attribute vec2 position;
    attribute vec3 color;
    varying   vec3 vColor;
    uniform   mat3 proj;
    uniform   mat3 view;

    void main() {
        vec3 projected_pos = proj * view * vec3(position, 1.0);
        projected_pos.z = 0.0;

        gl_Position = vec4(projected_pos, 1.0);
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
