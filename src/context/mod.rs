pub use self::context::*;
#[cfg(not(target_arch = "wasm32"))]
pub use self::gl_context::GLContext;
#[cfg(target_arch = "wasm32")]
pub use self::webgl_context::WebGLContext;

mod context;
#[cfg(not(target_arch = "wasm32"))]
mod gl_context;
#[cfg(target_arch = "wasm32")]
mod webgl_context;
