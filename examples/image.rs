extern crate kiss3d;
extern crate nalgebra as na;

use std::path::Path;

use na::Vec3;
use kiss3d::window::Window;
use kiss3d::light::Light;

// Based on cube example.
fn main() {
    let mut window = Window::new("Kiss3d: cube");
    let mut c      = window.add_cube(0.2, 0.2, 0.2);

    c.set_color(1.0, 0.0, 0.0);
    c.prepend_to_local_rotation(&Vec3::new(0.0f32, 0.785, 0.0));
    c.prepend_to_local_rotation(&Vec3::new(-0.6f32, 0.0, 0.0));

    window.set_light(Light::StickToCamera);

    while window.render() {
        let img = window.snap_image();
        let img_path = Path::new("cube.png");
        img.save(img_path).unwrap();
        break;
    }
}
