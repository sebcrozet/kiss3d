use glfw;
use na::{Pnt2, Pnt3, Pnt4, Vec2, Vec3, Mat4, Iso3};
use na;
use resource::ShaderUniform;

/// Trait every camera must implement.
pub trait Camera {
    /*
     * Event handling.
     */
    /// Handle a mouse event.
    fn handle_event(&mut self, _window: &glfw::Window, _event: &glfw::WindowEvent);

    /*
     * Transformation-related methods.
     */
    /// The camera position.
    fn eye(&self) -> Pnt3<f32>; // FIXME: should this be here?
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
    fn upload(&self, _pass: usize, uniform: &mut ShaderUniform<Mat4<f32>>) {
        uniform.upload(&self.transformation());
    }

    /// The number of passes required by this camera.
    #[inline]
    fn num_passes(&self) -> usize { 1usize }

    /// Indicates that a pass will begin.
    #[inline]
    fn start_pass(&self, _pass: usize, _window: &glfw::Window) { }

    /// Indicates that the scene has been rendered and the post-processing is being run.
    #[inline]
    fn render_complete(&self, _window: &glfw::Window) { }

    /// Converts a 3d point to 2d screen coordinates, assuming the screen has the size `size`.
    fn project(&self, world_coord: &Pnt3<f32>, size: &Vec2<f32>) -> Vec2<f32> {
        let h_world_coord      = na::to_homogeneous(world_coord);
        let h_normalized_coord = self.transformation() * h_world_coord;

        let normalized_coord: Pnt3<f32> = na::from_homogeneous(&h_normalized_coord);

        Vec2::new(
            (1.0 + normalized_coord.x) * size.x / 2.0,
            (1.0 + normalized_coord.y) * size.y / 2.0)
    }

    /// Converts a point in 2d screen coordinates to a ray (a 3d position and a direction).
    ///
    /// The screen is assumed to have a size given by `size`.
    fn unproject(&self, window_coord: &Pnt2<f32>, size: &Vec2<f32>) -> (Pnt3<f32>, Vec3<f32>) {
        let normalized_coord = Pnt2::new(
            2.0 * window_coord.x  / size.x - 1.0,
            2.0 * -window_coord.y / size.y + 1.0);

        let normalized_begin = Pnt4::new(normalized_coord.x, normalized_coord.y, -1.0, 1.0);
        let normalized_end   = Pnt4::new(normalized_coord.x, normalized_coord.y, 1.0, 1.0);

        let cam = self.inv_transformation();

        let h_unprojected_begin = cam * normalized_begin;
        let h_unprojected_end   = cam * normalized_end;

        let unprojected_begin: Pnt3<f32> = na::from_homogeneous(&h_unprojected_begin);
        let unprojected_end:   Pnt3<f32> = na::from_homogeneous(&h_unprojected_end);

        (unprojected_begin, na::normalize(&(unprojected_end - unprojected_begin)))
    }
}
