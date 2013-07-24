use std::num::{Zero, One};
use nalgebra::traits::norm::Norm;
use nalgebra::traits::cross::Cross;
use nalgebra::traits::scalar_op::ScalarMul;
use nalgebra::types::Iso3f64;
use nalgebra::vec::Vec3;
use glfw::consts::*;
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
pub struct ArcBall
{
  /// The focus point.
  at:    Vec3<f64>,
  /// Yaw of the camera (rotation along the y axis).
  yaw:   f64,
  /// Pitch of the camera (rotation along the x axis).
  pitch: f64,
  /// Distance from the camera to the `at` focus point.
  dist:  f64,

  /// Increment of the yaw per unit mouse movement. The default value is 0.005.
  yaw_step:   f64,
  /// Increment of the pitch per unit mouse movement. The default value is 0.005.
  pitch_step: f64,
  /// Increment of the distance per unit scrolling. The default value is 40.0.
  dist_step:  f64,
}

impl ArcBall
{
  /// Creates a new arc ball camera with default sensitivity values.
  pub fn new() -> ArcBall
  {
    ArcBall {
      at:         Vec3::new(0.0, 0.0, 0.0),
      yaw:        -Real::pi::<f64>() / 2.0,
      pitch:      Real::pi::<f64>() / 2.0,
      dist:       3.0,
      yaw_step:   0.005,
      pitch_step: 0.005,
      dist_step:  40.0
    }
  }

  /// Changes the orientation and position of the arc-ball to look at the specified point.
  pub fn look_at(&mut self, _: Vec3<f64>, at: Vec3<f64>)
  {
    self.at = at;
    fail!("Not yet implemented.");
  }

  /// The camera actual transformation.
  pub fn transformation(&self) -> Iso3f64
  {
    let mut id = One::one::<Iso3f64>();
    id.look_at_z(&self.eye(), &self.at, &Vec3::new(0.0, 1.0, 0.0));

    id
  }

  /// The position of the camera.
  pub fn eye(&self) -> Vec3<f64>
  {
    let px = self.at.x + self.dist * self.yaw.cos() * self.pitch.sin();
    let py = self.at.y + self.dist * self.pitch.cos();
    let pz = self.at.z + self.dist * self.yaw.sin() * self.pitch.sin();

    Vec3::new(px, py, pz)
  }

  fn update_restrictions(&mut self)
  {
    if (self.dist < 0.00001)
    { self.dist = 0.00001 }

    if (self.pitch <= 0.0001)
    { self.pitch = 0.0001 }

    if (self.pitch > Real::pi::<f64>() - 0.0001)
    { self.pitch = Real::pi::<f64>() - 0.0001 }
  }

  #[doc(hidden)]
  pub fn handle_left_button_displacement(&mut self, dx: float, dy: float)
  {
    self.yaw   = self.yaw   + dx as f64 * self.yaw_step;
    self.pitch = self.pitch - dy as f64 * self.pitch_step;

    self.update_restrictions();
  }

  #[doc(hidden)]
  pub fn handle_right_button_displacement(&mut self, dx: float, dy: float)
  {
    let eye       = self.eye();
    let dir       = (self.at - eye).normalized();
    let tangent   = Vec3::new(0.0, 1.0, 0.0).cross(&dir).normalized();
    let bitangent = dir.cross(&tangent);

    let mult = self.dist / 1000.0;

    self.at = self.at + tangent.scalar_mul(&(dx as f64 * mult))
                      + bitangent.scalar_mul(&(dy as f64 * mult))
  }

  #[doc(hidden)]
  pub fn handle_scroll(&mut self, yoff: float)
  {
    self.dist = self.dist + self.dist_step * (yoff as f64) / 120.0;
    self.update_restrictions();
  }

  #[doc(hidden)]
  pub fn handle_keyboard(&mut self, event: &event::KeyboardEvent)
  {
    match *event
    {
      event::KeyReleased(button) => if button == KEY_ENTER { self.at = Zero::zero() },
      _ => { }
    }
  }
}
