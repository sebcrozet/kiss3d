extern mod kiss3d;
extern mod nalgebra;

use std::rand::random;
use nalgebra::na;
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

        na::translate_by(&mut c, &na::vec3(2.0, 0.0, 0.0));
        na::translate_by(&mut b, &na::vec3(4.0, 0.0, 0.0));
        na::translate_by(&mut p, &na::vec3(-2.0, 0.0, 0.0));
        na::translate_by(&mut y, &na::vec3(-4.0, 0.0, 0.0));
        na::translate_by(&mut a, &na::vec3(0.0, 0.0, 0.0));

        window.set_light(StickToCamera);

        do window.render_loop |_| {
            na::rotate_wrt_center(&mut c, &na::vec3(0.0f64, 0.014, 0.0));
            na::rotate_wrt_center(&mut b, &na::vec3(0.0f64, 0.014, 0.0));
            na::rotate_wrt_center(&mut p, &na::vec3(0.0f64, 0.014, 0.0));
            na::rotate_wrt_center(&mut y, &na::vec3(0.0f64, 0.014, 0.0));
            na::rotate_wrt_center(&mut a, &na::vec3(0.0f64, 0.014, 0.0));
        };
    };
}
