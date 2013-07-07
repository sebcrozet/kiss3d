extern mod kiss3d;
extern mod nalgebra;

use std::rand::random;
use kiss3d::window;

fn main()
{
  do window::Window::spawn(~"Kiss3d: cube") |window|
  {
    let c    = window.add_quad(5.0, 4.0, 500, 400).set_color(random(), random(), random());
    let time = @mut 0.016f32;

    do window.set_loop_callback |_|
    {
      do c.modify_vertices |vs|
      {
        for vs.mut_iter().advance |v|
        {
          v.at[2] = time.sin() * (((v.at[0] + *time) * 4.0).cos() +
                    time.sin() * ((v.at[1] + *time) * 4.0 + *time).cos()) / 2.0
        }

        true
      }

      *time = *time + 0.016;
    }

    window.set_light(window::StickToCamera);
  }
}
