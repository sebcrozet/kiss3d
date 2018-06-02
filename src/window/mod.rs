//! The window, and things to handle the rendering loop and events.

pub(crate) use self::canvas::AbstractCanvas;
pub use self::canvas::Canvas;
#[cfg(not(target_arch = "wasm32"))]
pub use self::gl_canvas::GLCanvas;
pub use self::state::State;
#[cfg(target_arch = "wasm32")]
pub use self::webgl_canvas::WebGLCanvas;
pub use self::window::Window;

mod canvas;
#[cfg(not(target_arch = "wasm32"))]
mod gl_canvas;
mod state;
#[cfg(target_arch = "wasm32")]
mod webgl_canvas;
mod window;
