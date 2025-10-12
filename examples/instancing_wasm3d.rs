extern crate kiss3d;
extern crate nalgebra as na;

use kiss3d::light::Light;
use kiss3d::scene::{InstanceData, SceneNode};
use kiss3d::window::{State, Window};
use na::{Matrix3, Point3, UnitQuaternion, Vector3};

struct AppState {
    c: SceneNode,
    rot: UnitQuaternion<f32>,
}

impl State for AppState {
    fn step(&mut self, _window: &mut Window) {
        self.c.prepend_to_local_rotation(&self.rot);
    }
}

fn main() {
    env_logger::init();
    let mut window = Window::new("Kiss3d: instancing 3D (WASM compatible)");
    let mut c = window.add_cube(1.0, 1.0, 1.0);
    let mut instances = vec![];

    for i in 0..100 {
        for j in 0..100 {
            for k in 0..100 {
                let ii = i as f32;
                let jj = j as f32;
                let kk = k as f32;
                instances.push(InstanceData {
                    position: Point3::new(ii, jj, kk) * 1.5,
                    color: [ii / 100.0, jj / 100.0, kk / 100.0 + 0.1, 1.0],
                    #[rustfmt::skip]
                    deformation: Matrix3::new(
                        1.0, ii * 0.004, kk * 0.004,
                        ii * 0.004, 1.0, jj * 0.004,
                        kk * 0.004, jj * 0.004, 1.0,
                    ),
                });
            }
        }
    }

    c.set_instances(&instances);

    window.set_light(Light::StickToCamera);

    let rot = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.014);

    let state = AppState { c, rot };

    window.render_loop(state);
}
