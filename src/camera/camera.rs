use event::WindowEvent;
use na::{self, Isometry3, Matrix4, Point2, Point3, Point4, Vector2, Vector3};
use resource::ShaderUniform;
use window::Canvas;

/// Trait every camera must implement.
pub trait Camera {
    /*
     * Event handling.
     */
    /// Handle a mouse event.
    fn handle_event(&mut self, canvas: &Canvas, event: &WindowEvent);

    /*
     * Transformation-related methods.
     */
    /// The camera position.
    fn eye(&self) -> Point3<f32>; // FIXME: should this be here?
    /// The camera view transform.
    fn view_transform(&self) -> Isometry3<f32>;
    /// The transformation applied by the camera to transform a point in world coordinates to
    /// a point in device coordinates.
    fn transformation(&self) -> Matrix4<f32>;
    /// The transformation applied by the camera to transform point in device coordinates to a
    /// point in world coordinate.
    fn inverse_transformation(&self) -> Matrix4<f32>;
    /// The clipping planes, aka. (`znear`, `zfar`).
    fn clip_planes(&self) -> (f32, f32); // FIXME: should this be here?

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
        pass: usize,
        proj: &mut ShaderUniform<Matrix4<f32>>,
        view: &mut ShaderUniform<Matrix4<f32>>,
    );

    /// The number of passes required by this camera.
    #[inline]
    fn num_passes(&self) -> usize {
        1usize
    }

    /// Indicates that a pass will begin.
    #[inline]
    fn start_pass(&self, _pass: usize, _canvas: &Canvas) {}

    /// Indicates that the scene has been rendered and the post-processing is being run.
    #[inline]
    fn render_complete(&self, _canvas: &Canvas) {}

    /// Converts a 3d point to 2d screen coordinates, assuming the screen has the size `size`.
    fn project(&self, world_coord: &Point3<f32>, size: &Vector2<f32>) -> Vector2<f32> {
        let h_world_coord = world_coord.to_homogeneous();
        let h_normalized_coord = self.transformation() * h_world_coord;

        let normalized_coord = Point3::from_homogeneous(h_normalized_coord).unwrap();

        Vector2::new(
            (1.0 + normalized_coord.x) * size.x / 2.0,
            (1.0 + normalized_coord.y) * size.y / 2.0,
        )
    }

    /// Converts a point in 2d screen coordinates to a ray (a 3d position and a direction).
    ///
    /// The screen is assumed to have a size given by `size`.
    fn unproject(
        &self,
        window_coord: &Point2<f32>,
        size: &Vector2<f32>,
    ) -> (Point3<f32>, Vector3<f32>) {
        let normalized_coord = Point2::new(
            2.0 * window_coord.x / size.x - 1.0,
            2.0 * -window_coord.y / size.y + 1.0,
        );

        let normalized_begin = Point4::new(normalized_coord.x, normalized_coord.y, -1.0, 1.0);
        let normalized_end = Point4::new(normalized_coord.x, normalized_coord.y, 1.0, 1.0);

        let cam = self.inverse_transformation();

        let h_unprojected_begin = cam * normalized_begin;
        let h_unprojected_end = cam * normalized_end;

        let unprojected_begin = Point3::from_homogeneous(h_unprojected_begin.coords).unwrap();
        let unprojected_end = Point3::from_homogeneous(h_unprojected_end.coords).unwrap();

        (
            unprojected_begin,
            na::normalize(&(unprojected_end - unprojected_begin)),
        )
    }
}
