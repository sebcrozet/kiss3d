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
    do Window::spawn("Kiss3d: scaled primitives") |window| {
        // NOTE: scaling is not possible.
        for i in range(0u, 11) {
            let dim: f32 = random::<f32>() / 2.0;
            let dim2 = dim / 2.0;

            let offset = i as f64 * 1.0 - 5.0;

            let mut cu = window.add_cube(dim2, dim2, dim2);
            let mut s  = window.add_sphere(dim2);
            let mut co = window.add_cone(dim, dim2);
            let mut cy = window.add_cylinder(dim, dim2);
            let mut ca = window.add_capsule(dim, dim2);

            na::translate_by(&mut cu, &na::vec3(offset, 1.0, 0.0));
            na::translate_by(&mut s, &na::vec3(offset, -1.0, 0.0));
            na::translate_by(&mut co, &na::vec3(offset, 2.0, 0.0));
            na::translate_by(&mut cy, &na::vec3(offset, -2.0, 0.0));
            na::translate_by(&mut ca, &na::vec3(offset, 0.0, 0.0));

            cu.set_color(random(), random(), random());
            s.set_color(random(), random(), random());
            co.set_color(random(), random(), random());
            cy.set_color(random(), random(), random());
            ca.set_color(random(), random(), random());
        }

        window.set_light(StickToCamera);

        do window.render_loop |w| {
            for o in w.objects_mut().mut_iter() {
                na::rotate_wrt_center(o, &na::vec3(0.0f64, 0.014, 0.0));
            }
        };
    };
}
