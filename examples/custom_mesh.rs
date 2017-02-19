extern crate kiss3d;
extern crate nalgebra as na;

use std::rc::Rc;
use std::cell::RefCell;
use na::{Point3, Vector3, UnitQuaternion};
use kiss3d::window::Window;
use kiss3d::resource::Mesh;
use kiss3d::light::Light;

fn main() {
    let mut window = Window::new("Kiss3d: custom_mesh");

    let a = Point3::new(-1.0, -1.0, 0.0);
    let b = Point3::new(1.0, -1.0, 0.0);
    let c = Point3::new(0.0, 1.0, 0.0);

    let vertices = vec!(a, b, c);
    let indices  = vec!(Point3::new(0u32, 1, 2));

    let mesh  = Rc::new(RefCell::new(Mesh::new(vertices, indices, None, None, false)));
    let mut c = window.add_mesh(mesh, Vector3::new(1.0, 1.0, 1.0));

    c.set_color(1.0, 0.0, 0.0);
    c.enable_backface_culling(false);

    window.set_light(Light::StickToCamera);

    let rot = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.014);

    while window.render() {
        c.prepend_to_local_rotation(&rot);
    }
}
