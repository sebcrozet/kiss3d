use std::f32;
use num::Float;
use glfw::{Key, Action};
use glfw;
use glfw::WindowEvent;
use na::{Pnt3, Vec2, Vec3, Mat4, Iso3, PerspMat3};
use na;
use camera::Camera;

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
pub struct ArcBall {
    /// The focus point.
    at:    Pnt3<f32>,
    /// Yaw of the camera (rotation along the y axis).
    yaw:   f32,
    /// Pitch of the camera (rotation along the x axis).
    pitch: f32,
    /// Distance from the camera to the `at` focus point.
    dist:  f32,

    /// Increment of the yaw per unit mouse movement. The default value is 0.005.
    yaw_step:   f32,
    /// Increment of the pitch per unit mouse movement. The default value is 0.005.
    pitch_step: f32,
    /// Increment of the distance per unit scrolling. The default value is 40.0.
    dist_step:  f32,

    projection:      PerspMat3<f32>,
    proj_view:       Mat4<f32>,
    inv_proj_view:   Mat4<f32>,
    last_cursor_pos: Vec2<f32>
}

impl ArcBall {
    /// Create a new arc-ball camera.
    pub fn new(eye: Pnt3<f32>, at: Pnt3<f32>) -> ArcBall {
        ArcBall::new_with_frustrum(f32::consts::PI / 4.0, 0.1, 1024.0, eye, at)
    }

    /// Creates a new arc ball camera with default sensitivity values.
    pub fn new_with_frustrum(fov:    f32,
                             znear:  f32,
                             zfar:   f32,
                             eye:    Pnt3<f32>,
                             at:     Pnt3<f32>) -> ArcBall {
        let mut res = ArcBall {
            at:              Pnt3::new(0.0, 0.0, 0.0),
            yaw:             0.0,
            pitch:           0.0,
            dist:            0.0,
            yaw_step:        0.005,
            pitch_step:      0.005,
            dist_step:       40.0,
            projection:      PerspMat3::new(800.0 / 600.0, fov, znear, zfar),
            proj_view:       na::zero(),
            inv_proj_view:   na::zero(),
            last_cursor_pos: na::zero()
        };

        res.look_at_z(eye, at);

        res
    }

    /// The point the arc-ball is looking at.
    pub fn at(&self) -> Pnt3<f32> {
        self.at
    }

    /// Get a mutable reference to the point the camera is looking at.
    pub fn set_at(&mut self, at: Pnt3<f32>) {
        self.at = at;
        self.update_projviews();
    }

    /// The arc-ball camera `yaw`.
    pub fn yaw(&self) -> f32 {
        self.yaw
    }

    /// Sets the camera `yaw`. Change this to modify the rotation along the `up` axis.
    pub fn set_yaw(&mut self, yaw: f32) {
        self.yaw = yaw;

        self.update_restrictions();
        self.update_projviews();
    }

    /// The arc-ball camera `pitch`.
    pub fn pitch(&self) -> f32 {
        self.pitch
    }

    /// Sets the camera `pitch`.
    pub fn set_pitch(&mut self, pitch: f32) {
        self.pitch = pitch;

        self.update_restrictions();
        self.update_projviews();
    }

    /// The distance from the camera position to its view point.
    pub fn dist(&self) -> f32 {
        self.dist
    }

    /// Move the camera such that it is at a given distance from the view point.
    pub fn set_dist(&mut self, dist: f32) {
        self.dist = dist;

        self.update_restrictions();
        self.update_projviews();
    }

    /// Move and orient the camera such that it looks at a specific point.
    pub fn look_at_z(&mut self, eye: Pnt3<f32>, at: Pnt3<f32>) {
        let dist  = na::norm(&(eye - at));
        let pitch = ((eye.y - at.y) / dist).acos();
        let yaw   = (eye.z - at.z).atan2(eye.x - at.x);

        self.at    = at;
        self.dist  = dist;
        self.yaw   = yaw;
        self.pitch = pitch;
        self.update_projviews();
    }

    /// Transformation applied by the camera without perspective.
    fn update_restrictions(&mut self) {
        if self.dist < 0.00001 {
            self.dist = 0.00001
        }

        if self.pitch <= 0.0001 {
            self.pitch = 0.0001
        }

        let _pi: f32 = f32::consts::PI;
        if self.pitch > _pi - 0.0001 {
            self.pitch = _pi - 0.0001
        }
    }

    fn handle_left_button_displacement(&mut self, dpos: &Vec2<f32>) {
        self.yaw   = self.yaw   + dpos.x * self.yaw_step;
        self.pitch = self.pitch - dpos.y * self.pitch_step;

        self.update_restrictions();
        self.update_projviews();
    }

    fn handle_right_button_displacement(&mut self, dpos: &Vec2<f32>) {
        let eye       = self.eye();
        let dir       = na::normalize(&(self.at - eye));
        let tangent   = na::normalize(&na::cross(&Vec3::y(), &dir));
        let bitangent = na::cross(&dir, &tangent);
        let mult      = self.dist / 1000.0;

        self.at = self.at + tangent * (dpos.x * mult) + bitangent * (dpos.y * mult);
        self.update_projviews();
    }

    fn handle_scroll(&mut self, off: f32) {
        self.dist = self.dist + self.dist_step * (off) / 120.0;
        self.update_restrictions();
        self.update_projviews();
    }

    fn update_projviews(&mut self) {
        self.proj_view     = *self.projection.as_mat() * na::to_homogeneous(&na::inv(&self.view_transform()).unwrap());
        self.inv_proj_view = na::inv(&self.proj_view).unwrap();
    }
}

impl Camera for ArcBall {
    fn clip_planes(&self) -> (f32, f32) {
        (self.projection.znear(), self.projection.zfar())
    }

    fn view_transform(&self) -> Iso3<f32> {
        let mut id: Iso3<f32> = na::one();
        id.look_at_z(&self.eye(), &self.at, &Vec3::y());

        id
    }

    fn eye(&self) -> Pnt3<f32> {
        let px = self.at.x + self.dist * self.yaw.cos() * self.pitch.sin();
        let py = self.at.y + self.dist * self.pitch.cos();
        let pz = self.at.z + self.dist * self.yaw.sin() * self.pitch.sin();

        Pnt3::new(px, py, pz)
    }

    fn handle_event(&mut self, window: &glfw::Window, event: &WindowEvent) {
        match *event {
            WindowEvent::CursorPos(x, y) => {
                let curr_pos = Vec2::new(x as f32, y as f32);

                if window.get_mouse_button(glfw::MouseButtonLeft) == Action::Press {
                    let dpos = curr_pos - self.last_cursor_pos;
                    self.handle_left_button_displacement(&dpos)
                }

                if window.get_mouse_button(glfw::MouseButtonRight) == Action::Press {
                    let dpos = curr_pos - self.last_cursor_pos;
                    self.handle_right_button_displacement(&dpos)
                }

                self.last_cursor_pos = curr_pos;
            },
            WindowEvent::Key(Key::Enter, _, Action::Press, _) => {
                self.at = na::orig();
                self.update_projviews();
            },
            WindowEvent::Scroll(_, off) => self.handle_scroll(off as f32),
            WindowEvent::FramebufferSize(w, h) => {
                self.projection.set_aspect(w as f32 / h as f32);
                self.update_projviews();
            },
            _ => { }
        }
    }

    fn transformation(&self) -> Mat4<f32> {
        self.proj_view
    }

    fn inv_transformation(&self) -> Mat4<f32> {
        self.inv_proj_view
    }

    fn update(&mut self, _: &glfw::Window) { }
}
