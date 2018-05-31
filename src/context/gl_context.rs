use context::*;

#[derive(Clone)]
pub struct GLContext;

impl GLContext {
    pub fn new() -> Self {
        GLContext
    }
}
impl Context for GLContext {
    type GLUniformLocation = u32;

    fn get_error(&self) -> u32 {
        gl::get_error()
    }
}

// if attribute_location != -1
