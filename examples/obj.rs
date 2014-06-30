extern crate native;
extern crate kiss3d;
extern crate nalgebra;

use nalgebra::na::Vec3;
use kiss3d::window::Window;
use kiss3d::light;

#[start]
fn start(argc: int, argv: *const *const u8) -> int {
    native::start(argc, argv, main)
}

fn main() {
    let mut window = Window::new("Kiss3d: obj");
    let obj_path   = Path::new("media/teapot/teapot.obj");
    let mtl_path   = Path::new("media/teapot");
    let mut o      = window.add_obj(&obj_path, &mtl_path, Vec3::new(0.001, 0.001, 0.001));

    window.set_light(light::StickToCamera);

    for _ in window.iter() {
        o.prepend_to_local_rotation(&Vec3::new(0.0f32, 0.014, 0.0))
    }
}
