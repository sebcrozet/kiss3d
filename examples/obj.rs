extern mod kiss3d;
extern mod nalgebra;

use nalgebra::na::{Vec3, Rotation};
use kiss3d::window;
use kiss3d::light;

fn main() {
    do window::Window::spawn("Kiss3d: obj") |window| {
        let obj_path = Path::new("media/teapot/teapot.obj");
        let mtl_path = Path::new("media/teapot");
        let mut cs   = window.add_obj(&obj_path, &mtl_path, 0.001);

        window.set_light(light::StickToCamera);

        window.render_loop(|_| {
            for c in cs.mut_iter() {
                c.prepend_rotation(&Vec3::new(0.0f32, 0.014, 0.0))
            }
        })
    }
}
