extern crate native;
extern crate kiss3d;
extern crate nalgebra;

use kiss3d::window::Window;
use kiss3d::light;

#[start]
fn start(argc: int, argv: *const *const u8) -> int {
    native::start(argc, argv, main)
}

fn main() {
    let mut window = Window::new("Kiss3d: add_remove");
    let mut c      = window.add_cube(1.0, 1.0, 1.0);
    let mut added  = true;

    window.set_light(light::StickToCamera);

    for mut frame in window.iter() {
        if added {
            frame.window().remove(&mut c);
        }
        else {
            c = frame.window().add_cube(1.0, 1.0, 1.0);
            c.set_color(1.0, 0.0, 0.0);
        }

        added = !added;
    }
}
