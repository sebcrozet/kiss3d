//! A batched line renderer.

use gl;
use gl::types::*;
use nalgebra::na::Vec3;
use resource::{GPUVector, ArrayBuffer, StreamDraw};
use builtin::LinesMaterial;
use camera::Camera;

#[path = "error.rs"]
mod error;

/// Structure which manages the display of short-living lines.
pub struct LineRenderer {
    priv material:  LinesMaterial,
    priv lines:     GPUVector<Vec3<GLfloat>>,
    priv max_lines: uint
}

impl LineRenderer {
    /// Creates a new lines manager.
    pub fn new() -> LineRenderer {
        let mut vbuf: GLuint = 0;
        
        unsafe { verify!(gl::GenBuffers(1, &mut vbuf)) };

        LineRenderer {
            lines:     GPUVector::new(~[], ArrayBuffer, StreamDraw),
            max_lines: 0,
            material:  LinesMaterial::new()
        }
    }
 
    /// Indicates whether some lines have to be drawn.
    pub fn needs_rendering(&self) -> bool {
        self.lines.len() != 0
    }

    /// Adds a line to be drawn during the next frame. Lines are not persistent between frames.
    /// This method must be called for each line to draw, and at each update loop iteration.
    pub fn draw_line(&mut self, a: Vec3<GLfloat>, b: Vec3<GLfloat>, color: Vec3<GLfloat>) {
        for lines in self.lines.data_mut().mut_iter() {
            lines.push(a);
            lines.push(color);
            lines.push(b);
            lines.push(color);
        }
    }

    /// Actually draws the lines.
    pub fn render(&mut self, pass: uint, camera: &mut Camera) {
        if self.lines.len() == 0 { return }

        self.material.activate();

        camera.upload(pass, &mut self.material.view);

        self.material.color.bind_sub_buffer(&mut self.lines, 1, 1);
        self.material.pos.bind_sub_buffer(&mut self.lines, 1, 0);

        verify!(gl::DrawArrays(gl::LINES, 0, (self.lines.len() / 2) as i32));

        self.material.deactivate();

        for lines in self.lines.data_mut().mut_iter() {
            lines.clear()
        }
    }
}
