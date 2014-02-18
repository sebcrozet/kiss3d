use glfw;
use nalgebra::na::{Vec3, Mat4, Iso3};
use resource::ShaderUniform;

/// Trait every camera must implement.
pub trait Camera {
    /*
     * Event handling.
     */
    /// Handle a mouse event.
    fn handle_event(&mut self, &glfw::Window, &glfw::WindowEvent);

    /*
     * Transformation-related methods.
     */
    /// The camera position.
    fn eye(&self) -> Vec3<f32>; // FIXME: should this be here?
    /// The camera view transform.
    fn view_transform(&self) -> Iso3<f32>;
    /// The transformation applied by the camera to transform a point in world coordinates to
    /// a point in device coordinates.
    fn transformation(&self) -> Mat4<f32>;
    /// The transformation applied by the camera to transform point in device coordinates to a
    /// point in world coordinate.
    fn inv_transformation(&self) -> Mat4<f32>;
    /// The clipping planes, aka. (`znear`, `zfar`).
    fn clip_planes(&self) -> (f32, f32); // FIXME: should this be here?

    /*
     * Update & upload
     */
    // FIXME: dont use glfw::Window
    /// Update the camera. This is called once at the beginning of the render loop.
    fn update(&mut self, window: &glfw::Window);

    /// Upload the camera transformation to the gpu. This can be called multiple times on the
    /// render loop.
    #[inline]
    fn upload(&self, _pass: uint, uniform: &mut ShaderUniform<Mat4<f32>>) {
        uniform.upload(&self.transformation());
    }

    /// The number of passes required by this camera.
    fn num_passes(&self) -> uint { 1u }

    /// Indicates that a pass will begin.
    fn start_pass(&self, uint, &glfw::Window) { }

    /// Indicates that the scene has been rendered and the post-processing is being run.
    fn render_complete(&self, &glfw::Window) { }
}
