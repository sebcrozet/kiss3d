extern crate native;
extern crate kiss3d;
extern crate nalgebra;

use nalgebra::na::Vec3;
use kiss3d::window::Window;
use kiss3d::light;

#[start]
fn start(argc: int, argv: *const *const u8) -> int {
    native::start(argc, argv, main)
}

fn main() {
    let mut window = Window::new("Kiss3d: points");

    window.set_light(light::StickToCamera);

    for mut frame in window.iter() {
        let a = Vec3::new(-0.1, -0.1, 0.0);
        let b = Vec3::new(0.0, 0.1, 0.0);
        let c = Vec3::new(0.1, -0.1, 0.0);

        frame.draw_point(&a, &Vec3::x());
        frame.draw_point(&b, &Vec3::y());
        frame.draw_point(&c, &Vec3::z());
    }
}
