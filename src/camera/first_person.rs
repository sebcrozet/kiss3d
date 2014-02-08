use std::num::{Zero, atan2};
use glfw;
use nalgebra::na::{Translation, Vec2, Vec3, Mat4, Iso3};
use nalgebra::na;
use camera::Camera;
use event;

/// First-person camera mode.
///
///   * Left button press + drag - look around
///   * Right button press + drag - translates the camera position on the plane orthogonal to the
///   view direction
///   * Scroll in/out - zoom in/out
#[deriving(ToStr, Clone)]
pub struct FirstPerson {
    priv eye:             Vec3<f32>,
    priv yaw:             f32,
    priv pitch:           f32,

    priv yaw_step:        f32,
    priv pitch_step:      f32,
    priv move_step:       f32,

    priv fov:             f32,
    priv znear:           f32,
    priv zfar:            f32,
    priv projection:      Mat4<f32>,
    priv proj_view:       Mat4<f32>,
    priv inv_proj_view:   Mat4<f32>,
    priv last_cursor_pos: Vec2<f32>
}

impl FirstPerson {
    /// Creates a first person camera with default sensitivity values.
    pub fn new(eye: Vec3<f32>, at: Vec3<f32>) -> FirstPerson {
        FirstPerson::new_with_frustrum(45.0f32.to_radians(), 0.1, 1024.0, eye, at)
    }

    /// Creates a new first person camera with default sensitivity values.
    pub fn new_with_frustrum(fov:    f32,
                             znear:  f32,
                             zfar:   f32,
                             eye:    Vec3<f32>,
                             at:     Vec3<f32>) -> FirstPerson {
        let mut res = FirstPerson {
            eye:             Vec3::new(0.0, 0.0, 0.0),
            yaw:             0.0,
            pitch:           0.0,
            yaw_step:        0.005,
            pitch_step:      0.005,
            move_step:       0.5,
            fov:             fov,
            znear:           znear,
            zfar:            zfar,
            projection:      na::perspective3d(800.0, 600.0, fov, znear, zfar),
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
    pub fn look_at_z(&mut self, eye: Vec3<f32>, at: Vec3<f32>) {
        let dist  = na::norm(&(eye - at));

        let pitch = ((at.y - eye.y) / dist).acos();
        let yaw   = atan2(at.z - eye.z, at.x - eye.x);

        self.eye   = eye;
        self.yaw   = yaw;
        self.pitch = pitch;
        self.update_projviews();
    }

    /// The point the camera is looking at.
    pub fn at(&self) -> Vec3<f32> {
        let ax = self.eye.x + self.yaw.cos() * self.pitch.sin();
        let ay = self.eye.y + self.pitch.cos();
        let az = self.eye.z + self.yaw.sin() * self.pitch.sin();

        Vec3::new(ax, ay, az)
    }

    fn update_restrictions(&mut self) {
        if self.pitch <= 0.01 {
            self.pitch = 0.01
        }

        let _pi: f32 = Real::pi();
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
            self.proj_view = self.projection * na::to_homogeneous(&inv_view)
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

        if movement.is_zero() {
            movement
        }
        else {
            na::normalize(&movement)
        }
    }
}

impl Camera for FirstPerson {
    fn clip_planes(&self) -> (f32, f32) {
        (self.znear, self.zfar)
    }

    /// The camera view transformation (i-e transformation without projection).
    fn view_transform(&self) -> Iso3<f32> {
        let mut id: Iso3<f32> = na::one();
        id.look_at_z(&self.eye, &self.at(), &Vec3::y());

        id
    }

    fn handle_event(&mut self, window: &glfw::Window, event: &event::Event) {
        match *event {
            event::CursorPos(x, y) => {
                let curr_pos = Vec2::new(x, y);

                if window.get_mouse_button(glfw::MouseButtonLeft) == glfw::Press {
                    let dpos = curr_pos - self.last_cursor_pos;
                    self.handle_left_button_displacement(&dpos)
                }

                if window.get_mouse_button(glfw::MouseButtonRight) == glfw::Press {
                    let dpos = curr_pos - self.last_cursor_pos;
                    self.handle_right_button_displacement(&dpos)
                }

                self.last_cursor_pos = curr_pos;
            },
            event::Scroll(_, off) => self.handle_scroll(off),
            event::FramebufferSize(w, h) => {
                self.projection = na::perspective3d(w, h, self.fov, self.znear, self.zfar);
                self.update_projviews();
            }
            _ => { }
        }
    }

    fn eye(&self) -> Vec3<f32> {
        self.eye
    }

    fn transformation(&self) -> Mat4<f32> {
        self.proj_view
    }

    fn inv_transformation(&self) -> Mat4<f32> {
        self.inv_proj_view
    }

    fn update(&mut self, window: &glfw::Window) {
        let up    = window.get_key(glfw::KeyUp)    == glfw::Press;
        let down  = window.get_key(glfw::KeyDown)  == glfw::Press;
        let right = window.get_key(glfw::KeyRight) == glfw::Press;
        let left  = window.get_key(glfw::KeyLeft)  == glfw::Press;
        let dir   = self.move_dir(up, down, right, left);

        let move  = dir * self.move_step;
        self.append_translation(&move);
    }
}

impl Translation<Vec3<f32>> for FirstPerson {
    #[inline]
    fn translation(&self) -> Vec3<f32> {
        self.eye
    }

    #[inline]
    fn inv_translation(&self) -> Vec3<f32> {
        -self.eye
    }

    #[inline]
    fn append_translation(&mut self, t: &Vec3<f32>) {
        let new_t = self.eye + *t;

        self.set_translation(new_t);
    }

    #[inline]
    fn append_translation_cpy(me: &FirstPerson, t: &Vec3<f32>) -> FirstPerson {
        let mut res = me.clone();

        res.append_translation(t);

        res
    }

    #[inline]
    fn prepend_translation(&mut self, t: &Vec3<f32>) {
        let new_t = self.eye - *t;

        self.set_translation(new_t); // FIXME: is this correct?
    }

    #[inline]
    fn prepend_translation_cpy(me: &FirstPerson, t: &Vec3<f32>) -> FirstPerson {
        let mut res = me.clone();

        res.prepend_translation(t);

        res
    }

    #[inline]
    fn set_translation(&mut self, t: Vec3<f32>) {
        self.eye = t;
        self.update_restrictions();
        self.update_projviews();
    }
}
