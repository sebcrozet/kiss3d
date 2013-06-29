extern mod kiss3d;
extern mod nalgebra;

use nalgebra::traits::translation::Translation;
use nalgebra::traits::rotation::Rotation;
use nalgebra::vec::Vec3;
use kiss3d::window::Window;

fn main()
{
  do Window::spawn |w| {
    let b = w.add_cube().set_color(0.0, 0.5, 0.0);

    b.transformation().rotate_by(&Vec3::new([0.0, 3.14 / 4.0, 3.14 / 4.0]));
    b.transformation().translate_by(&Vec3::new([0.0, 0.0, -4.00]));
  };
}
