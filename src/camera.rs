pub enum CameraMode
{
  ArcBall, // FIXME: add parameters for sencitivity
  FPS      // FIXME: add parameters for sencitivity
}

pub struct Camera
{
  mode: CameraMode
}

impl Camera
{
  pub fn new(mode: CameraMode) -> Camera
  {
    Camera {
      mode: mode
    }
  }

  pub fn upload(&self)
  {
  }
}
