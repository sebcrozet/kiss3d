//! Abstractions over OpenGL/WebGL contexts.

pub use self::context::*;
pub use self::gl_context::GLContext;
mod context;
mod gl_context;
