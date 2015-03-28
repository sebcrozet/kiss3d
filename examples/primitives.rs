extern crate rand;
extern crate kiss3d;
extern crate nalgebra as na;

use rand::random;
use na::{Vec3, Translation, RotationWithTranslation};
use kiss3d::window::Window;
use kiss3d::light::Light;

fn main() {
    let mut window = Window::new("Kiss3d: primitives");

    let mut c = window.add_cube(1.0, 1.0, 1.0);
    let mut s = window.add_sphere(0.5);
    let mut p = window.add_cone(0.5, 1.0);
    let mut y = window.add_cylinder(0.5, 1.0);
    let mut a = window.add_capsule(0.5, 1.0);

    c.set_color(random(), random(), random());
    s.set_color(random(), random(), random());
    p.set_color(random(), random(), random());
    y.set_color(random(), random(), random());
    a.set_color(random(), random(), random());

    c.append_translation(&Vec3::new(2.0, 0.0, 0.0));
    s.append_translation(&Vec3::new(4.0, 0.0, 0.0));
    p.append_translation(&Vec3::new(-2.0, 0.0, 0.0));
    y.append_translation(&Vec3::new(-4.0, 0.0, 0.0));
    a.append_translation(&Vec3::new(0.0, 0.0, 0.0));

    window.set_light(Light::StickToCamera);

    while window.render() {
        c.append_rotation_wrt_center(&Vec3::new(0.0f32, 0.014, 0.0));
        s.append_rotation_wrt_center(&Vec3::new(0.0f32, 0.014, 0.0));
        p.append_rotation_wrt_center(&Vec3::new(0.0f32, 0.014, 0.0));
        y.append_rotation_wrt_center(&Vec3::new(0.0f32, 0.014, 0.0));
        a.append_rotation_wrt_center(&Vec3::new(0.0f32, 0.014, 0.0));
    }
}
