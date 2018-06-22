use camera::Camera2;
use event::{Action, Key, MouseButton, WindowEvent};
use na::{self, Isometry3, Matrix3, Matrix4, Perspective3, Point2, Translation2, Vector2, Vector3};
use num::Pow;
use resource::ShaderUniform;
use std::f32;
use window::Canvas;

#[derive(Clone, Debug)]
pub struct PlanarCamera {
    at: Point2<f32>,
    /// Distance from the camera to the `at` focus point.
    zoom: f32,

    /// Increment of the zoomance per unit scrolling. The default value is 40.0.
    zoom_step: f32,
    drag_button: Option<MouseButton>,

    view: Matrix3<f32>,
    proj: Matrix3<f32>,
    scaled_proj: Matrix3<f32>,
    last_cursor_pos: Vector2<f32>,
}

impl PlanarCamera {
    /// Create a new arc-ball camera.
    pub fn new(at: Point2<f32>) -> PlanarCamera {
        PlanarCamera::new_with_frustrum(at)
    }

    /// Creates a new arc ball camera with default sensitivity values.
    pub fn new_with_frustrum(at: Point2<f32>) -> PlanarCamera {
        let mut res = PlanarCamera {
            at: Point2::origin(),
            zoom: 1.0,
            zoom_step: 0.9,
            drag_button: Some(MouseButton::Button2),
            view: na::zero(),
            proj: na::zero(),
            scaled_proj: na::zero(),
            last_cursor_pos: na::zero(),
        };

        res.look_at(at);

        res
    }

    /// The point the arc-ball is looking at.
    pub fn at(&self) -> Point2<f32> {
        self.at
    }

    /// Get a mutable reference to the point the camera is looking at.
    pub fn set_at(&mut self, at: Point2<f32>) {
        self.at = at;
        self.update_projviews();
    }

    /// Sets the zoom of the camera.
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom;

        self.update_restrictions();
        self.update_projviews();
    }

    /// Move the camera such that it is centered on a specific point.
    pub fn look_at(&mut self, at: Point2<f32>) {
        self.at = at;
        self.update_projviews();
    }

    /// Transformation applied by the camera without perspective.
    fn update_restrictions(&mut self) {
        if self.zoom < 0.00001 {
            self.zoom = 0.00001
        }
    }

    /// The button used to drag the PlanarCamera camera.
    pub fn drag_button(&self) -> Option<MouseButton> {
        self.drag_button
    }

    /// Set the button used to drag the PlanarCamera camera.
    /// Use None to disable dragging.
    pub fn rebind_drag_button(&mut self, new_button: Option<MouseButton>) {
        self.drag_button = new_button;
    }

    fn handle_right_button_displacement(&mut self, dpos: &Vector2<f32>) {
        self.at.x += dpos.x / self.zoom;
        self.at.y -= dpos.y / self.zoom;
        self.update_projviews();
    }

    fn handle_scroll(&mut self, off: f32) {
        self.zoom = self.zoom / self.zoom_step.pow(off / 120.0);
        self.update_restrictions();
        self.update_projviews();
    }

    fn update_projviews(&mut self) {
        self.view = Translation2::new(self.at.x, self.at.y).to_homogeneous();
        self.scaled_proj = self.proj;
        self.scaled_proj.m11 *= self.zoom;
        self.scaled_proj.m22 *= self.zoom;
    }
}

impl Camera2 for PlanarCamera {
    fn handle_event(&mut self, canvas: &Canvas, event: &WindowEvent) {
        let hidpi = 1.0; // canvas.hidpi_factor();

        match *event {
            WindowEvent::CursorPos(x, y, _) => {
                let curr_pos = Vector2::new(x as f32, y as f32);

                if let Some(drag_button) = self.drag_button {
                    if canvas.get_mouse_button(drag_button) == Action::Press {
                        let dpos = curr_pos - self.last_cursor_pos;
                        self.handle_right_button_displacement(&dpos)
                    }
                }

                self.last_cursor_pos = curr_pos;
            }
            WindowEvent::Scroll(_, off, _) => self.handle_scroll(off as f32),
            WindowEvent::FramebufferSize(w, h) => {
                self.proj = Matrix3::new(
                    2.0 * (hidpi as f32) / (w as f32),
                    0.0,
                    0.0,
                    0.0,
                    2.0 * (hidpi as f32) / (h as f32),
                    0.0,
                    0.0,
                    0.0,
                    1.0,
                );
                self.update_projviews();
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
        proj.upload(&self.scaled_proj);
        view.upload(&self.view);
    }

    fn update(&mut self, _: &Canvas) {}
}
