extern mod kiss3d;
extern mod nalgebra;

use nalgebra::na::{Vec3, Rotation};
use kiss3d::window;

#[start]
fn start(argc: int, argv: **u8) -> int {
    std::rt::start_on_main_thread(argc, argv, main)
}

fn main() {
    do window::Window::spawn("Kiss3d: wireframe") |window| {
        let mut c = window.add_cube(1.0, 1.0, 1.0);

        c.set_color(1.0, 0.0, 0.0);

        window.set_light(window::StickToCamera);
        window.set_framerate_limit(Some(60));

        let mut time = 0u;
        let mut on   = true;

        window.render_loop(|w| {
            if time % 200 == 0 {
                // Toogle the wireframe mode.
                w.set_wireframe_mode(on);
                on = !on;
            }

            time = time + 1;

            c.append_rotation(&Vec3::new(0.0f32, 0.014, 0.0))
        })
    }
}
