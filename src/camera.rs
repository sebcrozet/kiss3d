use std::cast;
use std::num::Zero;
use glcore::types::GL_VERSION_1_0::*;
use glcore::consts::GL_VERSION_1_1::*;
use glcore::functions::GL_VERSION_2_0::*;
use glfw::consts::*;
use nalgebra::types::Iso3f64;
use nalgebra::traits::inv::Inv;
use nalgebra::traits::mat_cast::MatCast;
use nalgebra::traits::transpose::Transpose;
use nalgebra::traits::homogeneous::ToHomogeneous;
use nalgebra::vec::{Vec2, Vec3};
use nalgebra::mat::Mat4;
use event;
use arc_ball;
use first_person;

enum Button {
    RightButton,
    LeftButton,
    ReleasedButton
}

pub enum CameraMode {
    ArcBall(arc_ball::ArcBall),
    FirstPerson(first_person::FirstPerson)
}

/// Structure representing the camera.
pub struct Camera {
    priv changed:       bool,
    priv mode:          CameraMode,
    priv mouse_pressed: Button,
    priv mouse_start:   Vec2<float>,
}

impl Camera {
    #[doc(hidden)]
    pub fn new(mode: CameraMode) -> Camera {
        Camera {
            changed:       true,
            mode:          mode,
            mouse_pressed: ReleasedButton,
            mouse_start:   Zero::zero(),
        }
    }

    /// Modify the camera mode. Any modifications will be taken in account during the nexit display
    /// loop.
    pub fn change_mode<'r>(&'r mut self, f: &fn(&'r mut CameraMode)) {
        f(&'r mut self.mode);

        self.changed = true;
    }

    /// Changes the orientation and position of the camera to look at the specified point.
    pub fn look_at_z(&mut self, eye: Vec3<f64>, at: Vec3<f64>) {
        match self.mode {
            ArcBall(ref mut ab)     => ab.look_at_z(eye, at),
            FirstPerson(ref mut fp) => fp.look_at_z(eye, at),
        }

        self.changed = true;
    }

    /// Indicates whether this camera has changed since the last update.
    pub fn needs_rendering(&self) -> bool {
        self.changed
    }

    /// The current camera mode.
    pub fn mode(&self) -> CameraMode {
        self.mode
    }

    #[doc(hidden)]
    pub fn handle_mouse(&mut self, event: &event::MouseEvent) {
        match *event {
            event::ButtonPressed(button, _)  => {
                self.mouse_pressed = if button == MOUSE_BUTTON_1 {
                    LeftButton
                } else {
                    RightButton
                }
            },
            event::ButtonReleased(_, _) => {
                self.mouse_pressed = ReleasedButton
            },
            event::CursorPos(x, y) => self.handle_cursor_pos(x, y),
            event::Scroll(_, off)  => self.handle_scroll(off)
        }
    }

    fn handle_cursor_pos(&mut self, xpos: float, ypos: float) {
        let dx = xpos - self.mouse_start.x;
        let dy = ypos - self.mouse_start.y;

        match self.mode {
            ArcBall(ref mut arcball) => {
                match self.mouse_pressed {
                    RightButton => {
                        arcball.handle_right_button_displacement(dx, dy);
                        self.changed = true
                    },
                    LeftButton => {
                        arcball.handle_left_button_displacement(dx, dy);
                        self.changed = true
                    },
                    ReleasedButton => { }
                }
            },
            FirstPerson(ref mut fp) => {
                match self.mouse_pressed {
                    RightButton => {
                        fp.handle_right_button_displacement(dx, dy);
                        self.changed = true
                    },
                    LeftButton => {
                        fp.handle_left_button_displacement(dx, dy);
                        self.changed = true
                    },
                    _ => { }
                }
            },
        }

        self.mouse_start.x = xpos;
        self.mouse_start.y = ypos;
    }

    fn handle_scroll(&mut self, off: float) {
        match self.mode {
            ArcBall(ref mut ab)     => ab.handle_scroll(off),
            FirstPerson(ref mut fp) => fp.handle_scroll(off)
        }

        self.changed = true;
    }

    #[doc(hidden)]
    pub fn update(&mut self) {
        match self.mode {
            ArcBall(_)              => { },
            FirstPerson(ref mut fp) => self.changed =  fp.update() || self.changed
        };
    }

    /// Switches the current camera mode between `FirstPerson` and `ArcBall`.
    pub fn switch_mode(&mut self) {
        self.mode = match self.mode {
            ArcBall(ref ab)     => FirstPerson(first_person::FirstPerson::new(ab.eye(), ab.at)),
            FirstPerson(ref fp) => ArcBall(arc_ball::ArcBall::new(fp.eye, fp.at()))
        };
        self.changed = true;
    }

    #[doc(hidden)]
    pub fn handle_keyboard(&mut self, event: &event::KeyboardEvent) {
        match *event {
            event::KeyReleased(k) => if k == KEY_TAB { self.switch_mode() },
            _ => { }
        }

        match self.mode {
            ArcBall(ref mut ab)     => ab.handle_keyboard(event),
            FirstPerson(ref mut fp) => fp.handle_keyboard(event)
        }

        self.changed = true;
    }

    /// The transformation of the camera. This corresponds to the position and orientation of the
    /// camera. Note that this is not the projection used as the view matrix. The view matrix is the
    /// inverse of this matrix.
    pub fn transformation(&self) -> Iso3f64 {
        match self.mode {
            ArcBall(ref ab)     => ab.transformation(),
            FirstPerson(ref fp) => fp.transformation()
        }
    }

    #[doc(hidden)]
    pub fn upload(&mut self, view_location: i32) {
        let mut homo = self.transformation().inverse().unwrap().to_homogeneous();

        homo.transpose();

        let homo32: Mat4<GLfloat> = MatCast::from(homo);

        unsafe {
            glUniformMatrix4fv(
                view_location,
                1,
                GL_FALSE,
                cast::transmute(&homo32));
        }

        self.changed = false;
    }
}
