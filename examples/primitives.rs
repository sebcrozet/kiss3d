extern mod kiss3d;
extern mod nalgebra;

use std::rand::random;
use nalgebra::mat;
use nalgebra::mat::{Translation, rotate_wrt_center};
use nalgebra::vec::Vec3;
use kiss3d::window::{Window, StickToCamera};

#[start]
fn start(argc: int, argv: **u8, crate_map: *u8) -> int {
    std::rt::start_on_main_thread(argc, argv, crate_map, main)
}

fn main() {
    do Window::spawn("Kiss3d: primitives") |w| {
        let c = w.add_cube(1.0, 1.0, 1.0).set_color(random(), random(), random());
        let b = w.add_sphere(0.5).set_color(random(), random(), random());
        let p = w.add_cone(1.0, 0.5).set_color(random(), random(), random());
        let y = w.add_cylinder(1.0, 0.5).set_color(random(), random(), random());
        let a = w.add_capsule(1.0, 0.5).set_color(random(), random(), random());

        c.transformation().translate_by(&Vec3::new(2.0, 0.0, 0.0));
        b.transformation().translate_by(&Vec3::new(4.0, 0.0, 0.0));
        p.transformation().translate_by(&Vec3::new(-2.0, 0.0, 0.0));
        y.transformation().translate_by(&Vec3::new(-4.0, 0.0, 0.0));
        a.transformation().translate_by(&Vec3::new(0.0, 0.0, 0.0));

        do w.set_loop_callback {
            mat::rotate_wrt_center(c.transformation(), &Vec3::new(0.0f64, 0.014, 0.0));
            mat::rotate_wrt_center(b.transformation(), &Vec3::new(0.0f64, 0.014, 0.0));
            mat::rotate_wrt_center(p.transformation(), &Vec3::new(0.0f64, 0.014, 0.0));
            mat::rotate_wrt_center(y.transformation(), &Vec3::new(0.0f64, 0.014, 0.0));
            mat::rotate_wrt_center(a.transformation(), &Vec3::new(0.0f64, 0.014, 0.0));
        };

        w.set_light(StickToCamera);
    };
}
