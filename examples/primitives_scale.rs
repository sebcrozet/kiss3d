extern mod kiss3d;
extern mod nalgebra;

use std::rand::random;
use nalgebra::mat;
use nalgebra::mat::Translation;
use nalgebra::vec::Vec3;
use kiss3d::window::{Window, StickToCamera};

#[start]
fn start(argc: int, argv: **u8, crate_map: *u8) -> int {
    std::rt::start_on_main_thread(argc, argv, crate_map, main)
}

fn main() {
    do Window::spawn("Kiss3d: scaled primitives") |w| {
        // NOTE: scaling is not possible.
        for i in range(0u, 11) {
            let dim: f32 = random::<f32>() / 2.0;
            let dim2 = dim / 2.0;

            let offset = i as f64 * 1.0 - 5.0;

            w.add_cube(dim2, dim2, dim2).set_color(random(), random(), random())
                                        .transformation()
                                        .translate_by(&Vec3::new(offset, 1.0, 0.0));

            w.add_sphere(dim2).set_color(random(), random(), random())
                              .transformation()
                              .translate_by(&Vec3::new(offset, -1.0, 0.0));

            w.add_cone(dim, dim2).set_color(random(), random(), random())
                                 .transformation()
                                 .translate_by(&Vec3::new(offset, 2.0, 0.0));

            w.add_cylinder(dim, dim2).set_color(random(), random(), random())
                                     .transformation()
                                     .translate_by(&Vec3::new(offset, -2.0, 0.0));

            w.add_capsule(dim, dim2).set_color(random(), random(), random())
                                    .transformation()
                                    .translate_by(&Vec3::new(offset, 0.0, 0.0));
        }

        do w.set_loop_callback {
            for o in w.objects().iter() {
                mat::rotate_wrt_center(o.transformation(), &Vec3::new(0.0f64, 0.014, 0.0));
            }
        };

        w.set_light(StickToCamera);
    };
}
