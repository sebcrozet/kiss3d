use event::WindowEvent;
use na::{Matrix3, Point2, Vector2};
use resource::ShaderUniform;
use window::Canvas;

/// Trait every 2D camera must implement.
pub trait PlanarCamera {
    /*
     * Event handling.
     */
    /// Handle a mouse event.
    fn handle_event(&mut self, canvas: &Canvas, event: &WindowEvent);

    /*
     * Update & upload
     */
    /// Update the camera. This is called once at the beginning of the render loop.
    fn update(&mut self, canvas: &Canvas);

    /// Upload the camera view and projection to the gpu. This can be called multiple times on the
    /// render loop.
    #[inline]
    fn upload(
        &self,
        proj: &mut ShaderUniform<Matrix3<f32>>,
        view: &mut ShaderUniform<Matrix3<f32>>,
    );

    /// Computes the 2D world-space coordiates corresponding to the given screen-space coordiates.
    fn unproject(&self, window_coord: &Point2<f32>, window_size: &Vector2<f32>) -> Point2<f32>;
}
