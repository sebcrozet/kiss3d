extern mod kiss3d;
extern mod nalgebra;

use nalgebra::na::{Vec3, Rotation};
use kiss3d::window;

fn main() {
    do window::Window::spawn("Kiss3d: obj") |window| {
        let obj_path = Path::new("media/cube/cube.obj");
        let mtl_path = Path::new("media/cube");
        let mut cs   = window.add_obj(&obj_path, &mtl_path, 1.0);

        window.set_light(window::StickToCamera);

        window.render_loop(|_| {
            for c in cs.mut_iter() {
                c.prepend_rotation(&Vec3::new(0.0f32, 0.014, 0.0))
            }
        })
    }
}
