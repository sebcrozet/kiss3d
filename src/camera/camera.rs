use std::cast;
use glfw;
use gl;
use nalgebra::na::{Vec3, Mat4, Iso3};
use event;

/// Trait every camera must implement.
pub trait Camera {
    /*
     * Event handling.
     */
    /// Handle a mouse event.
    fn handle_event(&mut self, &glfw::Window, &event::Event);

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

    /// Upload the camera transfomation to the gpu. This cam be called multiple times on the render
    /// loop.
    fn upload(&self, _pass: uint, view_location: i32) {
        self.upload_mat(view_location, &self.transformation());
    }

    fn upload_mat(&self, view_location: i32, homo: &Mat4<f32>) {
        unsafe {
            gl::UniformMatrix4fv(
                view_location,
                1,
                gl::FALSE as u8,
                cast::transmute(homo));
        }
    }

    fn num_passes(&self) -> uint { 1u }

    fn start_pass(&self, _pass: uint, _window: &glfw::Window) { }

    fn render_complete(&self, _window: &glfw::Window) { }
}
