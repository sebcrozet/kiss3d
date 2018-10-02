extern crate kiss3d;
extern crate nalgebra as na;

use std::path::Path;
use na::{Vector3, UnitComplex, UnitQuaternion, Translation2};
use kiss3d::window::Window;
use kiss3d::light::Light;

const RECTANGLE_UVS: [[f32; 2]; 4] = [
        [0.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0],
        [1.0, 0.0],
    ];

fn main() {
    let mut window = Window::new("Kiss3d: texturing");

    let mut c = window.add_cube(1.0, 1.0, 1.0);
    c.set_color(1.0, 0.0, 0.0);
    c.set_texture_from_file(&Path::new("media/kitten.png"), "kitten");

    let mut r = window.add_rectangle(100.0, 100.0);
    r.append_translation(&Translation2::new(-100.0, -100.0));
    r.set_color(0.0, 0.0, 1.0);
    r.set_texture_from_memory(include_bytes!("media/kitten.png"), "kitten_mem");

    r.modify_uvs(&mut |points|{
        for(i, p) in points.iter_mut().enumerate() {
            p[0] = RECTANGLE_UVS[i][0];
            p[1] = RECTANGLE_UVS[i][1];
        }
    });

    window.set_light(Light::StickToCamera);

    let rot3d = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.014);
    let rot2d = UnitComplex::new(0.01);

    while window.render() {
        c.append_rotation(&rot3d);
        r.prepend_to_local_rotation(&rot2d)
    }
}
