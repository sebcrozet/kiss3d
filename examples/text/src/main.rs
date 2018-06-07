extern crate kiss3d;
extern crate nalgebra as na;

use std::rc::Rc;

use kiss3d::light::Light;
use kiss3d::scene::SceneNode;
use kiss3d::text::Font;
use kiss3d::window::{State, Window};
use na::{Point2, Point3, Translation3, UnitQuaternion, Vector3};
use std::path::Path;

struct AppState {
    font: Rc<Font>,
}

impl State for AppState {
    fn step(&mut self, window: &mut Window) {
        window.draw_text(
            "Hello birds!",
            &Point2::origin(),
            120.0,
            &mut self.font,
            &Point3::new(0.0, 1.0, 1.0),
        );
        let ascii = " !\"#$%&'`()*+,-_./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^abcdefghijklmnopqrstuvwxyz{|}~";
        window.draw_text(
            ascii,
            &Point2::new(0.0, 120.0),
            60.0,
            &mut self.font,
            &Point3::new(1.0, 1.0, 0.0),
        );
    }
}

fn main() {
    let mut window = Window::new("Kiss3d: text");

    window.set_light(Light::StickToCamera);

    let font = Font::default();
    let state = AppState { font };

    window.render_loop(state)
}
