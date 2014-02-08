extern mod native;
extern mod kiss3d;
extern mod nalgebra;

use std::rand::random;
use kiss3d::window::Window;
use kiss3d::light;

#[start]
fn start(argc: int, argv: **u8) -> int {
    native::start(argc, argv, main)
}

fn main() {
    Window::spawn("Kiss3d: quad waves", proc(window) {
        let mut c = window.add_quad(5.0, 4.0, 500, 400);
        
        c.set_color(random(), random(), random());

        let mut time = 0.016f32;

        window.set_light(light::StickToCamera);

        window.render_loop(|_| {
            c.modify_vertices(|coords| {
                for v in coords.mut_iter() {
                    v.z = time.sin() * (((v.x + time) * 4.0).cos() +
                          time.sin() * ((v.y + time) * 4.0 + time).cos()) / 2.0
                }
            });
            c.recompute_normals();

            time = time + 0.016;
        })
    })
}
