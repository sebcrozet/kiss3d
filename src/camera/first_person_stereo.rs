use std::num::{Zero, One, atan2};
use glfw;
use gl;
use nalgebra::na::{Vec2, Vec3, Norm, Cross};
use nalgebra::na::{Mat4, Rotate};
use nalgebra::na::Iso3;
use nalgebra::na;
use camera::Camera;
use event;

#[path = "../error.rs"]
mod error;

/// First-person camera mode.
///
///   * Left button press + drag - look around
///   * Right button press + drag - translates the camera position on the plane orthogonal to the view
///   direction
///   * Scroll in/out - zoom in/out
#[deriving(ToStr)]
pub struct FirstPersonStereo {
    /// The camera position
    priv eye:        Vec3<f64>,
    priv eye_left:   Vec3<f64>,
    priv eye_right:  Vec3<f64>,

    /// Inter Pupilary Distance
    priv ipd:        f64,

    /// Yaw of the camera (rotation along the y axis).
    priv yaw:        f64,
    /// Pitch of the camera (rotation along the x axis).
    priv pitch:      f64,

    /// Increment of the yaw per unit mouse movement. The default value is 0.005.
    priv yaw_step:   f64,
    /// Increment of the pitch per unit mouse movement. The default value is 0.005.
    priv pitch_step: f64,
    /// Increment of the translation per arrow press. The default value is 0.1.
    priv move_step:  f64,

    /// Low level datas
    priv fov:        f64,
    priv znear:      f64,
    priv zfar:       f64,
    priv projection:      Mat4<f64>,
    priv proj_view:       Mat4<f64>,
    priv proj_view_left:  Mat4<f64>,
    priv proj_view_right: Mat4<f64>,
    priv inv_proj_view:   Mat4<f64>,
    priv last_cursor_pos: Vec2<f64>
}

impl FirstPersonStereo {
    /// Creates a first person camera with default sensitivity values.
    pub fn new(eye: Vec3<f64>, at: Vec3<f64>, ipd: f64) -> FirstPersonStereo {
        FirstPersonStereo::new_with_frustrum(45.0f64.to_radians(), 0.1, 1024.0, eye, at, ipd)
    }

    /// Creates a new first person camera with default sensitivity values.
    pub fn new_with_frustrum(fov:    f64,
                             znear:  f64,
                             zfar:   f64,
                             eye:    Vec3<f64>,
                             at:     Vec3<f64>,
                             ipd:    f64) -> FirstPersonStereo {
        let mut res = FirstPersonStereo {
            eye:           Vec3::new(0.0, 0.0, 0.0),
            // left & right are initially wrong, don't take ipd into accound
            eye_left:      Vec3::new(0.0, 0.0, 0.0),
            eye_right:     Vec3::new(0.0, 0.0, 0.0),
            ipd:           ipd,
            yaw:           0.0,
            pitch:         0.0,
            yaw_step:      0.005,
            pitch_step:    0.005,
            move_step:     0.5,
            fov:        fov,
            znear:      znear,
            zfar:       zfar,
            projection: Mat4::new_perspective(800.0, 600.0, fov, znear, zfar),
            proj_view:  Zero::zero(),
            inv_proj_view:   Zero::zero(),
            last_cursor_pos: Zero::zero(),
            proj_view_left: Zero::zero(),
            proj_view_right: Zero::zero(),
        };

        res.look_at_z(eye, at);

        res
    }


    /// Changes the orientation and position of the camera to look at the specified point.
    pub fn look_at_z(&mut self, eye: Vec3<f64>, at: Vec3<f64>) {
        let dist  = (eye - at).norm();

        let pitch = ((at.y - eye.y) / dist).acos();
        let yaw   = atan2(at.z - eye.z, at.x - eye.x);

        self.eye   = eye;
        self.yaw   = yaw;
        self.pitch = pitch;
        self.update_projviews();
    }

    /// The point the camera is looking at.
    pub fn at(&self) -> Vec3<f64> {
        let ax = self.eye.x + self.yaw.cos() * self.pitch.sin();
        let ay = self.eye.y + self.pitch.cos();
        let az = self.eye.z + self.yaw.sin() * self.pitch.sin();

        Vec3::new(ax, ay, az)
    }

    fn update_restrictions(&mut self) {
        if (self.pitch <= 0.0001) {
            self.pitch = 0.0001
        }

        let _pi: f64 = Real::pi();
        if (self.pitch > _pi - 0.0001) {
            self.pitch = _pi - 0.0001
        }
    }

    #[doc(hidden)]
    pub fn handle_left_button_displacement(&mut self, dpos: &Vec2<f64>) {
        self.yaw   = self.yaw   + dpos.x * self.yaw_step;
        self.pitch = self.pitch + dpos.y * self.pitch_step;

        self.update_restrictions();
        self.update_projviews();
    }

    fn update_eyes_location(&mut self) {
        // left and right are on a line perpendicular to both up and the target
        // up is always y
        let dir       = (self.at() - self.eye).normalized();
        let tangent   = Vec3::y().cross(&dir).normalized();
        self.eye_left = self.eye - tangent * (self.ipd / 2.0 as f64);
        self.eye_right = self.eye + tangent * (self.ipd / 2.0 as f64);
        //println(fmt!("eye_left = %f,%f,%f", self.eye_left.x as float, self.eye_left.y as float, self.eye_left.z as float));
        //println(fmt!("eye_right = %f,%f,%f", self.eye_right.x as float, self.eye_right.y as float, self.eye_right.z as float));
        // TODO: verify with an assert or something that the distance between the eyes is ipd, just to make me feel good.
    }

    #[doc(hidden)]
    pub fn handle_right_button_displacement(&mut self, dpos: &Vec2<f64>) {
        let at        = self.at();
        let dir       = (at - self.eye).normalized();
        let tangent   = Vec3::y().cross(&dir).normalized();
        let bitangent = dir.cross(&tangent);

        self.eye = self.eye + tangent * (0.01 * dpos.x as f64 / 10.0) + bitangent * (0.01 * dpos.y as f64 / 10.0);
        // TODO: ugly - should move eye update to where eye_left & eye_right are updated
        self.update_eyes_location();
        self.update_restrictions();
        self.update_projviews();
    }

    #[doc(hidden)]
    pub fn handle_scroll(&mut self, yoff: f64) {
        let front: Vec3<f64> = self.view_transform().rotate(&Vec3::z());

        self.eye = self.eye + front * (self.move_step * yoff);

        self.update_eyes_location();
        self.update_restrictions();
        self.update_projviews();
    }

    fn update_projviews(&mut self) {
        self.proj_view = self.projection * na::to_homogeneous(&na::inverted(&self.view_transform()).unwrap());
        self.inv_proj_view = na::inverted(&self.proj_view).unwrap();
        self.proj_view_left = self.projection * na::to_homogeneous(&na::inverted(&self.view_transform_left()).unwrap());
        self.proj_view_right = self.projection * na::to_homogeneous(&na::inverted(&self.view_transform_right()).unwrap());
    }

    fn transformation_eye(&self, eye: uint) -> Mat4<f64> {
        match eye {
            0u => self.proj_view_left,
            1u => self.proj_view_right,
            _ => fail!("bad eye index")
        }
    }

    /// The left eye camera view transformation
    fn view_transform_left(&self) -> Iso3<f64> {
        let mut id: Iso3<f64> = One::one();
        id.look_at_z(&self.eye_left, &self.at(), &Vec3::y());

        id
    }

    /// The right eye camera view transformation
    fn view_transform_right(&self) -> Iso3<f64> {
        let mut id: Iso3<f64> = One::one();
        id.look_at_z(&self.eye_right, &self.at(), &Vec3::y());

        id
    }

    /// return Inter Pupilary Distance
    pub fn ipd(&self) -> f64 {
        self.ipd
    }
    
    /// change Inter Pupilary Distance
    pub fn set_ipd(&mut self, ipd: f64) {
        self.ipd = ipd;

        self.update_eyes_location();
        self.update_restrictions();
        self.update_projviews();
    }

}

impl Camera for FirstPersonStereo {
    fn clip_planes(&self) -> (f64, f64) {
        (self.znear, self.zfar)
    }

    /// The imaginary middle eye camera view transformation (i-e transformation without projection).
    fn view_transform(&self) -> Iso3<f64> {
        let mut id: Iso3<f64> = One::one();
        id.look_at_z(&self.eye, &self.at(), &Vec3::y());

        id
    }

    fn handle_event(&mut self, window: &glfw::Window, event: &event::Event) {
        match *event {
            event::CursorPos(x, y) => {
                let curr_pos = Vec2::new(x as f64, y as f64);

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
                self.projection = Mat4::new_perspective(w, h, self.fov, self.znear, self.zfar);
                self.update_projviews();
            }
            _ => { }
        }
    }

    fn eye(&self) -> Vec3<f64> {
        self.eye
    }

    fn transformation(&self) -> Mat4<f64> {
        self.proj_view
    }

    fn inv_transformation(&self) -> Mat4<f64> {
        self.inv_proj_view
    }

    fn update(&mut self, window: &glfw::Window) {
        let t                = self.view_transform();
        let front: Vec3<f64> = t.rotate(&Vec3::z());
        let right: Vec3<f64> = t.rotate(&Vec3::x());

        if window.get_key(glfw::KeyUp) == glfw::Press {
            self.eye = self.eye + front * self.move_step
        }

        if window.get_key(glfw::KeyDown) == glfw::Press {
            self.eye = self.eye + front * (-self.move_step)
        }

        if window.get_key(glfw::KeyRight) == glfw::Press {
            self.eye = self.eye + right * (-self.move_step)
        }

        if window.get_key(glfw::KeyLeft) == glfw::Press {
            self.eye = self.eye + right * self.move_step
        }

        self.update_eyes_location();
        self.update_restrictions();
        self.update_projviews();
    }

    fn upload(&self, pass: uint, view_location: i32) {
        self.upload_mat(view_location, &self.transformation_eye(pass));
    }

    fn num_passes(&self) -> uint { 2u }

    fn start_pass(&self, pass: uint, window: &glfw::Window) {
        let (win_w, win_h) = window.get_size();
        let (x, y, w, h) = match pass {
            0u => (0, 0, win_w / 2 , win_h),
            1u => (win_w / 2, 0, win_w / 2, win_h),
            _ => fail!("stereo first person takes only two passes")
        };
        verify!(gl::Viewport(x as i32, y as i32, w as i32, h as i32));
        verify!(gl::Scissor(x as i32, y as i32, w as i32, h as i32));
    }

    fn render_complete(&self, window: &glfw::Window) {
        let (w, h) = window.get_size();
        verify!(gl::Viewport(0, 0, w as i32, h as i32));
        verify!(gl::Scissor(0, 0, w as i32, h as i32));
    }
}
