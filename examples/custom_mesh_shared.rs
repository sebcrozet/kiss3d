extern crate native;
extern crate kiss3d;
extern crate nalgebra;

use nalgebra::na::{Vec3, Rotation};
use kiss3d::window::Window;
use kiss3d::resource::Mesh;
use kiss3d::light;

#[start]
fn start(argc: int, argv: **u8) -> int {
    native::start(argc, argv, main)
}

fn main() {
    Window::spawn("Kiss3d: cube", proc(window) {
        let a = Vec3::new(-1.0, -1.0, 0.0);
        let b = Vec3::new(1.0, -1.0, 0.0);
        let c = Vec3::new(0.0, 1.0, 0.0);

        let vertices = vec!(a, b, c);
        let indices  = vec!(Vec3::new(0u32, 1, 2));

        let mesh = Mesh::new(vertices, indices, None, None, false);

        window.register_mesh("custom_mesh", mesh);

        let mut c1 = window.add("custom_mesh", 1.0).unwrap();
        let mut c2 = window.add("custom_mesh", 1.0).unwrap();

        c1.set_color(1.0, 0.0, 0.0);
        c2.set_color(0.0, 1.0, 0.0);

        window.set_light(light::StickToCamera);

        window.render_loop(|_| {
            c1.prepend_rotation(&Vec3::new(0.0f32, 0.014, 0.0));
            c2.prepend_rotation(&Vec3::new(0.0f32, -0.014, 0.0))
        })
    })
}
