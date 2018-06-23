extern crate kiss3d;
extern crate nalgebra as na;

use kiss3d::scene::SceneNode2;
use kiss3d::window::{State, Window};
use na::UnitComplex;

struct AppState {
    c: SceneNode2,
    rot: UnitComplex<f32>,
}

impl State for AppState {
    fn step(&mut self, _: &mut Window) {
        self.c.prepend_to_local_rotation(&self.rot)
    }
}

fn main() {
    let mut window = Window::new("Kiss3d: rectangle");
    let mut c = window.add_rectangle(100.0, 150.0);

    c.set_color(1.0, 0.0, 0.0);

    let rot = UnitComplex::new(0.014);
    let state = AppState { c, rot };

    window.render_loop(state)
}
