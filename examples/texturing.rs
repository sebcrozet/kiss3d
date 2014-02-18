extern crate native;
extern crate kiss3d;
extern crate nalgebra;

use nalgebra::na::{Vec3, Rotation};
use kiss3d::window::Window;
use kiss3d::light;

#[start]
fn start(argc: int, argv: **u8) -> int {
    native::start(argc, argv, main)
}

fn main() {
    Window::spawn("Kiss3d: texturing", proc(window) {
        let mut c = window.add_cube(1.0, 1.0, 1.0);

        c.set_color(1.0, 0.0, 0.0);
        c.set_texture(&Path::new("media/kitten.png"), "kitten");

        window.set_light(light::StickToCamera);

        window.render_loop(|_| {
            c.append_rotation(&Vec3::new(0.0f32, 0.014, 0.0))
        })
    })
}
