extern crate kiss3d;

use kiss3d::window::Window;

fn main() {
    for _ in 0..2 {
        let mut window = Window::new("Kiss3d: window");
        window.add_cube(1.0, 1.0, 1.0);
        window.render();
        window.close();
    }
}
