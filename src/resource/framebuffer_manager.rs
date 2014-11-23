//! Resource manager to allocate and switch between framebuffers.

use std::ptr;
use std::kinds::marker::NoCopy;
use gl;
use gl::types::*;

#[path = "../error.rs"]
mod error;

/// The target to every rendering call.
pub enum RenderTarget {
    /// The screen (main framebuffer).
    Screen,
    /// An off-screen buffer.
    Offscreen(OffscreenBuffers)
}

/// OpenGL identifiers to an off-screen buffer.
pub struct OffscreenBuffers {
    texture: GLuint,
    depth:   GLuint,
    ncpy:    NoCopy
}

impl RenderTarget {
    /// Returns an opengl handle to the off-screen texture buffer.
    pub fn texture_id(&self) -> GLuint {
        match *self {
            RenderTarget::Screen           => 0,
            RenderTarget::Offscreen(ref o) => o.texture
        }
    }

    /// Returns an opengl handle to the off-screen depth buffer.
    pub fn depth_id(&self) -> GLuint {
        match *self {
            RenderTarget::Screen           => 0,
            RenderTarget::Offscreen(ref o) => o.depth
        }
    }

    /// Resizes this render target.
    pub fn resize(&mut self, w: f32, h: f32) {
        match *self {
            RenderTarget::Screen => {
                verify!(gl::Viewport(0, 0, w as i32, h as i32));
            },
            RenderTarget::Offscreen(ref o) => {
                // Update the fbo
                verify!(gl::BindTexture(gl::TEXTURE_2D, o.texture));
                unsafe {
                    verify!(gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as GLint, w as GLint, h as GLint, 0,
                    gl::RGBA, gl::UNSIGNED_BYTE, ptr::null()));
                }
                verify!(gl::BindTexture(gl::TEXTURE_2D, 0));

                verify!(gl::BindTexture(gl::TEXTURE_2D, o.depth));
                unsafe {
                    verify!(gl::TexImage2D(gl::TEXTURE_2D, 0, gl::DEPTH_COMPONENT as GLint, w as GLint, h as GLint, 0,
                    gl::DEPTH_COMPONENT, gl::UNSIGNED_BYTE, ptr::null()));
                }
                verify!(gl::BindTexture(gl::TEXTURE_2D, 0));
            }
        }
    }
}

/// A framebuffer manager. It is a simple to to switch between an off-screen framebuffer and the
/// default (window) framebuffer.
pub struct FramebufferManager {
    curr_fbo:   GLuint,
    curr_color: GLuint,
    curr_depth: GLuint,
    fbo:        GLuint
}

impl FramebufferManager {
    /// Creates a new framebuffer manager.
    pub fn new() -> FramebufferManager {
        // create an off-screen framebuffer
        let mut fbo: GLuint = 0;

        unsafe { gl::GenFramebuffers(1, &mut fbo); }

        // ensure that the current framebuffer is the screen
        verify!(gl::BindFramebuffer(gl::FRAMEBUFFER, 0));

        FramebufferManager {
            curr_fbo:   0,
            curr_color: 0,
            curr_depth: 0,
            fbo:        fbo
        }
    }

    /// Creates a new render target. A render target is the combination of a color buffer and a
    /// depth buffer.
    pub fn new_render_target(width: uint, height: uint) -> RenderTarget {
        let mut fbo_texture: GLuint = 0;
        let mut fbo_depth:   GLuint = 0;

        /* Texture */
        verify!(gl::ActiveTexture(gl::TEXTURE0));
        unsafe { verify!(gl::GenTextures(1, &mut fbo_texture)); }
        verify!(gl::BindTexture(gl::TEXTURE_2D, fbo_texture));
        verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint));
        verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint));
        verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint));
        verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint));
        unsafe {
            verify!(gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as GLint, width as GLint, height as GLint,
            0, gl::RGBA, gl::UNSIGNED_BYTE, ptr::null()));
        }
        verify!(gl::BindTexture(gl::TEXTURE_2D, 0));

        /* Depth buffer */
        verify!(gl::ActiveTexture(gl::TEXTURE1));
        unsafe { verify!(gl::GenTextures(1, &mut fbo_depth)); }
        verify!(gl::BindTexture(gl::TEXTURE_2D, fbo_depth));
        verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint));
        verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint));
        verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint));
        verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint));
        unsafe {
            verify!(gl::TexImage2D(gl::TEXTURE_2D, 0, gl::DEPTH_COMPONENT as GLint, width as GLint, height as GLint,
            0, gl::DEPTH_COMPONENT, gl::UNSIGNED_BYTE, ptr::null()));
        }
        verify!(gl::BindTexture(gl::TEXTURE_2D, 0));

        RenderTarget::Offscreen(OffscreenBuffers { texture: fbo_texture, depth: fbo_depth, ncpy: NoCopy })
    }

    /// Returns the render target associated with the screen.
    pub fn screen() -> RenderTarget {
        RenderTarget::Screen
    }

    /// Selects a specific render target
    pub fn select(&mut self, target: &RenderTarget) {
        match *target {
            RenderTarget::Screen => {
                self.do_select(0);
                self.curr_color = 0;
                self.curr_depth = 0;
            },
            RenderTarget::Offscreen(ref o) => {
                let fbo = self.fbo;
                self.do_select(fbo);

                if self.curr_color != o.texture {
                    verify!(gl::FramebufferTexture2D(gl::FRAMEBUFFER,
                                                     gl::COLOR_ATTACHMENT0,
                                                     gl::TEXTURE_2D,
                                                     o.texture,
                                                     0));
                    self.curr_color = o.texture;
                }

                if self.curr_depth != o.depth {
                    verify!(gl::FramebufferTexture2D(gl::FRAMEBUFFER,
                                                     gl::DEPTH_ATTACHMENT,
                                                     gl::TEXTURE_2D,
                                                     o.depth,
                                                     0));

                    self.curr_depth = o.depth;
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

impl Drop for FramebufferManager {
    fn drop(&mut self) {
        verify!(gl::BindFramebuffer(gl::FRAMEBUFFER, 0));
        unsafe { verify!(gl::DeleteFramebuffers(1, &self.fbo)); }
    }
}

impl Drop for OffscreenBuffers {
    fn drop(&mut self) {
        unsafe { verify!(gl::DeleteBuffers(1, &self.texture)); }
        unsafe { verify!(gl::DeleteBuffers(1, &self.depth)); }
    }
}
