use std::ptr;
use std::cast;
use std::sys;
use nalgebra::vec::Vec3;
use glcore::consts::GL_VERSION_1_1::*;
use glcore::consts::GL_VERSION_1_5::*;
use glcore::functions::GL_VERSION_1_1::*;
use glcore::functions::GL_VERSION_1_5::*;
use glcore::functions::GL_VERSION_2_0::*;
use glcore::types::GL_VERSION_1_5::*;
use glcore::types::GL_VERSION_1_0::*;
use shaders_manager::LinesShaderContext;

/// Structure which manages the display of short-living lines.
struct LinesManager {
    priv lines:     ~[(Vec3<GLfloat>, Vec3<GLfloat>, Vec3<GLfloat>, Vec3<GLfloat>)],
    priv vbuf:      GLuint,
    priv max_lines: uint
}

impl LinesManager {
    /// Creates a new lines manager.
    pub fn new() -> LinesManager {
        let vbuf: GLuint = 0;
        
        unsafe { glGenBuffers(1, &vbuf) };

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
            glBindBuffer(GL_ARRAY_BUFFER, self.vbuf);

            if self.lines.len() > self.max_lines {
                // realloc the vertex buffer
                self.max_lines = self.lines.capacity();

                glBufferData(
                    GL_ARRAY_BUFFER,
                    (self.max_lines * 4 * 3 * sys::size_of::<GLfloat>()) as GLsizeiptr,
                    cast::transmute(&self.lines[0]),
                    GL_DYNAMIC_DRAW
                    );
            }
            else {
                glBufferSubData(
                    GL_ARRAY_BUFFER,
                    0,
                    (self.lines.len() * 4 * 3 * sys::size_of::<GLfloat>()) as GLsizeiptr,
                    cast::transmute(&self.lines[0])
                    );
            }

            glVertexAttribPointer(
                context.color,
                3,
                GL_FLOAT,
                GL_FALSE,
                (6 * sys::size_of::<GLfloat>()) as GLint,
                cast::transmute(3 * sys::size_of::<GLfloat>()));
            glVertexAttribPointer(
                context.pos,
                3,
                GL_FLOAT,
                GL_FALSE,
                (6 * sys::size_of::<GLfloat>()) as GLint,
                ptr::null());

            glDrawArrays(GL_LINES, 0, (self.lines.len() * 2) as i32);
        }

        self.lines.clear();
    }
}
