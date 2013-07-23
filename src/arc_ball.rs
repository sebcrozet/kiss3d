use std::num::One;
use nalgebra::traits::norm::Norm;
use nalgebra::traits::cross::Cross;
use nalgebra::traits::scalar_op::ScalarMul;
use nalgebra::types::Iso3f64;
use nalgebra::vec::Vec3;

#[deriving(Clone, ToStr)]
pub struct ArcBall
{
  at:    Vec3<f64>,
  yaw:   f64,
  pitch: f64,
  dist:  f64,

  yaw_step:   f64,
  pitch_step: f64,
  dist_step:  f64,
}

impl ArcBall
{
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

  pub fn look_at(&mut self, _: Vec3<f64>, at: Vec3<f64>)
  {
    self.at = at;
    fail!("Not yet implemented.");
  }

  pub fn transformation(&self) -> Iso3f64
  {
    let mut id = One::one::<Iso3f64>();
    id.look_at_z(&self.eye(), &self.at, &Vec3::new(0.0, 1.0, 0.0));

    id
  }

  pub fn eye(&self) -> Vec3<f64>
  {
    let px = self.at.x + self.dist * self.yaw.cos() * self.pitch.sin();
    let py = self.at.y + self.dist * self.pitch.cos();
    let pz = self.at.z + self.dist * self.yaw.sin() * self.pitch.sin();

    Vec3::new(px, py, pz)
  }

  pub fn update_restrictions(&mut self)
  {
    if (self.dist < 0.00001)
    { self.dist = 0.00001 }

    if (self.pitch <= 0.0001)
    { self.pitch = 0.0001 }

    if (self.pitch > Real::pi::<f64>() - 0.0001)
    { self.pitch = Real::pi::<f64>() - 0.0001 }
  }

  pub fn handle_left_button_displacement(&mut self, dx: float, dy: float)
  {
    self.yaw   = self.yaw   + dx as f64 * self.yaw_step;
    self.pitch = self.pitch - dy as f64 * self.pitch_step;

    self.update_restrictions();
  }

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


  pub fn handle_scroll(&mut self, yoff: float)
  {
    self.dist = self.dist + self.dist_step * (yoff as f64) / 120.0;
    self.update_restrictions();
  }
}
