//! The window, and things to handle the rendering loop and events.

mod canvas;
#[cfg(not(target_arch = "wasm32"))]
mod gl_canvas;
mod state;
#[cfg(target_arch = "wasm32")]
mod webgl_canvas;
mod window;
mod window_cache;

pub(crate) use canvas::AbstractCanvas;
pub use canvas::{Canvas, CanvasSetup, NumSamples, RenderLoopClosure};
#[cfg(not(target_arch = "wasm32"))]
pub use gl_canvas::GLCanvas;
pub use state::State;
#[cfg(target_arch = "wasm32")]
pub use webgl_canvas::WebGLCanvas;
pub use window::Window;
pub(crate) use window_cache::WINDOW_CACHE;
