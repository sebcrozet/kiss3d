extern crate kiss3d;
extern crate nalgebra as na;

use na::Point3;
use kiss3d::window::Window;
use kiss3d::light::Light;

fn main() {
    let mut window = Window::new("Kiss3d: lines");

    window.set_light(Light::StickToCamera);

    while window.render() {
        let a = Point3::new(-0.1, -0.1, 0.0);
        let b = Point3::new(0.0, 0.1, 0.0);
        let c = Point3::new(0.1, -0.1, 0.0);

        window.draw_line(&a, &b, &Point3::new(1.0, 0.0, 0.0));
        window.draw_line(&b, &c, &Point3::new(0.0, 1.0, 0.0));
        window.draw_line(&c, &a, &Point3::new(0.0, 0.0, 1.0));
    }
}
