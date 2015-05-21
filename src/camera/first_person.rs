use std::f32;
use num::Float;
use glfw;
use glfw::{Key, Action, WindowEvent};
use na::{Translation, Pnt3, Vec2, Vec3, Mat4, Iso3, PerspMat3};
use na;
use camera::Camera;

/// First-person camera mode.
///
///   * Left button press + drag - look around
///   * Right button press + drag - translates the camera position on the plane orthogonal to the
///   view direction
///   * Scroll in/out - zoom in/out
#[derive(Debug, Clone)]
pub struct FirstPerson {
    eye:             Pnt3<f32>,
    yaw:             f32,
    pitch:           f32,

    yaw_step:        f32,
    pitch_step:      f32,
    move_step:       f32,

    projection:      PerspMat3<f32>,
    proj_view:       Mat4<f32>,
    inv_proj_view:   Mat4<f32>,
    last_cursor_pos: Vec2<f32>
}

impl FirstPerson {
    /// Creates a first person camera with default sensitivity values.
    pub fn new(eye: Pnt3<f32>, at: Pnt3<f32>) -> FirstPerson {
        FirstPerson::new_with_frustrum(f32::consts::PI / 4.0, 0.1, 1024.0, eye, at)
    }

    /// Creates a new first person camera with default sensitivity values.
    pub fn new_with_frustrum(fov:    f32,
                             znear:  f32,
                             zfar:   f32,
                             eye:    Pnt3<f32>,
                             at:     Pnt3<f32>) -> FirstPerson {
        let mut res = FirstPerson {
            eye:             Pnt3::new(0.0, 0.0, 0.0),
            yaw:             0.0,
            pitch:           0.0,
            yaw_step:        0.005,
            pitch_step:      0.005,
            move_step:       0.5,
            projection:      PerspMat3::new(800.0 / 600.0, fov, znear, zfar),
            proj_view:       na::zero(),
            inv_proj_view:   na::zero(),
            last_cursor_pos: na::zero(),
        };

        res.look_at_z(eye, at);

        res
    }

    /// Sets the translational increment per arrow press.
    ///
    /// The default value is 0.5.
    #[inline]
    pub fn set_move_step(&mut self, step: f32) {
        self.move_step = step;
    }

    /// Sets the pitch increment per mouse movement.
    ///
    /// The default value is 0.005.
    #[inline]
    pub fn set_pitch_step(&mut self, step: f32) {
        self.pitch_step = step;
    }


    /// Sets the yaw increment per mouse movement.
    ///
    /// The default value is 0.005.
    #[inline]
    pub fn set_yaw_step(&mut self, step: f32) {
        self.yaw_step = step;
    }

    /// Gets the translational increment per arrow press.
    #[inline]
    pub fn move_step(&self) -> f32 {
        self.move_step
    }

    /// Gets the pitch increment per mouse movement.
    #[inline]
    pub fn pitch_step(&self) -> f32 {
        self.pitch_step
    }

    /// Gets the yaw  increment per mouse movement.
    #[inline]
    pub fn yaw_step(&self) -> f32 {
        self.yaw_step
    }

    /// Changes the orientation and position of the camera to look at the specified point.
    pub fn look_at_z(&mut self, eye: Pnt3<f32>, at: Pnt3<f32>) {
        let dist  = na::norm(&(eye - at));

        let pitch = ((at.y - eye.y) / dist).acos();
        let yaw   = (at.z - eye.z).atan2(at.x - eye.x);

        self.eye   = eye;
        self.yaw   = yaw;
        self.pitch = pitch;
        self.update_projviews();
    }

    /// The point the camera is looking at.
    pub fn at(&self) -> Pnt3<f32> {
        let ax = self.eye.x + self.yaw.cos() * self.pitch.sin();
        let ay = self.eye.y + self.pitch.cos();
        let az = self.eye.z + self.yaw.sin() * self.pitch.sin();

        Pnt3::new(ax, ay, az)
    }

    fn update_restrictions(&mut self) {
        if self.pitch <= 0.01 {
            self.pitch = 0.01
        }

        let _pi: f32 = f32::consts::PI;
        if self.pitch > _pi - 0.01 {
            self.pitch = _pi - 0.01
        }
    }

    #[doc(hidden)]
    pub fn handle_left_button_displacement(&mut self, dpos: &Vec2<f32>) {
        self.yaw   = self.yaw   + dpos.x * self.yaw_step;
        self.pitch = self.pitch + dpos.y * self.pitch_step;

        self.update_restrictions();
        self.update_projviews();
    }

    #[doc(hidden)]
    pub fn handle_right_button_displacement(&mut self, dpos: &Vec2<f32>) {
        let at        = self.at();
        let dir       = na::normalize(&(at - self.eye));
        let tangent   = na::normalize(&na::cross(&Vec3::y(), &dir));
        let bitangent = na::cross(&dir, &tangent);

        self.eye = self.eye + tangent * (0.01 * dpos.x / 10.0) + bitangent * (0.01 * dpos.y / 10.0);
        self.update_restrictions();
        self.update_projviews();
    }

    #[doc(hidden)]
    pub fn handle_scroll(&mut self, yoff: f32) {
        let front: Vec3<f32> = na::rotate(&self.view_transform(), &Vec3::z());

        self.eye = self.eye + front * (self.move_step * yoff);

        self.update_restrictions();
        self.update_projviews();
    }

    fn update_projviews(&mut self) {
        let _ = na::inv(&self.view_transform()).map(|inv_view|
            self.proj_view = *self.projection.as_mat() * na::to_homogeneous(&inv_view)
        );

        let _ = na::inv(&self.proj_view).map(|inv_proj| self.inv_proj_view = inv_proj);
    }

    /// The direction this camera is looking at.
    pub fn eye_dir(&self) -> Vec3<f32> {
        na::normalize(&(self.at() - self.eye))
    }

    /// The direction this camera is being moved by the keyboard keys for a given set of key states.
    pub fn move_dir(&self, up: bool, down: bool, right: bool, left: bool) -> Vec3<f32> {
        let t                = self.view_transform();
        let frontv: Vec3<f32> = na::rotate(&t, &Vec3::z());
        let rightv: Vec3<f32> = na::rotate(&t, &Vec3::x());

        let mut movement = na::zero::<Vec3<f32>>();

        if up {
            movement = movement + frontv
        }

        if down {
            movement = movement - frontv
        }

        if right {
            movement = movement - rightv
        }

        if left {
            movement =  movement + rightv
        }

        if na::is_zero(&movement) {
            movement
        }
        else {
            na::normalize(&movement)
        }
    }
}

impl Camera for FirstPerson {
    fn clip_planes(&self) -> (f32, f32) {
        (self.projection.znear(), self.projection.zfar())
    }

    /// The camera view transformation (i-e transformation without projection).
    fn view_transform(&self) -> Iso3<f32> {
        let mut id: Iso3<f32> = na::one();
        id.look_at_z(&self.eye, &self.at(), &Vec3::y());

        id
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
            WindowEvent::Scroll(_, off) => self.handle_scroll(off as f32),
            WindowEvent::FramebufferSize(w, h) => {
                self.projection.set_aspect(w as f32 / h as f32);
                self.update_projviews();
            }
            _ => { }
        }
    }

    fn eye(&self) -> Pnt3<f32> {
        self.eye
    }

    fn transformation(&self) -> Mat4<f32> {
        self.proj_view
    }

    fn inv_transformation(&self) -> Mat4<f32> {
        self.inv_proj_view
    }

    fn update(&mut self, window: &glfw::Window) {
        let up    = window.get_key(Key::Up)    == Action::Press;
        let down  = window.get_key(Key::Down)  == Action::Press;
        let right = window.get_key(Key::Right) == Action::Press;
        let left  = window.get_key(Key::Left)  == Action::Press;
        let dir   = self.move_dir(up, down, right, left);

        let move_amount  = dir * self.move_step;
        self.append_translation_mut(&move_amount);
    }
}

impl Translation<Vec3<f32>> for FirstPerson {
    #[inline]
    fn translation(&self) -> Vec3<f32> {
        self.eye.as_vec().clone()
    }

    #[inline]
    fn inv_translation(&self) -> Vec3<f32> {
        -self.eye.as_vec().clone()
    }

    #[inline]
    fn append_translation_mut(&mut self, t: &Vec3<f32>) {
        let new_t = self.eye + *t;

        self.set_translation(new_t.to_vec());
    }

    #[inline]
    fn append_translation(&self, t: &Vec3<f32>) -> FirstPerson {
        let mut res = self.clone();

        res.append_translation_mut(t);

        res
    }

    #[inline]
    fn prepend_translation_mut(&mut self, t: &Vec3<f32>) {
        let new_t = self.eye - *t;

        self.set_translation(new_t.to_vec()); // FIXME: is this correct?
    }

    #[inline]
    fn prepend_translation(&self, t: &Vec3<f32>) -> FirstPerson {
        let mut res = self.clone();

        res.prepend_translation_mut(t);

        res
    }

    #[inline]
    fn set_translation(&mut self, t: Vec3<f32>) {
        self.eye = na::orig::<Pnt3<f32>>() + t;
        self.update_restrictions();
        self.update_projviews();
    }
}
