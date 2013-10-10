use std::cast;
use glfw;
use gl;
use gl::types::*;
use nalgebra::na::{Vec3, Mat4, Iso3};
use nalgebra::na;
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
    fn eye(&self) -> Vec3<f64>; // FIXME: should this be here?
    /// The camera view transform.
    fn view_transform(&self) -> Iso3<f64>;
    /// The transformation applied by the camera to transform a point in world coordinates to
    /// a point in device coordinates.
    fn transformation(&self) -> Mat4<f64>;
    /// The transformation applied by the camera to transform point in device coordinates to a
    /// point in world coordinate.
    fn inv_transformation(&self) -> Mat4<f64>;
    /// The clipping planes, aka. (`znear`, `zfar`).
    fn clip_planes(&self) -> (f64, f64); // FIXME: should this be here?

    /*
     * Update & upload
     */
    // FIXME: dont use glfw::Window
    /// Update the camera. This is called once at the beginning of the render loop.
    fn update(&mut self, window: &glfw::Window);

    /// Upload the camera transfomation to the gpu. This cam be called multiple times on the render
    /// loop.
    fn upload(&self, _pass: uint, view_location: i32) {
        self.upload_mat(view_location, self.transformation());
    }

    // TODO: is there an extra copy here? or does rust avoid it?
    fn upload_mat(&self, view_location: i32, homo_base: Mat4<f64>) {
        let mut homo = homo_base;
        na::transpose(&mut homo);

        let homo32: Mat4<GLfloat> = na::cast(homo);

        unsafe {
            gl::UniformMatrix4fv(
                view_location,
                1,
                gl::FALSE as u8,
                cast::transmute(&homo32));
        }
    }

    fn num_passes(&self) -> uint { 1u }

    fn start_pass(&self, _pass: uint, _window: &glfw::Window) { }

    fn render_complete(&self, _window: &glfw::Window) { }
}
