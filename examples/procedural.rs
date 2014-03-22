extern crate native;
extern crate kiss3d;
extern crate nalgebra;

use nalgebra::na;
use nalgebra::na::{Vec3, Rotation};
use kiss3d::window::Window;
use kiss3d::resource::Mesh;
use kiss3d::procedural;
use kiss3d::light;

#[start]
fn start(argc: int, argv: **u8) -> int {
    native::start(argc, argv, main)
}

fn main() {
    Window::spawn("Kiss3d: procedural", proc(window) {
        let cube = procedural::cube(&na::zero(), &Vec3::new(0.7f32, 0.2, 0.4));
        let mesh = Mesh::from_mesh_desc(cube, false);

        let mut c = window.add_mesh(mesh, 1.0);

        c.set_color(1.0, 0.0, 0.0);

        window.set_light(light::StickToCamera);

        window.render_loop(|_| {
            c.prepend_rotation(&Vec3::new(0.0f32, 0.014, 0.0))
        })
    })
}
