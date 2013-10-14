extern mod kiss3d;
extern mod nalgebra;

use nalgebra::na::{Vec3, Rotation};
use kiss3d::window;

#[start]
fn start(argc: int, argv: **u8) -> int {
    std::rt::start_on_main_thread(argc, argv, main)
}

fn main() {
    do window::Window::spawn("Kiss3d: texturing") |window| {
        let mut c = window.add_cube(1.0, 1.0, 1.0);

        c.set_color(1.0, 0.0, 0.0);
        c.set_texture("media/kitten.png");


        window.set_light(window::StickToCamera);

        do window.render_loop |_| {
            c.append_rotation(&Vec3::new(0.0f64, 0.014, 0.0))
        }
    }
}
