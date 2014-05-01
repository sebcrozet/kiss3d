extern crate native;
extern crate kiss3d;
extern crate nalgebra;

use nalgebra::na::Vec3;
use kiss3d::window::Window;
use kiss3d::light;

#[start]
fn start(argc: int, argv: **u8) -> int {
    native::start(argc, argv, main)
}

fn main() {
    Window::spawn("Kiss3d: points", |window| {

        window.set_light(light::StickToCamera);

        window.render_loop(|w| {
            let a = Vec3::new(-0.1, -0.1, 0.0);
            let b = Vec3::new(0.0, 0.1, 0.0);
            let c = Vec3::new(0.1, -0.1, 0.0);

            w.draw_point(&a, &Vec3::x());
            w.draw_point(&b, &Vec3::y());
            w.draw_point(&c, &Vec3::z());
        })
    })
}
