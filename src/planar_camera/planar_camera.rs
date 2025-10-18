use crate::event::WindowEvent;
use crate::resource::ShaderUniform;
use crate::window::Canvas;
use na::{Matrix3, Point2, Vector2};

/// Trait that all 2D camera implementations must implement.
///
/// Planar cameras control the view for 2D overlays and planar scene elements.
/// Unlike 3D cameras, planar cameras work with 2D transformations and projections.
///
/// # Implementations
/// kiss3d provides built-in 2D camera types:
/// - [`PlanarFixedView`](crate::planar_camera::PlanarFixedView) - Static 2D camera
/// - [`Sidescroll`](crate::planar_camera::Sidescroll) - Side-scrolling camera
pub trait PlanarCamera {
    /// Handles window events to update camera state.
    ///
    /// Called for each window event, allowing the camera to respond to user input.
    ///
    /// # Arguments
    /// * `canvas` - Reference to the rendering canvas
    /// * `event` - The window event to handle
    fn handle_event(&mut self, canvas: &Canvas, event: &WindowEvent);

    /// Updates the camera state for the current frame.
    ///
    /// Called once at the beginning of each frame before rendering.
    ///
    /// # Arguments
    /// * `canvas` - Reference to the rendering canvas
    fn update(&mut self, canvas: &Canvas);

    /// Uploads the camera's view and projection matrices to the GPU.
    ///
    /// This can be called multiple times during the render loop.
    ///
    /// # Arguments
    /// * `proj` - Shader uniform for the 2D projection matrix
    /// * `view` - Shader uniform for the 2D view matrix
    fn upload(
        &self,
        proj: &mut ShaderUniform<Matrix3<f32>>,
        view: &mut ShaderUniform<Matrix3<f32>>,
    );

    /// Converts screen coordinates to 2D world coordinates.
    ///
    /// # Arguments
    /// * `window_coord` - The point in screen space (pixels)
    /// * `window_size` - The size of the window in pixels
    ///
    /// # Returns
    /// The corresponding point in 2D world space
    fn unproject(&self, window_coord: &Point2<f32>, window_size: &Vector2<f32>) -> Point2<f32>;
}
