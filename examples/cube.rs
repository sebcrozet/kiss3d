extern mod kiss3d;
extern mod nalgebra;

use nalgebra::na::{Vec3, Rotation};
use kiss3d::window;
use kiss3d::light;

fn main() {
    do window::Window::spawn("Kiss3d: cube") |window| {
        let mut c = window.add_cube(1.0, 1.0, 1.0);

        c.set_color(1.0, 0.0, 0.0);

        window.set_light(light::StickToCamera);

        window.render_loop(|_| {
            c.prepend_rotation(&Vec3::new(0.0f32, 0.014, 0.0))
        })
    }
}
