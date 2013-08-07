extern mod kiss3d;
extern mod nalgebra;

use nalgebra::vec::Vec3;
use kiss3d::window;

fn main() {
    do window::Window::spawn(~"Kiss3d: cube") |window| {

        do window.set_loop_callback {
            let a = Vec3::new(-0.5, -0.5, 0.0);
            let b = Vec3::new(0.0, 0.5, 0.0);
            let c = Vec3::new(0.5, -0.5, 0.0);

            window.draw_line(&a, &b, &Vec3::x());
            window.draw_line(&b, &c, &Vec3::y());
            window.draw_line(&c, &a, &Vec3::z());
        }

        window.set_light(window::StickToCamera);
    }
}
