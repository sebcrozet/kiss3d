extern crate kiss3d;
extern crate nalgebra as na;

use kiss3d::window::Window;
use na::{Translation2, UnitComplex};

fn main() {
    let mut window = Window::new("Kiss3d: rectangle");
    let mut rect = window.add_rectangle(50.0, 150.0);
    let mut circ = window.add_circle(50.0);
    circ.append_translation(&Translation2::new(200.0, 0.0));

    rect.set_color(0.0, 1.0, 0.0);
    circ.set_color(0.0, 0.0, 1.0);

    let rot_rect = UnitComplex::new(0.014);
    let rot_circ = UnitComplex::new(-0.014);

    while window.render() {
        rect.prepend_to_local_rotation(&rot_rect);
        circ.append_rotation(&rot_circ);
    }
}
