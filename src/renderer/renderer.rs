use crate::camera::Camera;

/// Trait for implementing custom rendering logic.
///
/// Implement this trait to create custom renderers that can draw additional
/// geometry or effects during the render pipeline. Custom renderers are invoked
/// during each rendering pass.
///
/// # Example
/// ```no_run
/// # use kiss3d::renderer::Renderer;
/// # use kiss3d::camera::Camera;
/// struct MyRenderer;
///
/// impl Renderer for MyRenderer {
///     fn render(&mut self, pass: usize, camera: &mut dyn Camera) {
///         // Custom rendering code here
///         // Use OpenGL calls via kiss3d::context::Context
///     }
/// }
/// ```
pub trait Renderer {
    /// Performs a custom rendering pass.
    ///
    /// This method is called during each rendering pass, after the main scene
    /// has been rendered but before post-processing effects are applied.
    ///
    /// # Arguments
    /// * `pass` - The current rendering pass index (0 for single-pass rendering)
    /// * `camera` - The camera being used for rendering
    fn render(&mut self, pass: usize, camera: &mut dyn Camera);
}
