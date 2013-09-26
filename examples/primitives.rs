extern mod kiss3d;
extern mod nalgebra;

use std::rand::random;
use nalgebra::mat::{Translation, RotationWithTranslation};
use nalgebra::vec::Vec3;
use kiss3d::window::{Window, StickToCamera};

#[start]
fn start(argc: int, argv: **u8) -> int {
    std::rt::start_on_main_thread(argc, argv, main)
}

fn main() {
    do Window::spawn("Kiss3d: primitives") |window| {
        let mut c = window.add_cube(1.0, 1.0, 1.0);
        let mut b = window.add_sphere(0.5);
        let mut p = window.add_cone(1.0, 0.5);
        let mut y = window.add_cylinder(1.0, 0.5);
        let mut a = window.add_capsule(1.0, 0.5);

        c.set_color(random(), random(), random());
        b.set_color(random(), random(), random());
        p.set_color(random(), random(), random());
        y.set_color(random(), random(), random());
        a.set_color(random(), random(), random());

        c.translate_by(&Vec3::new(2.0, 0.0, 0.0));
        b.translate_by(&Vec3::new(4.0, 0.0, 0.0));
        p.translate_by(&Vec3::new(-2.0, 0.0, 0.0));
        y.translate_by(&Vec3::new(-4.0, 0.0, 0.0));
        a.translate_by(&Vec3::new(0.0, 0.0, 0.0));

        window.set_light(StickToCamera);

        do window.render_loop |_| {
            c.rotate_wrt_center(&Vec3::new(0.0f64, 0.014, 0.0));
            b.rotate_wrt_center(&Vec3::new(0.0f64, 0.014, 0.0));
            p.rotate_wrt_center(&Vec3::new(0.0f64, 0.014, 0.0));
            y.rotate_wrt_center(&Vec3::new(0.0f64, 0.014, 0.0));
            a.rotate_wrt_center(&Vec3::new(0.0f64, 0.014, 0.0));
        };
    };
}
