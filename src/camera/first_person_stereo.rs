use std::f32;
use num::Float;
use glfw;
use glfw::{Key, Action, WindowEvent};
use gl;
use na::{Pnt3, Pnt2, Vec2, Vec3, Mat4, Iso3, PerspMat3, Rotate};
use na;
use resource::ShaderUniform;
use camera::Camera;

#[path = "../error.rs"]
mod error;

/// First-person camera mode.
///
///   * Left button press + drag - look around
///   * Right button press + drag - translates the camera position on the plane orthogonal to the
///   view direction
///   * Scroll in/out - zoom in/out
#[derive(Debug)]
pub struct FirstPersonStereo {
    /// The camera position
    eye:        Pnt3<f32>,
    eye_left:   Pnt3<f32>,
    eye_right:  Pnt3<f32>,

    /// Inter Pupilary Distance
    ipd:        f32,

    /// Yaw of the camera (rotation along the y axis).
    yaw:        f32,
    /// Pitch of the camera (rotation along the x axis).
    pitch:      f32,

    /// Increment of the yaw per unit mouse movement. The default value is 0.005.
    yaw_step:   f32,
    /// Increment of the pitch per unit mouse movement. The default value is 0.005.
    pitch_step: f32,
    /// Increment of the translation per arrow press. The default value is 0.1.
    move_step:  f32,

    /// Low level datas
    projection:      PerspMat3<f32>,
    proj_view:       Mat4<f32>,
    proj_view_left:  Mat4<f32>,
    proj_view_right: Mat4<f32>,
    inv_proj_view:   Mat4<f32>,
    last_cursor_pos: Pnt2<f32>
}

impl FirstPersonStereo {
    /// Creates a first person camera with default sensitivity values.
    pub fn new(eye: Pnt3<f32>, at: Pnt3<f32>, ipd: f32) -> FirstPersonStereo {
        FirstPersonStereo::new_with_frustrum(f32::consts::PI / 4.0, 0.1, 1024.0, eye, at, ipd)
    }

    /// Creates a new first person camera with default sensitivity values.
    pub fn new_with_frustrum(fov:    f32,
                             znear:  f32,
                             zfar:   f32,
                             eye:    Pnt3<f32>,
                             at:     Pnt3<f32>,
                             ipd:    f32) -> FirstPersonStereo {
        let mut res = FirstPersonStereo {
            eye:           Pnt3::new(0.0, 0.0, 0.0),
            // left & right are initially wrong, don't take ipd into accound
            eye_left:      Pnt3::new(0.0, 0.0, 0.0),
            eye_right:     Pnt3::new(0.0, 0.0, 0.0),
            ipd:           ipd,
            yaw:           0.0,
            pitch:         0.0,
            yaw_step:      0.005,
            pitch_step:    0.005,
            move_step:     0.5,
            projection: PerspMat3::new(800.0 / 600.0, fov, znear, zfar),
            proj_view:  na::zero(),
            inv_proj_view:   na::zero(),
            last_cursor_pos: na::orig(),
            proj_view_left:  na::zero(),
            proj_view_right: na::zero(),
        };

        res.look_at_z(eye, at);

        res
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
        if self.pitch <= 0.0001 {
            self.pitch = 0.0001
        }

        let _pi: f32 = f32::consts::PI;
        if self.pitch > _pi - 0.0001 {
            self.pitch = _pi - 0.0001
        }
    }

    #[doc(hidden)]
    pub fn handle_left_button_displacement(&mut self, dpos: &Vec2<f32>) {
        self.yaw   = self.yaw   + dpos.x * self.yaw_step;
        self.pitch = self.pitch + dpos.y * self.pitch_step;

        self.update_restrictions();
        self.update_projviews();
    }

    fn update_eyes_location(&mut self) {
        // left and right are on a line perpendicular to both up and the target
        // up is always y
        let dir       = na::normalize(&(self.at() - self.eye));
        let tangent   = na::normalize(&na::cross(&Vec3::y(), &dir));
        self.eye_left = self.eye - tangent * (self.ipd / 2.0);
        self.eye_right = self.eye + tangent * (self.ipd / 2.0);
        //println(fmt!("eye_left = %f,%f,%f", self.eye_left.x as float, self.eye_left.y as float, self.eye_left.z as float));
        //println(fmt!("eye_right = %f,%f,%f", self.eye_right.x as float, self.eye_right.y as float, self.eye_right.z as float));
        // TODO: verify with an assert or something that the distance between the eyes is ipd, just to make me feel good.
    }

    #[doc(hidden)]
    pub fn handle_right_button_displacement(&mut self, dpos: &Vec2<f32>) {
        let at        = self.at();
        let dir       = na::normalize(&(at - self.eye));
        let tangent   = na::normalize(&na::cross(&Vec3::y(), &dir));
        let bitangent = na::cross(&dir, &tangent);

        self.eye = self.eye + tangent * (0.01 * dpos.x / 10.0) + bitangent * (0.01 * dpos.y / 10.0);
        // TODO: ugly - should move eye update to where eye_left & eye_right are updated
        self.update_eyes_location();
        self.update_restrictions();
        self.update_projviews();
    }

    #[doc(hidden)]
    pub fn handle_scroll(&mut self, yoff: f32) {
        let front: Vec3<f32> = self.view_transform().rotate(&Vec3::z());

        self.eye = self.eye + front * (self.move_step * yoff);

        self.update_eyes_location();
        self.update_restrictions();
        self.update_projviews();
    }

    fn update_projviews(&mut self) {
        self.proj_view = *self.projection.as_mat() * na::to_homogeneous(&na::inv(&self.view_transform()).unwrap());
        self.inv_proj_view = na::inv(&self.proj_view).unwrap();
        self.proj_view_left = *self.projection.as_mat() * na::to_homogeneous(&na::inv(&self.view_transform_left()).unwrap());
        self.proj_view_right = *self.projection.as_mat() * na::to_homogeneous(&na::inv(&self.view_transform_right()).unwrap());
    }

    fn transformation_eye(&self, eye: usize) -> Mat4<f32> {
        match eye {
            0usize => self.proj_view_left,
            1usize => self.proj_view_right,
            _      => panic!("bad eye index")
        }
    }

    /// The left eye camera view transformation
    fn view_transform_left(&self) -> Iso3<f32> {
        Iso3::look_at_z(&self.eye_left, &self.at(), &Vec3::y())
    }

    /// The right eye camera view transformation
    fn view_transform_right(&self) -> Iso3<f32> {
        Iso3::look_at_z(&self.eye_right, &self.at(), &Vec3::y())
    }

    /// return Inter Pupilary Distance
    pub fn ipd(&self) -> f32 {
        self.ipd
    }
    
    /// change Inter Pupilary Distance
    pub fn set_ipd(&mut self, ipd: f32) {
        self.ipd = ipd;

        self.update_eyes_location();
        self.update_restrictions();
        self.update_projviews();
    }

}

impl Camera for FirstPersonStereo {
    fn clip_planes(&self) -> (f32, f32) {
        (self.projection.znear(), self.projection.zfar())
    }

    /// The imaginary middle eye camera view transformation (i-e transformation without projection).
    fn view_transform(&self) -> Iso3<f32> {
        Iso3::look_at_z(&self.eye, &self.at(), &Vec3::y())
    }

    fn handle_event(&mut self, window: &glfw::Window, event: &WindowEvent) {
        match *event {
            WindowEvent::CursorPos(x, y) => {
                let curr_pos = Pnt2::new(x as f32, y as f32);

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
        let t                = self.view_transform();
        let front: Vec3<f32> = t.rotate(&Vec3::z());
        let right: Vec3<f32> = t.rotate(&Vec3::x());

        if window.get_key(Key::Up) == Action::Press {
            self.eye = self.eye + front * self.move_step
        }

        if window.get_key(Key::Down) == Action::Press {
            self.eye = self.eye + front * (-self.move_step)
        }

        if window.get_key(Key::Right) == Action::Press {
            self.eye = self.eye + right * (-self.move_step)
        }

        if window.get_key(Key::Left) == Action::Press {
            self.eye = self.eye + right * self.move_step
        }

        self.update_eyes_location();
        self.update_restrictions();
        self.update_projviews();
    }

    fn upload(&self, pass: usize, uniform: &mut ShaderUniform<Mat4<f32>>) {
        uniform.upload(&self.transformation_eye(pass));
    }

    fn num_passes(&self) -> usize { 2usize }

    fn start_pass(&self, pass: usize, window: &glfw::Window) {
        let (win_w, win_h) = window.get_size();
        let (x, y, w, h) = match pass {
            0usize => (0, 0, win_w / 2 , win_h),
            1usize => (win_w / 2, 0, win_w / 2, win_h),
            _      => panic!("stereo first person takes only two passes")
        };
        verify!(gl::Viewport(x, y, w, h));
        verify!(gl::Scissor(x, y, w, h));
    }

    fn render_complete(&self, window: &glfw::Window) {
        let (w, h) = window.get_size();
        verify!(gl::Viewport(0, 0, w, h));
        verify!(gl::Scissor(0, 0, w, h));
    }
}
