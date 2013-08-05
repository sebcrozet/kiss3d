use std::num::{Zero, One, atan2};
use nalgebra::traits::norm::Norm;
use nalgebra::traits::cross::Cross;
use nalgebra::traits::scalar_op::ScalarMul;
use nalgebra::traits::rotation::Rotate;
use nalgebra::types::Iso3f64;
use nalgebra::vec::Vec3;
use glfw::consts::*;
use event;

#[deriving(Eq, ToStr)]
enum KeyState {
    True,
    False,
    Impulse
}

/// First-person camera mode.
///
///   * Left button press + drag - look around
///   * Right button press + drag - translates the camera position on the plane orthogonal to the view
///   direction
///   * Scroll in/out - zoom in/out
///   * Enter key - look at the origin
#[deriving(ToStr)]
pub struct FirstPerson {
    priv up_pressed:    KeyState,
    priv down_pressed:  KeyState,
    priv right_pressed: KeyState,
    priv left_pressed:  KeyState,
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
            up_pressed:    False,
            down_pressed:  False,
            right_pressed: False,
            left_pressed:  False,
            eye:           Vec3::new(0.0, 0.0, 0.0),
            yaw:           0.0,
            pitch:         0.0,
            yaw_step:      0.005,
            pitch_step:    0.005,
            move_step:     0.1
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
    pub fn update(&mut self) -> bool {
        let t                = self.transformation();
        let front: Vec3<f64> = t.rotate(&Vec3::z());
        let right: Vec3<f64> = t.rotate(&Vec3::x());

        if self.up_pressed == True || self.up_pressed == Impulse {
            self.eye = self.eye + front.scalar_mul(&self.move_step)
        }

        if self.down_pressed == True || self.down_pressed == Impulse {
            self.eye = self.eye + front.scalar_mul(&-self.move_step)
        }

        if self.right_pressed == True || self.right_pressed == Impulse {
            self.eye = self.eye + right.scalar_mul(&-self.move_step)
        }

        if self.left_pressed == True || self.left_pressed == Impulse {
            self.eye = self.eye + right.scalar_mul(&self.move_step)
        }

        let changed = self.up_pressed    == True || self.up_pressed    == Impulse ||
            self.down_pressed  == True || self.down_pressed  == Impulse ||
            self.right_pressed == True || self.right_pressed == Impulse ||
            self.left_pressed  == True || self.left_pressed  == Impulse;

        if self.up_pressed == Impulse {
            self.up_pressed = False
        }

        if self.down_pressed == Impulse {
            self.down_pressed = False
        }

        if self.right_pressed == Impulse {
            self.right_pressed = False
        }

        if self.left_pressed == Impulse {
            self.left_pressed = False
        }


        changed
    }

    #[doc(hidden)]
    pub fn handle_keyboard(&mut self, event: &event::KeyboardEvent) {
        match *event {
            event::KeyPressed(button)  => self.set_key_state(button, True),
            event::KeyReleased(button) => {
                if button == KEY_ENTER && !self.eye.is_zero() {
                    self.look_at_z(self.eye, Zero::zero())
                }
                else {
                    self.set_key_state(button, Impulse)
                }
            },
        }
    }

    fn set_key_state(&mut self, button: event::MouseButton, state: KeyState) {
        match button {
            KEY_RIGHT => self.right_pressed = state,
            KEY_LEFT  => self.left_pressed  = state,
            KEY_DOWN  => self.down_pressed  = state,
            KEY_UP    => self.up_pressed    = state,
            _         => { }
        }
    }
}
