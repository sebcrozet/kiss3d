extern crate kiss3d;

use kiss3d::window::Window;

fn main() {
    let mut window = Window::new("Kiss3d: window");

    window.set_background_color(0.0, 0.0, 0.3);

    while window.render() {
    }
}
