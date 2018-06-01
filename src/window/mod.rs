//! The window, and things to handle the rendering loop and events.

pub use self::canvas::Canvas;
pub use self::event::{Action, Event, EventManager, Events, Key, MouseButton, WindowEvent};
#[cfg(not(target_arch = "wasm32"))]
pub use self::gl_canvas::GLCanvas;
#[cfg(target_arch = "wasm32")]
pub use self::webgl_canvas::WebGLCanvas;
pub use self::window::Window;

mod event;
mod window;

mod canvas;
#[cfg(not(target_arch = "wasm32"))]
mod gl_canvas;
#[cfg(target_arch = "wasm32")]
mod webgl_canvas;
