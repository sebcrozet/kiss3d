use camera::{Camera, Camera2};
use event::WindowEvent;
use na::{self, Isometry3, Matrix3, Matrix4, Perspective3, Point2, Point3, Vector2, Vector3};
use resource::ShaderUniform;
use std::f32;
use window::Canvas;

/// Arc-ball camera mode.
///
/// An arc-ball camera is a camera rotating around a fixed point (the focus point) and always
/// looking at it. The following inputs are handled:
///
/// * Left button press + drag - rotates the camera around the focus point
/// * Right button press + drag - translates the focus point on the plane orthogonal to the view
/// direction
/// * Scroll in/out - zoom in/out
/// * Enter key - set the focus point to the origin
#[derive(Clone, Debug)]
pub struct StaticCamera {
    projection: Perspective3<f32>,
    proj: Matrix4<f32>,
    inv_proj: Matrix4<f32>,
    proj2: Matrix3<f32>,
    inv_proj2: Matrix3<f32>,
}

impl StaticCamera {
    /// Create a new arc-ball camera.
    pub fn new() -> StaticCamera {
        StaticCamera::new_with_frustrum(f32::consts::PI / 4.0, 0.1, 1024.0)
    }

    /// Creates a new arc ball camera with default sensitivity values.
    pub fn new_with_frustrum(fov: f32, znear: f32, zfar: f32) -> StaticCamera {
        StaticCamera {
            projection: Perspective3::new(800.0 / 600.0, fov, znear, zfar),
            proj: na::one(),
            inv_proj: na::one(),
            proj2: na::one(),
            inv_proj2: na::one(),
        }
    }

    fn update_projviews(&mut self) {
        self.proj = *self.projection.as_matrix();
        let _ = self
            .proj
            .try_inverse()
            .map(|inv_proj| self.inv_proj = inv_proj);
    }
}

impl Camera for StaticCamera {
    fn clip_planes(&self) -> (f32, f32) {
        (self.projection.znear(), self.projection.zfar())
    }

    fn view_transform(&self) -> Isometry3<f32> {
        Isometry3::identity()
    }

    fn eye(&self) -> Point3<f32> {
        Point3::origin()
    }

    fn handle_event(&mut self, _: &Canvas, event: &WindowEvent) {
        match *event {
            WindowEvent::FramebufferSize(w, h) => {
                self.projection.set_aspect(w as f32 / h as f32);
                self.update_projviews();
            }
            _ => {}
        }
    }

    #[inline]
    fn upload(
        &self,
        _: usize,
        proj: &mut ShaderUniform<Matrix4<f32>>,
        view: &mut ShaderUniform<Matrix4<f32>>,
    ) {
        let view_mat = Matrix4::identity();
        proj.upload(&self.proj);
        view.upload(&view_mat);
    }

    fn transformation(&self) -> Matrix4<f32> {
        self.proj
    }

    fn inverse_transformation(&self) -> Matrix4<f32> {
        self.inv_proj
    }

    fn update(&mut self, _: &Canvas) {}
}

impl Camera2 for StaticCamera {
    fn handle_event(&mut self, canvas: &Canvas, event: &WindowEvent) {
        let hidpi = canvas.hidpi_factor();

        match *event {
            WindowEvent::FramebufferSize(w, h) => {
                let diag = Vector3::new(
                    2.0 * (hidpi as f32) / (w as f32),
                    2.0 * (hidpi as f32) / (h as f32),
                    1.0,
                );
                let inv_diag = Vector3::new(1.0 / diag.x, 1.0 / diag.y, 1.0);

                self.proj2 = Matrix3::from_diagonal(&diag);
                self.inv_proj2 = Matrix3::from_diagonal(&inv_diag);
            }
            _ => {}
        }
    }

    #[inline]
    fn upload(
        &self,
        proj: &mut ShaderUniform<Matrix3<f32>>,
        view: &mut ShaderUniform<Matrix3<f32>>,
    ) {
        let view_mat = Matrix3::identity();
        proj.upload(&self.proj2);
        view.upload(&view_mat);
    }

    fn update(&mut self, _: &Canvas) {}

    fn unproject(&self, window_coord: &Point2<f32>, size: &Vector2<f32>) -> Point2<f32> {
        let normalized_coords = Point2::new(
            2.0 * window_coord.x / size.x - 1.0,
            2.0 * -window_coord.y / size.y + 1.0,
        );

        let unprojected_hom = self.inv_proj2 * normalized_coords.to_homogeneous();
        Point2::from_homogeneous(unprojected_hom).unwrap()
    }
}
