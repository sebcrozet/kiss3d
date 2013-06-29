extern mod kiss3d;

use kiss3d::window::Window;

fn main()
{
  do Window::spawn |w| {
    w.add_cube();
  };
}
