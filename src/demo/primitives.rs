extern mod kiss3d;
extern mod nalgebra;

use std::rand::random;
use nalgebra::traits::rotation::rotate_wrt_center;
use nalgebra::traits::translation::Translation;
use nalgebra::vec::Vec3;
use kiss3d::window::{Window, StickToCamera};

fn main() {
    do Window::spawn("Kiss3d: primitives") |w| {
        let c = w.add_cube(1.0, 1.0, 1.0).set_color(random(), random(), random());
        let b = w.add_sphere(0.5).set_color(random(), random(), random());
        let p = w.add_cone(1.0, 0.5).set_color(random(), random(), random());
        let y = w.add_cylinder(1.0, 0.5).set_color(random(), random(), random());

        c.transformation().translate_by(&Vec3::new(1.0, 0.0, 0.0));
        b.transformation().translate_by(&Vec3::new(3.0, 0.0, 0.0));
        p.transformation().translate_by(&Vec3::new(-1.0, 0.0, 0.0));
        y.transformation().translate_by(&Vec3::new(-3.0, 0.0, 0.0));

        do w.set_loop_callback {
            rotate_wrt_center(c.transformation(), &Vec3::new(0.0f64, 0.014, 0.0));
            rotate_wrt_center(b.transformation(), &Vec3::new(0.0f64, 0.014, 0.0));
            rotate_wrt_center(p.transformation(), &Vec3::new(0.0f64, 0.014, 0.0));
            rotate_wrt_center(y.transformation(), &Vec3::new(0.0f64, 0.014, 0.0));
        };

        w.set_light(StickToCamera);
    };
}
