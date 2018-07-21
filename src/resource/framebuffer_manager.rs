//! Resource manager to allocate and switch between framebuffers.

use context::{Context, Framebuffer, Texture};

#[path = "../error.rs"]
mod error;

/// The target to every rendering call.
pub enum RenderTarget {
    /// The screen (main framebuffer).
    Screen,
    /// An off-screen buffer.
    Offscreen(OffscreenBuffers),
}

/// OpenGL identifiers to an off-screen buffer.
pub struct OffscreenBuffers {
    texture: Texture,
    depth: Option<Texture>,
}

impl RenderTarget {
    /// Returns an opengl handle to the off-screen texture buffer.
    ///
    /// Returns `None` if the texture is off-screen.
    pub fn texture_id(&self) -> Option<&Texture> {
        match *self {
            RenderTarget::Screen => None,
            RenderTarget::Offscreen(ref o) => Some(&o.texture),
        }
    }

    /// Returns an opengl handle to the off-screen depth buffer.
    ///
    /// Returns `None` if the texture is off-screen.
    #[cfg(not(any(target_arch = "wasm32", target_arch = "asmjs")))]
    pub fn depth_id(&self) -> Option<&Texture> {
        match *self {
            RenderTarget::Screen => None,
            RenderTarget::Offscreen(ref o) => o.depth.as_ref(),
        }
    }

    /// Resizes this render target.
    pub fn resize(&mut self, w: f32, h: f32) {
        let ctxt = Context::get();

        match *self {
            RenderTarget::Screen => {
                verify!(ctxt.viewport(0, 0, w as i32, h as i32));
            }
            RenderTarget::Offscreen(ref o) => {
                // Update the fbo
                verify!(ctxt.bind_texture(Context::TEXTURE_2D, Some(&o.texture)));
                verify!(ctxt.tex_image2d(
                    Context::TEXTURE_2D,
                    0,
                    Context::RGBA as i32,
                    w as i32,
                    h as i32,
                    0,
                    Context::RGBA,
                    None
                ));
                verify!(ctxt.bind_texture(Context::TEXTURE_2D, None));

                if cfg!(not(any(target_arch = "wasm32", target_arch = "asmjs"))) {
                    verify!(ctxt.bind_texture(Context::TEXTURE_2D, o.depth.as_ref()));
                    verify!(ctxt.tex_image2d(
                        Context::TEXTURE_2D,
                        0,
                        Context::DEPTH_COMPONENT as i32,
                        w as i32,
                        h as i32,
                        0,
                        Context::DEPTH_COMPONENT,
                        None
                    ));
                    verify!(ctxt.bind_texture(Context::TEXTURE_2D, None));
                }
            }
        }
    }
}

/// A framebuffer manager. It is a simple to to switch between an off-screen framebuffer and the
/// default (window) framebuffer.
pub struct FramebufferManager {
    fbo_onscreen: bool,
    fbo: Framebuffer,
}

impl FramebufferManager {
    /// Creates a new framebuffer manager.
    pub fn new() -> FramebufferManager {
        let ctxt = Context::get();

        // create an off-screen framebuffer
        let fbo = ctxt
            .create_framebuffer()
            .expect("Framebuffer creation failed.");

        // ensure that the current framebuffer is the screen
        verify!(ctxt.bind_framebuffer(Context::FRAMEBUFFER, None));

        FramebufferManager {
            fbo_onscreen: true,
            fbo: fbo,
        }
    }

    /// Creates a new render target. A render target is the combination of a color buffer and a
    /// depth buffer.
    pub fn new_render_target(width: usize, height: usize) -> RenderTarget {
        let ctxt = Context::get();

        /* Texture */
        verify!(ctxt.active_texture(Context::TEXTURE0));
        let fbo_texture = verify!(
            ctxt.create_texture()
                .expect("Failde to create framebuffer object texture.")
        );
        verify!(ctxt.bind_texture(Context::TEXTURE_2D, Some(&fbo_texture)));
        verify!(ctxt.tex_parameteri(
            Context::TEXTURE_2D,
            Context::TEXTURE_MAG_FILTER,
            Context::LINEAR as i32
        ));
        verify!(ctxt.tex_parameteri(
            Context::TEXTURE_2D,
            Context::TEXTURE_MIN_FILTER,
            Context::LINEAR as i32
        ));
        verify!(ctxt.tex_parameteri(
            Context::TEXTURE_2D,
            Context::TEXTURE_WRAP_S,
            Context::CLAMP_TO_EDGE as i32
        ));
        verify!(ctxt.tex_parameteri(
            Context::TEXTURE_2D,
            Context::TEXTURE_WRAP_T,
            Context::CLAMP_TO_EDGE as i32
        ));
        verify!(ctxt.tex_image2d(
            Context::TEXTURE_2D,
            0,
            Context::RGBA as i32,
            width as i32,
            height as i32,
            0,
            Context::RGBA,
            None
        ));
        verify!(ctxt.bind_texture(Context::TEXTURE_2D, None));

        /* Depth buffer */
        if cfg!(not(any(target_arch = "wasm32", target_arch = "asmjs"))) {
            verify!(ctxt.active_texture(Context::TEXTURE1));
            let fbo_depth = verify!(ctxt.create_texture().expect("Failed to create a texture."));
            verify!(ctxt.bind_texture(Context::TEXTURE_2D, Some(&fbo_depth)));
            verify!(ctxt.tex_parameteri(
                Context::TEXTURE_2D,
                Context::TEXTURE_MAG_FILTER,
                Context::LINEAR as i32
            ));
            verify!(ctxt.tex_parameteri(
                Context::TEXTURE_2D,
                Context::TEXTURE_MIN_FILTER,
                Context::LINEAR as i32
            ));
            verify!(ctxt.tex_parameteri(
                Context::TEXTURE_2D,
                Context::TEXTURE_WRAP_S,
                Context::CLAMP_TO_EDGE as i32
            ));
            verify!(ctxt.tex_parameteri(
                Context::TEXTURE_2D,
                Context::TEXTURE_WRAP_T,
                Context::CLAMP_TO_EDGE as i32
            ));
            verify!(ctxt.tex_image2di(
                Context::TEXTURE_2D,
                0,
                Context::DEPTH_COMPONENT as i32,
                width as i32,
                height as i32,
                0,
                Context::DEPTH_COMPONENT,
                None
            ));
            verify!(ctxt.bind_texture(Context::TEXTURE_2D, None));

            RenderTarget::Offscreen(OffscreenBuffers {
                texture: fbo_texture,
                depth: Some(fbo_depth),
            })
        } else {
            RenderTarget::Offscreen(OffscreenBuffers {
                texture: fbo_texture,
                depth: None,
            })
        }
    }

    /// Returns the render target associated with the screen.
    pub fn screen() -> RenderTarget {
        RenderTarget::Screen
    }

    /// Selects a specific render target
    pub fn select(&mut self, target: &RenderTarget) {
        match *target {
            RenderTarget::Screen => {
                self.select_onscreen();
            }
            RenderTarget::Offscreen(ref o) => {
                let ctxt = Context::get();
                self.select_fbo();

                // FIXME: don't switch if the current texture is
                // already o.texture ?
                verify!(ctxt.framebuffer_texture2d(
                    Context::FRAMEBUFFER,
                    Context::COLOR_ATTACHMENT0,
                    Context::TEXTURE_2D,
                    Some(&o.texture),
                    0
                ));
                verify!(ctxt.framebuffer_texture2d(
                    Context::FRAMEBUFFER,
                    Context::DEPTH_ATTACHMENT,
                    Context::TEXTURE_2D,
                    o.depth.as_ref(),
                    0
                ));
            }
        }
    }

    fn select_onscreen(&mut self) {
        if !self.fbo_onscreen {
            verify!(Context::get().bind_framebuffer(Context::FRAMEBUFFER, None));
            self.fbo_onscreen = true;
        }
    }

    fn select_fbo(&mut self) {
        if self.fbo_onscreen {
            verify!(Context::get().bind_framebuffer(Context::FRAMEBUFFER, Some(&self.fbo)));
            self.fbo_onscreen = false;
        }
    }
}

impl Drop for FramebufferManager {
    fn drop(&mut self) {
        let ctxt = Context::get();
        if ctxt.is_framebuffer(Some(&self.fbo)) {
            verify!(ctxt.bind_framebuffer(Context::FRAMEBUFFER, None));
            verify!(ctxt.delete_framebuffer(Some(&self.fbo)));
        }
    }
}

impl Drop for OffscreenBuffers {
    fn drop(&mut self) {
        let ctxt = Context::get();
        if ctxt.is_texture(Some(&self.texture)) {
            verify!(ctxt.delete_texture(Some(&self.texture)));
        }
        if ctxt.is_texture(self.depth.as_ref()) {
            verify!(ctxt.delete_texture(self.depth.as_ref()));
        }
    }
}
