use gl::types::*;
use shaders_manager::ShadersManager;

pub trait PostProcessingEffect {
    fn update(&mut self, dt: f64);
    fn draw(&self, shaders_manager: &mut ShadersManager, fbo_texture: GLuint);
}
