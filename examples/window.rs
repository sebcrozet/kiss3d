extern mod native;
extern mod kiss3d;

use kiss3d::window::Window;

#[start]
fn start(argc: int, argv: **u8) -> int {
    native::start(argc, argv, main)
}

fn main() {
    Window::spawn("Kiss3d: empty window", proc(window) {
        window.set_background_color(0.0, 0.0, 0.3);

        window.render_loop(|_| {})
    })
}
