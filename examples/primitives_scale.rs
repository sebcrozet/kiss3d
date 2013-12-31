extern mod kiss3d;
extern mod nalgebra;

use std::rand::random;
use nalgebra::na::{Vec3, Translation, RotationWithTranslation};
use kiss3d::window::{Window, StickToCamera};

fn main() {
    do Window::spawn("Kiss3d: scaled primitives") |window| {
        // NOTE: scaling is not possible.
        for i in range(0u, 11) {
            let dim: f32 = random::<f32>() / 2.0;
            let dim2 = dim / 2.0;

            let offset = i as f32 * 1.0 - 5.0;

            let mut cu = window.add_cube(dim2, dim2, dim2);
            let mut sp = window.add_sphere(dim2);
            let mut co = window.add_cone(dim, dim2);
            let mut cy = window.add_cylinder(dim, dim2);
            let mut ca = window.add_capsule(dim, dim2);

            cu.append_translation(&Vec3::new(offset, 1.0, 0.0));
            sp.append_translation(&Vec3::new(offset, -1.0, 0.0));
            co.append_translation(&Vec3::new(offset, 2.0, 0.0));
            cy.append_translation(&Vec3::new(offset, -2.0, 0.0));
            ca.append_translation(&Vec3::new(offset, 0.0, 0.0));

            cu.set_color(random(), random(), random());
            sp.set_color(random(), random(), random());
            co.set_color(random(), random(), random());
            cy.set_color(random(), random(), random());
            ca.set_color(random(), random(), random());
        }

        window.set_light(StickToCamera);

        window.render_loop(|w| {
            for o in w.objects_mut().mut_iter() {
                o.append_rotation_wrt_center(&Vec3::new(0.0f32, 0.014, 0.0));
            }
        });
    };
}
