use std::cast;
use glfw;
use gl;
use gl::types::*;
use nalgebra::vec::Vec3;
use nalgebra::mat::{Mat4, MatCast, Transpose};
use event;

/// Trait every camera must implement.
pub trait Camera {
    /*
     * Event handling.
     */
    /// Handle a mouse event.
    fn handle_mouse(&mut self, &glfw::Window, &event::MouseEvent);
    /// Handle a keyboard event.
    fn handle_keyboard(&mut self, &glfw::Window, &event::KeyboardEvent);
    /// Take in account a change in the framebuffer size.
    fn handle_framebuffer_size_change(&mut self, w: f64, h: f64);

    /*
     * Transformation-related methods.
     */
    /// The camera position.
    fn eye(&self) -> Vec3<f64>; // FIXME: should this be here?
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
    fn upload(&self, view_location: i32) {
        let mut homo = self.transformation();

        homo.transpose();

        let homo32: Mat4<GLfloat> = MatCast::from(homo);

        unsafe {
            gl::UniformMatrix4fv(
                view_location,
                1,
                gl::FALSE as u8,
                cast::transmute(&homo32));
        }
    }
}
