extern crate kiss3d;
extern crate nalgebra as na;

use kiss3d::light::Light;
use kiss3d::scene::SceneNode;
use kiss3d::window::{State, Window};
use na::{Translation3, UnitQuaternion, Vector3};

struct AppState {
    nodes: [SceneNode; 5],
    rot: UnitQuaternion<f32>,
}

impl State for AppState {
    fn step(&mut self, _: &mut Window) {
        for node in &mut self.nodes {
            node.append_rotation_wrt_center(&self.rot);
        }
    }
}

fn main() {
    let mut window = Window::new("Kiss3d: primitives");

    let mut c = window.add_cube(1.0, 1.0, 1.0);
    let mut s = window.add_sphere(0.5);
    let mut p = window.add_cone(0.5, 1.0);
    let mut y = window.add_cylinder(0.5, 1.0);
    let mut a = window.add_capsule(0.5, 1.0);

    c.set_color(1.0, 0.0, 0.0);
    s.set_color(0.0, 1.0, 0.0);
    p.set_color(0.0, 0.0, 1.0);
    y.set_color(1.0, 1.0, 0.0);
    a.set_color(0.0, 1.0, 1.0);

    c.append_translation(&Translation3::new(2.0, 0.0, 5.0));
    s.append_translation(&Translation3::new(4.0, 0.0, 5.0));
    p.append_translation(&Translation3::new(-2.0, 0.0, 5.0));
    y.append_translation(&Translation3::new(-4.0, 0.0, 5.0));
    a.append_translation(&Translation3::new(0.0, 0.0, 5.0));

    window.set_light(Light::StickToCamera);

    let rot = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.014);
    let state = AppState {
        nodes: [c, s, p, y, a],
        rot,
    };

    window.render_loop(state)
}
