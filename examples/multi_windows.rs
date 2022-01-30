extern crate kiss3d;

use kiss3d::window::Window;

fn main() {
    for _ in 0..5 {
        let mut window = Window::new("Kiss3d window");
        window.add_cube(1.0, 1.0, 1.0);
        let mut i = 0;
        while i < 100 && window.render() {
            i += 1;
        }
        window.close();
    }
}
