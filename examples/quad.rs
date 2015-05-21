extern crate rand;
extern crate num;
extern crate kiss3d;
extern crate nalgebra as na;

use num::Float;
use rand::random;
use kiss3d::window::Window;
use kiss3d::light::Light;

fn main() {
    let mut window = Window::new("Kiss3d: quad");

    let mut c = window.add_quad(5.0, 4.0, 500, 400);

    c.set_color(random(), random(), random());

    let mut time = 0.016f32;

    window.set_light(Light::StickToCamera);

    while window.render() {
        c.modify_vertices(&mut |coords| {
            for v in coords.iter_mut() {
                v.z = time.sin() * (((v.x + time) * 4.0).cos() +
                      time.sin() * ((v.y + time) * 4.0 + time).cos()) / 2.0
            }
        });
        c.recompute_normals();

        time = time + 0.016;
    }
}
