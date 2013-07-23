extern mod kiss3d;
extern mod nalgebra;

use nalgebra::traits::translation::Translation;
use nalgebra::vec::Vec3;
use kiss3d::window;
use kiss3d::camera;

fn main()
{
  do window::Window::spawn(~"Kiss3d: cube") |w|
  {
    w.add_cube(1.0, 0.01, 0.01).set_color(1.0, 0.0, 0.0)
                               .transformation()
                               .translate_by(&Vec3::new(0.505, 0.0, 0.0));

    w.add_cube(0.01, 1.0, 0.01).set_color(0.0, 1.0, 0.0)
                               .transformation()
                               .translate_by(&Vec3::new(0.0, 0.505, 0.0));

    w.add_cube(0.01, 0.01, 1.0).set_color(0.0, 0.0, 1.0)
                               .transformation()
                               .translate_by(&Vec3::new(0.0, 0.0, 0.505));

    do w.set_loop_callback |_|
    {
      // do w.camera().change_mode |mode|
      // {
      //   match *mode
      //   {
      //     camera::ArcBall(ref mut ab) => ab.yaw = ab.yaw + 0.05,
      //     _                           => { }
      //   }
      // }
    }

    w.set_light(window::StickToCamera);
  }
}
