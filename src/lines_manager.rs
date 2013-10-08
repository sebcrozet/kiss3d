use std::ptr;
use std::cast;
use std::sys;
use gl;
use gl::types::*;
use nalgebra::na::Vec3;
use resources::shaders_manager::LinesShaderContext;

#[path = "error.rs"]
mod error;

/// Structure which manages the display of short-living lines.
struct LinesManager {
    priv lines:     ~[(Vec3<GLfloat>, Vec3<GLfloat>, Vec3<GLfloat>, Vec3<GLfloat>)],
    priv vbuf:      GLuint,
    priv max_lines: uint
}

impl LinesManager {
    /// Creates a new lines manager.
    pub fn new() -> LinesManager {
        let mut vbuf: GLuint = 0;
        
        unsafe { verify!(gl::GenBuffers(1, &mut vbuf)) };

        LinesManager {
            lines:     ~[],
            vbuf:      vbuf,
            max_lines: 0
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
    pub fn upload(&mut self, context: &LinesShaderContext) {
        if self.lines.len() == 0 { return }

        unsafe {
            verify!(gl::BindBuffer(gl::ARRAY_BUFFER, self.vbuf));

            if self.lines.len() > self.max_lines {
                // realloc the vertex buffer
                self.max_lines = self.lines.capacity();

                verify!(gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (self.max_lines * 4 * 3 * sys::size_of::<GLfloat>()) as GLsizeiptr,
                    cast::transmute(&self.lines[0]),
                    gl::STREAM_DRAW));
            }
            else {
                verify!(gl::BufferSubData(
                    gl::ARRAY_BUFFER,
                    0,
                    (self.lines.len() * 4 * 3 * sys::size_of::<GLfloat>()) as GLsizeiptr,
                    cast::transmute(&self.lines[0])));
            }

            verify!(gl::VertexAttribPointer(
                context.color,
                3,
                gl::FLOAT,
                gl::FALSE as u8,
                (6 * sys::size_of::<GLfloat>()) as GLint,
                cast::transmute(3 * sys::size_of::<GLfloat>())));

            verify!(gl::VertexAttribPointer(
                context.pos,
                3,
                gl::FLOAT,
                gl::FALSE as u8,
                (6 * sys::size_of::<GLfloat>()) as GLint,
                ptr::null()));

            verify!(gl::DrawArrays(gl::LINES, 0, (self.lines.len() * 2) as i32));
        }

        self.lines.clear();
    }
}
