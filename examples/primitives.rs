extern crate native;
extern crate rand;
extern crate kiss3d;
extern crate nalgebra;

use rand::random;
use nalgebra::na::{Vec3, Translation, RotationWithTranslation};
use kiss3d::window::Window;
use kiss3d::light;

#[start]
fn start(argc: int, argv: **u8) -> int {
    native::start(argc, argv, main)
}

fn main() {
    Window::spawn("Kiss3d: primitives", |window| {
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

        c.append_translation(&Vec3::new(2.0, 0.0, 0.0));
        b.append_translation(&Vec3::new(4.0, 0.0, 0.0));
        p.append_translation(&Vec3::new(-2.0, 0.0, 0.0));
        y.append_translation(&Vec3::new(-4.0, 0.0, 0.0));
        a.append_translation(&Vec3::new(0.0, 0.0, 0.0));

        window.set_light(light::StickToCamera);

        window.render_loop(|_| {
            c.append_rotation_wrt_center(&Vec3::new(0.0f32, 0.014, 0.0));
            b.append_rotation_wrt_center(&Vec3::new(0.0f32, 0.014, 0.0));
            p.append_rotation_wrt_center(&Vec3::new(0.0f32, 0.014, 0.0));
            y.append_rotation_wrt_center(&Vec3::new(0.0f32, 0.014, 0.0));
            a.append_rotation_wrt_center(&Vec3::new(0.0f32, 0.014, 0.0));
        });
    })
}
