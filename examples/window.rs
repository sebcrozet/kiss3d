extern mod kiss3d;

use kiss3d::window::Window;

fn main() {
    Window::spawn("Kiss3d: empty window", proc(window) {
        window.set_background_color(0.0, 0.0, 0.3);

        window.render_loop(|_| {})
    })
}
