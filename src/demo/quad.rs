extern mod kiss3d;
extern mod nalgebra;

use nalgebra::traits::rotation::Rotation;
use nalgebra::vec::Vec3;
use kiss3d::window;

fn main()
{
  do window::Window::spawn(~"Kiss3d: cube") |window|
  {
    let c = window.add_quad(5.0, 4.0, 5, 4, true).set_color(1.0, 0.0, 0.0);

    do window.set_loop_callback |_|
    { c.transformation().rotate_by(&Vec3::new([0.0f64, 0.014, 0.0])) }

    window.set_light(window::StickToCamera);
  }
}
