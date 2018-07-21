use camera::Camera;
use event::WindowEvent;
use na::{self, Isometry3, Matrix4, Perspective3, Point3};
use resource::ShaderUniform;
use std::f32;
use window::Canvas;

/// A camera that cannot move.
#[derive(Clone, Debug)]
pub struct FixedView {
    projection: Perspective3<f32>,
    proj: Matrix4<f32>,
    inv_proj: Matrix4<f32>,
}

impl FixedView {
    /// Create a new static camera.
    pub fn new() -> FixedView {
        FixedView::new_with_frustrum(f32::consts::PI / 4.0, 0.1, 1024.0)
    }

    /// Creates a new arc ball camera with default sensitivity values.
    pub fn new_with_frustrum(fov: f32, znear: f32, zfar: f32) -> FixedView {
        FixedView {
            projection: Perspective3::new(800.0 / 600.0, fov, znear, zfar),
            proj: na::one(),
            inv_proj: na::one(),
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

impl Camera for FixedView {
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
