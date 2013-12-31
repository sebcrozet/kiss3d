extern mod kiss3d;
extern mod nalgebra;

use nalgebra::na::{Vec3, Rotation};
use kiss3d::window;
use kiss3d::mesh::{Mesh, StorageLocation};

fn main() {
    do window::Window::spawn("Kiss3d: cube") |window| {
        let a = Vec3::new(-1.0, -1.0, 0.0);
        let b = Vec3::new(1.0, -1.0, 0.0);
        let c = Vec3::new(0.0, 1.0, 0.0);

        let vertices = StorageLocation::new(~[a, b, c], false);
        let indices  = StorageLocation::new(~[Vec3::new(0u32, 1, 2)], false);

        let mesh = Mesh::new(vertices, indices, None, None, false);

        window.register_mesh("custom_mesh", mesh);

        let mut c1 = window.add("custom_mesh", 1.0).unwrap();
        let mut c2 = window.add("custom_mesh", 1.0).unwrap();

        c1.set_color(1.0, 0.0, 0.0);
        c2.set_color(0.0, 1.0, 0.0);

        window.set_light(window::StickToCamera);

        window.render_loop(|_| {
            c1.prepend_rotation(&Vec3::new(0.0f32, 0.014, 0.0));
            c2.prepend_rotation(&Vec3::new(0.0f32, -0.014, 0.0))
        })
    }
}
