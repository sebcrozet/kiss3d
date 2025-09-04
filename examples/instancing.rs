extern crate kiss3d;
extern crate nalgebra as na;

use kiss3d::light::Light;
use kiss3d::window::Window;
use na::{Matrix3, Point3, Translation3, UnitQuaternion, Vector3};
use kiss3d::scene::InstanceData;

fn main() {
    env_logger::init();
    let mut window = Window::new("Kiss3d: instancing 3D");
    let mut c = window.add_cube(1.0, 1.0, 1.0);

    // TODO: that API needs to be simplified!

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
                    )
                });
            }
        }
    }

    c.data_mut().get_object_mut().set_instances(&instances);

    window.set_light(Light::StickToCamera);

    let rot = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.014);

    while window.render() {
        c.prepend_to_local_rotation(&rot);
    }
}
