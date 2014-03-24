extern crate native;
extern crate kiss3d;
extern crate nalgebra;

use nalgebra::na::{Vec3, Translation};
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
        let cube  = procedural::cube(&Vec3::new(0.7f32, 0.2, 0.4));
        let mesh  = Mesh::from_mesh_desc(cube, false);
        let mut c = window.add_mesh(mesh, 1.0);
        c.append_translation(&Vec3::new(1.0, 0.0, 0.0));
        c.set_texture(&Path::new("media/kitten.png"), "kitten");

        let sphere = procedural::sphere(&0.4f32, 20, 20);
        let mesh   = Mesh::from_mesh_desc(sphere, false);
        let mut s  = window.add_mesh(mesh, 1.0);
        s.set_texture_with_name("kitten");

        let capsule = procedural::capsule(&0.4f32, &0.4f32, 20, 20);
        let mesh    = Mesh::from_mesh_desc(capsule, false);
        let mut c   = window.add_mesh(mesh, 1.0);
        c.append_translation(&Vec3::new(-1.0, 0.0, 0.0));
        c.set_color(0.0, 0.0, 1.0);

        window.set_light(light::StickToCamera);

        window.render_loop(|_| {
        })
    })
}
