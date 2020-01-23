//! The window, and things to handle the rendering loop and events.

pub(crate) use self::canvas::AbstractCanvas;
pub use self::canvas::{Canvas, CanvasSetup, NumSamples};
#[cfg(not(any(target_arch = "wasm32", target_arch = "asmjs")))]
pub use self::gl_canvas::GLCanvas;
pub use self::state::State;
//#[cfg(any(target_arch = "wasm32", target_arch = "asmjs"))]
//pub use self::webgl_canvas::WebGLCanvas;
pub use self::window::Window;

mod canvas;
#[cfg(not(any(target_arch = "wasm32", target_arch = "asmjs")))]
mod gl_canvas;
mod state;
//#[cfg(any(target_arch = "wasm32", target_arch = "asmjs"))]
//mod webgl_canvas;
#[cfg(feature="web_sys")]
mod webgl_websys_canvas;
#[cfg(feature="web_sys")]
pub use self::webgl_websys_canvas::WebGLCanvas;
mod window;

