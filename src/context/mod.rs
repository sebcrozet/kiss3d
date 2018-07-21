//! Abstractions over OpenGL/WebGL contexts.

pub use self::context::*;
#[cfg(not(any(target_arch = "wasm32", target_arch = "asmjs")))]
pub use self::gl_context::GLContext;
#[cfg(any(target_arch = "wasm32", target_arch = "asmjs"))]
pub use self::webgl_context::WebGLContext;

mod context;
#[cfg(not(any(target_arch = "wasm32", target_arch = "asmjs")))]
mod gl_context;
#[cfg(any(target_arch = "wasm32", target_arch = "asmjs"))]
mod webgl_bindings;
#[cfg(any(target_arch = "wasm32", target_arch = "asmjs"))]
mod webgl_context;
