extern mod kiss3d;
extern mod nalgebra;

use std::rand::random;
use kiss3d::window;

fn main()
{
  do window::Window::spawn(~"Kiss3d: quad waves") |window|
  {
    let c    = window.add_quad(5.0, 4.0, 500, 400).set_color(random(), random(), random());
    let time = @mut 0.016f32;

    do window.set_loop_callback
    {
      do c.modify_vertices |vs|
      {
        foreach v in vs.mut_iter()
        {
          v.z = time.sin() * (((v.x + *time) * 4.0).cos() +
                time.sin() * ((v.y + *time) * 4.0 + *time).cos()) / 2.0
        }

        true
      }

      *time = *time + 0.016;
    }

    window.set_light(window::StickToCamera);
  }
}
