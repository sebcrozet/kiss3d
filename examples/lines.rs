extern crate kiss3d;
extern crate nalgebra as na;

use kiss3d::light::Light;
use kiss3d::window::Window;
use na::{Point2, Point3};

fn main() {
    let mut window = Window::new("Kiss3d: lines");

    window.set_light(Light::StickToCamera);

    while window.render() {
        let a = Point3::new(-0.1, -0.1, 0.0);
        let b = Point3::new(0.0, 0.1, 0.0);
        let c = Point3::new(0.1, -0.1, 0.0);

        window.set_line_width(2.0);
        window.draw_line(&a, &b, &Point3::new(1.0, 0.0, 0.0));
        window.draw_line(&b, &c, &Point3::new(0.0, 1.0, 0.0));
        window.draw_line(&c, &a, &Point3::new(0.0, 0.0, 1.0));

        window.draw_planar_line(
            &Point2::new(-100.0, -200.0),
            &Point2::new(100.0, -200.0),
            &Point3::new(1.0, 1.0, 1.0),
        );
    }
}
