extern mod kiss3d;
extern mod nalgebra;

use nalgebra::traits::translation::Translation;
use nalgebra::traits::rotation::rotate_wrt_center;
use nalgebra::vec::Vec3;
use kiss3d::window::{Window, StickToCamera};

fn main()
{
  do Window::spawn |w| {
    let b = w.add_cube().set_color(0.2, 0.2, 0.2);

    b.transformation().translate_by(&Vec3::new([0.0, 0.0, 0.0]));

    w.set_loop_callback(|w| {
      rotate_wrt_center(w.objects()[0].transformation(),
                        &Vec3::new::<f32>([0.0, 0.014, 0.0]));
    });

    w.set_light(StickToCamera);
  };
}
