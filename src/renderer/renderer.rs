use crate::camera::Camera;

/// Trait implemented by custom renderer.
pub trait Renderer {
    /// Perform a rendering pass.
    fn render(&mut self, pass: usize, camera: &mut dyn Camera);
}
