use gl;
use gl::types::*;

#[path = "../error.rs"]
mod error;

pub enum RenderTarget {
    Screen,
    Offscreen(GLuint, GLuint)
}

/// A framebuffer manager. It is a simple to to switch between an offscreen framebuffer and the
/// default (window) framebuffer.
pub struct FramebuffersManager {
    priv curr_fbo:   GLuint,
    priv curr_color: GLuint,
    priv curr_depth: GLuint,
    priv fbo:        GLuint
}

impl FramebuffersManager {
    /// Creates a new framebuffer manager.
    pub fn new() -> FramebuffersManager {
        // create an off-screen framebuffer
        let fbo: GLuint = 0;

        unsafe { gl::GenFramebuffers(1, &fbo); }

        // ensure that the current framebuffer is the screen
        verify!(gl::BindFramebuffer(gl::FRAMEBUFFER, 0));

        FramebuffersManager {
            curr_fbo:   0,
            curr_color: 0,
            curr_depth: 0,
            fbo:        fbo
        }
    }

    /// Selects a specific render target
    pub fn select(&mut self, target: RenderTarget) {
        match target {
            Screen => {
                self.do_select(0);
            },
            Offscreen(color, depth) => {
                self.do_select(self.fbo);

                if self.curr_color != color {
                    verify!(gl::FramebufferTexture2D(gl::FRAMEBUFFER,
                                                     gl::COLOR_ATTACHMENT0,
                                                     gl::TEXTURE_2D,
                                                     color,
                                                     0));
                    self.curr_color = color;
                }

                if self.curr_depth != depth {
                    verify!(gl::FramebufferTexture2D(gl::FRAMEBUFFER,
                                                     gl::DEPTH_ATTACHMENT,
                                                     gl::TEXTURE_2D,
                                                     depth,
                                                     0));

                    self.curr_depth = depth;
                }
            }
        }
    }

    fn do_select(&mut self, fbo: GLuint) {
        if self.curr_fbo != fbo {
            verify!(gl::BindFramebuffer(gl::FRAMEBUFFER, fbo));

            self.curr_fbo = fbo;
        }
    }
}

impl Drop for FramebuffersManager {
    fn drop(&self) {
        verify!(gl::BindFramebuffer(gl::FRAMEBUFFER, 0));
        unsafe { verify!(gl::DeleteFramebuffers(1, &self.fbo)); }
    }
}
