extern mod kiss3d;
extern mod nalgebra;

use nalgebra::traits::rotation::Rotation;
use nalgebra::vec::Vec3;
use kiss3d::window;

fn main()
{
  do window::Window::spawn |window|
  {
    let c = window.add_cube(1.0, 1.0, 1.0).set_color(1.0, 0.0, 0.0);

    do window.set_loop_callback |_|
    { c.transformation().rotate_by(&Vec3::new([0.0f64, 0.014, 0.0])) }

    window.set_light(window::StickToCamera);
  }
}
