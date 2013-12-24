extern mod kiss3d;
extern mod nalgebra;

use nalgebra::na::{Vec3, Rotation};
use kiss3d::window;

#[start]
fn start(argc: int, argv: **u8) -> int {
    std::rt::start_on_main_thread(argc, argv, main)
}

fn main() {
    do window::Window::spawn("Kiss3d: obj") |window| {
        let mut c = window.add_obj("media/Great Hall Model.obj", 1.0, false);

        c.set_color(1.0, 0.0, 0.0);

        window.set_light(window::StickToCamera);

        window.render_loop(|_| {
            c.append_rotation(&Vec3::new(0.0f32, 0.014, 0.0))
        })
    }
}
