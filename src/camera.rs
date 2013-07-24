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
use nalgebra::vec::Vec2;
use nalgebra::mat::Mat4;
use event;
use arc_ball;

enum Button
{
  RightButton,
  LeftButton,
  ReleasedButton
}

pub enum CameraMode
{
  ArcBall(arc_ball::ArcBall),
  FPS
}

pub struct Camera
{
  priv changed:       bool,
  priv mode:          CameraMode,
  priv mouse_pressed: Button,
  priv mouse_start:   Vec2<float>,
}

impl Camera
{
  pub fn new(mode: CameraMode) -> Camera
  {
    Camera {
      changed:       true,
      mode:          mode,
      mouse_pressed: ReleasedButton,
      mouse_start:   Zero::zero(),
    }
  }

  pub fn change_mode<'r>(&'r mut self, f: &fn(&'r mut CameraMode))
  {
    f(&'r mut self.mode);

    self.changed = true;
  }

  pub fn mode(&self) -> CameraMode
  { self.mode }

  pub fn handle_mouse(&mut self, event: &event::MouseEvent)
  {
    match *event
    {
      event::ButtonPressed(button, _)  => {
        self.mouse_pressed = if button == MOUSE_BUTTON_1 { LeftButton } else { RightButton }
      },
      event::ButtonReleased(_, _) => {
        self.mouse_pressed = ReleasedButton
      },
      event::CursorPos(x, y) => self.handle_cursor_pos(x, y),
      event::Scroll(_, off)  => self.handle_scroll(off)
    }
  }

  fn handle_cursor_pos(&mut self, xpos: float, ypos: float)
  {
    let dx = xpos - self.mouse_start.x;
    let dy = ypos - self.mouse_start.y;

    match self.mode
    {
      ArcBall(ref mut arcball) =>
      {
        match self.mouse_pressed
        {
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
      FPS => fail!("Not yet implemented.")
    }

    self.mouse_start.x = xpos;
    self.mouse_start.y = ypos;
  }

  fn handle_scroll(&mut self, off: float)
  {
    match self.mode
    {
      ArcBall(ref mut ab) => ab.handle_scroll(off),
      FPS                 => fail!("FPS mode not yet implemented.")
    }

    self.changed = true;
  }

  pub fn handle_keyboard(&mut self, event: &event::KeyboardEvent)
  {
    match self.mode
    {
      ArcBall(ref mut ab) => {
        ab.handle_keyboard(event);
        self.changed = true;
      },
      FPS                 => fail!("FPS mode not yet implemented.")
    }
  }

  pub fn transformation(&self) -> Iso3f64
  {
    match self.mode
    {
      ArcBall(ref ab) => ab.transformation(),
      FPS             => fail!("Not yet implemented.")
    }
  }

  pub fn upload(&mut self, view_location: i32)
  {
    if self.changed // do not reupload if nothing changed
    {
      // FIXME: its a bit weird that we have to type everything exlicitly…
      let mut homo: Mat4<f64> = self.transformation().inverse().unwrap().to_homogeneous();

      homo.transpose();

      let homo32: Mat4<GLfloat> = MatCast::from(homo);

      unsafe {
        glUniformMatrix4fv(view_location,
                           1,
                           GL_FALSE,
                           cast::transmute(&homo32));
      }

      self.changed = false;
    }
  }
}
