extern crate native;
extern crate kiss3d;

use kiss3d::window::Window;

#[start]
fn start(argc: int, argv: *const *const u8) -> int {
    native::start(argc, argv, main)
}

fn main() {
    let mut window = Window::new("Kiss3d: window");

    window.set_background_color(0.0, 0.0, 0.3);

    while window.render() {
    }
}
