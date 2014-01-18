//! A batched line renderer.

use std::ptr;
use std::cast;
use std::mem;
use gl;
use gl::types::*;
use nalgebra::na::Vec3;
use builtin::LinesMaterial;
use camera::Camera;

#[path = "error.rs"]
mod error;

/// Structure which manages the display of short-living lines.
pub struct LineRenderer {
    priv material:  LinesMaterial,
    priv lines:     ~[(Vec3<GLfloat>, Vec3<GLfloat>, Vec3<GLfloat>, Vec3<GLfloat>)],
    priv vbuf:      GLuint,
    priv max_lines: uint
}

impl LineRenderer {
    /// Creates a new lines manager.
    pub fn new() -> LineRenderer {
        let mut vbuf: GLuint = 0;
        
        unsafe { verify!(gl::GenBuffers(1, &mut vbuf)) };

        LineRenderer {
            lines:     ~[],
            vbuf:      vbuf,
            max_lines: 0,
            material:  LinesMaterial::new()
        }
    }
 
    /// Indicates whether some lines have to be drawn.
    pub fn needs_rendering(&self) -> bool {
        self.lines.len() != 0
    }

    /// Adds a line to be drawn during the next frame. Lines are not persistant between frames.
    /// This method must be called for each line to draw, and at each update loop iteration.
    pub fn draw_line(&mut self, a: Vec3<GLfloat>, b: Vec3<GLfloat>, color: Vec3<GLfloat>) {
        self.lines.push((a, color, b, color));
    }

    /// Actually draws the lines.
    pub fn render(&mut self, pass: uint, camera: &mut Camera) {
        if self.lines.len() == 0 { return }

        unsafe {
            self.material.activate();

            /*
             *
             * Setup camera
             *
             */
            camera.upload(pass, self.material.view);

            /*
             *
             * Setup line-related stuffs.
             *
             */
            verify!(gl::BindBuffer(gl::ARRAY_BUFFER, self.vbuf));

            if self.lines.len() > self.max_lines {
                // realloc the vertex buffer
                self.max_lines = self.lines.capacity();

                verify!(gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (self.max_lines * 4 * 3 * mem::size_of::<GLfloat>()) as GLsizeiptr,
                    cast::transmute(&self.lines[0]),
                    gl::STREAM_DRAW));
            }
            else {
                verify!(gl::BufferSubData(
                    gl::ARRAY_BUFFER,
                    0,
                    (self.lines.len() * 4 * 3 * mem::size_of::<GLfloat>()) as GLsizeiptr,
                    cast::transmute(&self.lines[0])));
            }

            verify!(gl::VertexAttribPointer(
                self.material.color,
                3,
                gl::FLOAT,
                gl::FALSE as u8,
                (6 * mem::size_of::<GLfloat>()) as GLint,
                cast::transmute(3 * mem::size_of::<GLfloat>())));

            verify!(gl::VertexAttribPointer(
                self.material.pos,
                3,
                gl::FLOAT,
                gl::FALSE as u8,
                (6 * mem::size_of::<GLfloat>()) as GLint,
                ptr::null()));

            verify!(gl::DrawArrays(gl::LINES, 0, (self.lines.len() * 2) as i32));

            self.material.deactivate();
        }

        self.lines.clear();
    }
}
