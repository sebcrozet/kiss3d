extern crate kiss3d;
extern crate nalgebra as na;

use std::f32;
use std::path::Path;
use na::Vec3;
use kiss3d::window::Window;
use kiss3d::light::Light;

fn main() {
    let mut window = Window::new("Kiss3d: obj");

    // Teapot
    let obj_path   = Path::new("media/teapot/teapot.obj");
    let mtl_path   = Path::new("media/teapot");
    let mut teapot = window.add_obj(&obj_path, &mtl_path, Vec3::new(0.001, 0.001, 0.001));
    teapot.append_translation(&Vec3::new(0.0, -0.05, -0.2));

    // Rust logo
    let obj_path = Path::new("media/rust_logo/rust_logo.obj");
    let mtl_path = Path::new("media/rust_logo");
    let mut rust = window.add_obj(&obj_path, &mtl_path, Vec3::new(0.05, 0.05, 0.05));
    rust.prepend_to_local_rotation(&Vec3::new(-1.0 * f32::consts::FRAC_PI_2, 0.0, 0.0));
    rust.set_color(0.0, 0.0, 1.0);

    window.set_light(Light::StickToCamera);

    while window.render() {
        teapot.prepend_to_local_rotation(&Vec3::new(0.0f32, 0.014, 0.0));
        rust.prepend_to_local_rotation(&Vec3::new(0.0f32, -0.014, 0.0));
    }
}
