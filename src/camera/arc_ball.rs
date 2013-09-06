use std::num::{Zero, One, atan2};
use glfw;
use glfw::consts::*;
use nalgebra::vec::{Vec2, Vec3, AlgebraicVec, Cross};
use nalgebra::mat::{Mat4, Inv, ToHomogeneous};
use nalgebra::types::Iso3f64;
use camera::Camera;
use event;

/// Arc-ball camera mode. An arc-ball camera is a camera rotating around a fixed point (the focus
/// point) and always looking at it. The following inputs are handled:
///
///   * Left button press + drag - rotates the camera around the focus point
///   * Right button press + drag - translates the focus point on the plane orthogonal to the view
///   direction
///   * Scroll in/out - zoom in/out
///   * Enter key - set the focus point to the origin
#[deriving(Clone, ToStr)]
pub struct ArcBall {
    /// The focus point.
    priv at:    Vec3<f64>,
    /// Yaw of the camera (rotation along the y axis).
    priv yaw:   f64,
    /// Pitch of the camera (rotation along the x axis).
    priv pitch: f64,
    /// Distance from the camera to the `at` focus point.
    priv dist:  f64,

    /// Increment of the yaw per unit mouse movement. The default value is 0.005.
    priv yaw_step:   f64,
    /// Increment of the pitch per unit mouse movement. The default value is 0.005.
    priv pitch_step: f64,
    /// Increment of the distance per unit scrolling. The default value is 40.0.
    priv dist_step:  f64,

    priv fov:        f64,
    priv znear:      f64,
    priv zfar:       f64,
    priv projection:      Mat4<f64>,
    priv proj_view:       Mat4<f64>,
    priv inv_proj_view:   Mat4<f64>,
    priv last_cursor_pos: Vec2<f64>
}

impl ArcBall {
    /// Create a new arc-ball camera.
    pub fn new(eye: Vec3<f64>, at: Vec3<f64>) -> ArcBall {
        ArcBall::new_with_frustrum(45.0f64.to_radians(), 0.1, 1024.0, eye, at)
    }

    /// Creates a new arc ball camera with default sensitivity values.
    pub fn new_with_frustrum(fov:    f64,
                             znear:  f64,
                             zfar:   f64,
                             eye:    Vec3<f64>,
                             at:     Vec3<f64>) -> ArcBall {
        let mut res = ArcBall {
            at:         Vec3::new(0.0, 0.0, 0.0),
            yaw:        0.0,
            pitch:      0.0,
            dist:       0.0,
            yaw_step:   0.005,
            pitch_step: 0.005,
            dist_step:  40.0,
            fov:        fov,
            znear:      znear,
            zfar:       zfar,
            projection: Mat4::new_perspective(800.0, 600.0, fov, znear, zfar),
            proj_view:  Zero::zero(),
            inv_proj_view:   Zero::zero(),
            last_cursor_pos: Zero::zero()
        };

        res.look_at_z(eye, at);

        res
    }

    /// The point the arc-ball is looking at.
    pub fn at(&self) -> Vec3<f64> {
        self.at
    }

    /// The arc-ball camera `yaw`.
    pub fn yaw(&self) -> f64 {
        self.yaw
    }

    /// Sets the camera `yaw`. Change this to modify the rotation along the `up` axis.
    pub fn set_yaw(&mut self, yaw: f64) {
        self.yaw = yaw;

        self.update_restrictions();
        self.update_projviews();
    }

    /// The arc-ball camera `pitch`.
    pub fn pitch(&self) -> f64 {
        self.pitch
    }

    /// Sets the camera `pitch`.
    pub fn set_pitch(&mut self, pitch: f64) {
        self.pitch = pitch;

        self.update_restrictions();
        self.update_projviews();
    }

    /// The distance from the camera position to its view point.
    pub fn dist(&self) -> f64 {
        self.dist
    }

    /// Move the camera such that it is at a given distance from the view point.
    pub fn set_dist(&mut self, dist: f64) {
        self.dist = dist;

        self.update_restrictions();
        self.update_projviews();
    }

    /// Move and orient the camera such that it looks at a specific point.
    pub fn look_at_z(&mut self, eye: Vec3<f64>, at: Vec3<f64>) {
        let dist  = (eye - at).norm();
        let pitch = ((eye.y - at.y) / dist).acos();
        let yaw   = atan2(eye.z - at.z, eye.x - at.x);

        self.at    = at;
        self.dist  = dist;
        self.yaw   = yaw;
        self.pitch = pitch;
        self.update_projviews();
    }

    /// Transformation applied by the camera without perspective.
    fn update_restrictions(&mut self) {
        if (self.dist < 0.00001) {
            self.dist = 0.00001
        }

        if (self.pitch <= 0.0001) {
            self.pitch = 0.0001
        }

        let _pi: f64 = Real::pi();
        if (self.pitch > _pi - 0.0001) {
            self.pitch = _pi - 0.0001
        }
    }

    fn handle_left_button_displacement(&mut self, dpos: &Vec2<f64>) {
        self.yaw   = self.yaw   + dpos.x * self.yaw_step;
        self.pitch = self.pitch - dpos.y * self.pitch_step;

        self.update_restrictions();
        self.update_projviews();
    }

    fn handle_right_button_displacement(&mut self, dpos: &Vec2<f64>) {
        let eye       = self.eye();
        let dir       = (self.at - eye).normalized();
        let tangent   = Vec3::y().cross(&dir).normalized();
        let bitangent = dir.cross(&tangent);
        let mult      = self.dist / 1000.0;

        self.at = self.at + tangent * (dpos.x * mult) + bitangent * (dpos.y * mult);
        self.update_projviews();
    }

    fn handle_scroll(&mut self, off: float) {
        self.dist = self.dist + self.dist_step * (off as f64) / 120.0;
        self.update_restrictions();
        self.update_projviews();
    }

    fn update_projviews(&mut self) {
        self.proj_view = self.projection * self.view_transform().inverse().unwrap().to_homogeneous();
        self.inv_proj_view = self.proj_view.inverse().unwrap();
    }
}

impl Camera for ArcBall {
    fn clip_planes(&self) -> (f64, f64) {
        (self.znear, self.zfar)
    }

    fn view_transform(&self) -> Iso3f64 {
        let mut id: Iso3f64 = One::one();
        id.look_at_z(&self.eye(), &self.at, &Vec3::y());

        id
    }

    fn eye(&self) -> Vec3<f64> {
        let px = self.at.x + self.dist * self.yaw.cos() * self.pitch.sin();
        let py = self.at.y + self.dist * self.pitch.cos();
        let pz = self.at.z + self.dist * self.yaw.sin() * self.pitch.sin();

        Vec3::new(px, py, pz)
    }

    fn handle_mouse(&mut self, window: &glfw::Window, event: &event::MouseEvent) {
        match *event {
            event::CursorPos(x, y) => {
                let curr_pos = Vec2::new(x as f64, y as f64);

                if window.get_mouse_button(MOUSE_BUTTON_1) == PRESS {
                    let dpos = curr_pos - self.last_cursor_pos;
                    self.handle_left_button_displacement(&dpos)
                }

                if window.get_mouse_button(MOUSE_BUTTON_2) == PRESS {
                    let dpos = curr_pos - self.last_cursor_pos;
                    self.handle_right_button_displacement(&dpos)
                }

                self.last_cursor_pos = curr_pos;
            },
            event::Scroll(_, off)  => self.handle_scroll(off),
            _ => { }
        }
    }

    fn handle_keyboard(&mut self, _: &glfw::Window, event: &event::KeyboardEvent) {
        match *event {
            event::KeyReleased(button) => if button == KEY_ENTER {
                self.at = Zero::zero();
                self.update_projviews();
            },
            _ => { }
        }
    }

    fn handle_framebuffer_size_change(&mut self, w: f64, h: f64) {
        self.projection = Mat4::new_perspective(w, h, self.fov, self.znear, self.zfar);
        self.update_projviews();
    }

    fn transformation(&self) -> Mat4<f64> {
        self.proj_view
    }

    fn inv_transformation(&self) -> Mat4<f64> {
        self.inv_proj_view
    }

    fn update(&mut self, _: &glfw::Window) { }
}
