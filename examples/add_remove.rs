extern mod kiss3d;
extern mod nalgebra;

use kiss3d::window;

#[start]
fn start(argc: int, argv: **u8) -> int {
    std::rt::start_on_main_thread(argc, argv, main)
}

fn main() {
    do window::Window::spawn("Kiss3d: cube") |window| {
        let mut c     = window.add_cube(1.0, 1.0, 1.0);
        let mut added = true;

        window.set_light(window::StickToCamera);

        window.render_loop(|w| {
            if added {
                w.remove(c.clone());
            }
            else {
                c = w.add_cube(1.0, 1.0, 1.0);
                c.set_color(1.0, 0.0, 0.0);
            }

            added = !added;
        });
    }
}
