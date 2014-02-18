extern crate native;
extern crate kiss3d;
extern crate nalgebra;

use nalgebra::na::{Vec3, Rotation};
use kiss3d::window::Window;
use kiss3d::light;

#[start]
fn start(argc: int, argv: **u8) -> int {
    native::start(argc, argv, main)
}

fn main() {
    Window::spawn("Kiss3d: obj", proc(window) {
        let obj_path = Path::new("media/teapot/teapot.obj");
        let mtl_path = Path::new("media/teapot");
        let mut cs   = window.add_obj(&obj_path, &mtl_path, 0.001).unwrap();

        window.set_light(light::StickToCamera);

        window.render_loop(|_| {
            for c in cs.mut_iter() {
                c.prepend_rotation(&Vec3::new(0.0f32, 0.014, 0.0))
            }
        })
    })
}
