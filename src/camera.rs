use std::num::Zero;
use std::ptr;
use glcore::types::GL_VERSION_1_0::*;
use glcore::consts::GL_VERSION_1_1::*;
use glcore::functions::GL_VERSION_2_0::*;
use nalgebra::traits::norm::Norm;
use nalgebra::traits::cross::Cross;
use nalgebra::traits::dot::Dot;
use nalgebra::vec::{Vec2, Vec3};
use nalgebra::mat::Mat4;

pub enum CameraMode
{
  ArcBall(Vec3<GLfloat>, Vec3<GLfloat>, float), // FIXME: add parameters for sencitivity
  FPS                                           // FIXME: add parameters for sencitivity
}

pub struct Camera
{
  priv changed:       bool,
  priv mode:          CameraMode,
  priv distance:      float,
  priv pitch:         float,
  priv yaw:           float,
  priv mouse_pressed: bool,
  priv mouse_start:   Vec2<float>
}

impl Camera
{
  pub fn new(mode: CameraMode) -> Camera
  {
    Camera {
      changed:       true,
      mode:          mode,
      distance:      3.0,
      pitch:         Real::pi::<float>() / 2.0,
      yaw:           0.0,
      mouse_pressed: false,
      mouse_start: Zero::zero()
    }
  }

  pub fn position(&mut self) -> Vec3<GLfloat>
  {
    match self.mode
    {
      ArcBall(ref pos, _, _) => *pos,
      FPS                    => fail!("FPS camera not yet implemented.")
    }
  }

  pub fn set_mode(&mut self, mode: CameraMode)
  {
    self.mode    = mode;
    self.changed = true;
    // FIXME: update internal datas
  }

  pub fn handle_cursor_pos(&mut self, xpos: float, ypos: float)
  {
    let yaw_step   = 0.005; // FIXME: should be a parameter of the camera
    let pitch_step = 0.005; // FIXME: should be a parameter of the camera

    if self.mouse_pressed
    {
      let dx = -xpos + self.mouse_start.at[0];
      let dy = ypos - self.mouse_start.at[1];

      self.yaw   = self.yaw   - dx * yaw_step;
      self.pitch = self.pitch - dy * pitch_step;

      self.changed = true
    }

    self.mouse_start.at[0] = xpos;
    self.mouse_start.at[1] = ypos;
  }

  pub fn handle_mouse_button(&mut self, _: int, action: int, _: int)
  {
    if action == 1
    { self.mouse_pressed = true }
    else
    { self.mouse_pressed = false }
  }

  pub fn handle_scroll(&mut self, _: float, yoff: float)
  {
    match self.mode
    {
      ArcBall(_, _, zoom_factor) => self.distance += zoom_factor * yoff / 120.0,
      FPS                        => fail!("FPS mode not yet implemented.")
    }

    self.changed = true;
  }

  pub fn handle_keyboard(&mut self,
                         _: int,
                         _: int)
  {
    // FIXME: useful for FPS mode
  }

  fn update(&mut self)
  {
    let curr_mode = self.mode;
    match curr_mode
    {
      ArcBall(ref _0, ref at, ref _1) => {
        if (self.distance < 0.00001)
        { self.distance = 0.00001 }

        if (self.pitch <= 0.0001)
        { self.pitch = 0.0001 }

        if (self.pitch > Real::pi::<float>() - 0.0001)
        { self.pitch = Real::pi::<float>() - 0.0001 }

        let px = at.at[0] as float + self.distance * self.yaw.cos() * self.pitch.sin();
        let py = at.at[1] as float + self.distance * self.pitch.cos();
        let pz = at.at[2] as float + self.distance * self.yaw.sin() * self.pitch.sin();

        self.mode = ArcBall(Vec3::new([px as GLfloat, py as GLfloat, pz as GLfloat]), *at, *_1);
      }
      FPS => { }
    }
  }

  pub fn upload(&mut self, view_location: i32)
  {
    if self.changed // do not reupload if nothing changed
    {
      self.update();

      let (eye, at) = match self.mode
      {
        ArcBall(ref e, ref a, _) => (e, a),
        FPS                      => fail!("FPS camera not yet implemented.")
      };

      let zaxis = (eye - *at).normalized();
      let xaxis = Vec3::new([0.0, 1.0, 0.0]).cross(&zaxis).normalized();
      let yaxis = zaxis.cross(&xaxis);

      let look_at= Mat4::new::<GLfloat>(
        [
          xaxis.at[0], yaxis.at[0], zaxis.at[0], 0.0,
          xaxis.at[1], yaxis.at[1], zaxis.at[1], 0.0,
          xaxis.at[2], yaxis.at[2], zaxis.at[2], 0.0,
          -xaxis.dot(eye), -yaxis.dot(eye), -zaxis.dot(eye), 1.0
        ]
      );

      unsafe {
        glUniformMatrix4fv(view_location,
                           1,
                           GL_FALSE,
                           ptr::to_unsafe_ptr(&look_at.mij[0]));
      }

      self.changed = false;
    }
  }
}
