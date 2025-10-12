//! A batched line renderer.

use crate::context::Context;
use crate::planar_camera::PlanarCamera;
use crate::resource::{AllocationType, BufferType, Effect, GPUVec, ShaderAttribute, ShaderUniform};
use crate::verify;
use na::{Matrix3, Point2, Point3};

/// Structure which manages the display of short-living lines.
pub struct PlanarLineRenderer {
    shader: Effect,
    pos: ShaderAttribute<Point2<f32>>,
    color: ShaderAttribute<Point3<f32>>,
    view: ShaderUniform<Matrix3<f32>>,
    proj: ShaderUniform<Matrix3<f32>>,
    colors: GPUVec<Point3<f32>>,
    lines: GPUVec<Point2<f32>>,
    line_width: f32,
}

impl Default for PlanarLineRenderer {
    fn default() -> Self {
        Self::new()
    }
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
            shader,
            line_width: 1.0,
        }
    }

    /// Indicates whether some lines have to be drawn.
    pub fn needs_rendering(&self) -> bool {
        !self.lines.is_empty()
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
    pub fn render(&mut self, camera: &mut dyn PlanarCamera) {
        if self.lines.is_empty() {
            return;
        }

        self.shader.use_program();
        self.pos.enable();
        self.color.enable();

        camera.upload(&mut self.proj, &mut self.view);

        self.color.bind_sub_buffer(&mut self.colors, 0, 0);
        self.pos.bind_sub_buffer(&mut self.lines, 0, 0);

        let ctxt = Context::get();
        verify!(ctxt.line_width(self.line_width));
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

    /// Sets the line width for the rendered lines.
    pub fn set_line_width(&mut self, line_width: f32) {
        self.line_width = line_width.max(
            f32::EPSILON, /* Gl will usually round this to 1 pixel */
        );
    }
}

/// WGSL shader for planar line rendering
const PLANAR_LINES_WGSL_SRC: &str = include_str!("planar_lines.wgsl");

/// Vertex shader used by the material to display line.
static LINES_VERTEX_SRC: &str = PLANAR_LINES_WGSL_SRC;
/// Fragment shader used by the material to display line.
static LINES_FRAGMENT_SRC: &str = PLANAR_LINES_WGSL_SRC;
