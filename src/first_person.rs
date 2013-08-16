use std::num::{One, atan2};
use nalgebra::traits::norm::Norm;
use nalgebra::traits::cross::Cross;
use nalgebra::traits::scalar_op::ScalarMul;
use nalgebra::traits::rotation::Rotate;
use nalgebra::types::Iso3f64;
use nalgebra::vec::Vec3;
use glfw::consts::*;
use glfw;
use event;

/// First-person camera mode.
///
///   * Left button press + drag - look around
///   * Right button press + drag - translates the camera position on the plane orthogonal to the view
///   direction
///   * Scroll in/out - zoom in/out
///   * Enter key - look at the origin
#[deriving(ToStr)]
pub struct FirstPerson {
    /// The camera position
    eye:   Vec3<f64>,
    /// Yaw of the camera (rotation along the y axis).
    yaw:   f64,
    /// Pitch of the camera (rotation along the x axis).
    pitch: f64,

    /// Increment of the yaw per unit mouse movement. The default value is 0.005.
    yaw_step:   f64,
    /// Increment of the pitch per unit mouse movement. The default value is 0.005.
    pitch_step: f64,
    /// Increment of the translation per arrow press. The default value is 0.1.
    move_step: f64
}

impl FirstPerson {
    /// Creates a new arc ball camera with default sensitivity values.
    pub fn new(eye: Vec3<f64>, at: Vec3<f64>) -> FirstPerson {
        let mut res = FirstPerson {
            eye:           Vec3::new(0.0, 0.0, 0.0),
            yaw:           0.0,
            pitch:         0.0,
            yaw_step:      0.005,
            pitch_step:    0.005,
            move_step:     0.5
        };

        res.look_at_z(eye, at);

        res
    }


    /// Changes the orientation and position of the arc-ball to look at the specified point.
    pub fn look_at_z(&mut self, eye: Vec3<f64>, at: Vec3<f64>) {
        let dist  = (eye - at).norm();

        let pitch = ((at.y - eye.y) / dist).acos();
        let yaw   = atan2(at.z - eye.z, at.x - eye.x);

        self.eye   = eye;
        self.yaw   = yaw;
        self.pitch = pitch;
    }

    /// The camera actual transformation.
    pub fn transformation(&self) -> Iso3f64 {
        let mut id = One::one::<Iso3f64>();
        id.look_at_z(&self.eye, &self.at(), &Vec3::y());

        id
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

        if (self.pitch > Real::pi::<f64>() - 0.0001) {
            self.pitch = Real::pi::<f64>() - 0.0001
        }
    }

    #[doc(hidden)]
    pub fn handle_left_button_displacement(&mut self, dx: float, dy: float) {
        self.yaw   = self.yaw   + dx as f64 * self.yaw_step;
        self.pitch = self.pitch + dy as f64 * self.pitch_step;

        self.update_restrictions();
    }

    #[doc(hidden)]
    pub fn handle_right_button_displacement(&mut self, dx: float, dy: float) {
        let at        = self.at();
        let dir       = (at - self.eye).normalized();
        let tangent   = Vec3::y().cross(&dir).normalized();
        let bitangent = dir.cross(&tangent);

        self.eye = self.eye + tangent.scalar_mul(&(0.01 * dx as f64 / 10.0))
            + bitangent.scalar_mul(&(0.01 * dy as f64 / 10.0))
    }

    #[doc(hidden)]
    pub fn handle_scroll(&mut self, yoff: float) {
        let front: Vec3<f64> = self.transformation().rotate(&Vec3::z());

        self.eye = self.eye + front.scalar_mul(&(self.move_step * (yoff as f64)))
    }

    #[doc(hidden)]
    pub fn update(&mut self, window: &glfw::Window) -> bool {
        let t                = self.transformation();
        let front: Vec3<f64> = t.rotate(&Vec3::z());
        let right: Vec3<f64> = t.rotate(&Vec3::x());

        let mut changed = false;

        if window.get_key(KEY_UP) == TRUE {
            changed = true;
            self.eye = self.eye + front.scalar_mul(&self.move_step)
        }

        if window.get_key(KEY_DOWN) == TRUE {
            changed = true;
            self.eye = self.eye + front.scalar_mul(&-self.move_step)
        }

        if window.get_key(KEY_RIGHT) == TRUE {
            changed = true;
            self.eye = self.eye + right.scalar_mul(&-self.move_step)
        }

        if window.get_key(KEY_LEFT) == TRUE {
            changed = true;
            self.eye = self.eye + right.scalar_mul(&self.move_step)
        }

        changed
    }

    #[doc(hidden)]
    pub fn handle_keyboard(&mut self, _: &event::KeyboardEvent) {
    }
}
