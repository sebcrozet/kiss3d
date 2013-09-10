extern mod kiss3d;
extern mod nalgebra;

use std::rand::random;
use kiss3d::window;

#[start]
fn start(argc: int, argv: **u8, crate_map: *u8) -> int {
    std::rt::start_on_main_thread(argc, argv, crate_map, main)
}

fn main() {
    do window::Window::spawn("Kiss3d: quad waves") |window| {
        let mut c = window.add_quad(5.0, 4.0, 500, 400);
        
        c.set_color(random(), random(), random());

        let mut time = 0.016f32;

        window.set_light(window::StickToCamera);

        do window.render_loop |_| {
            do c.modify_mesh |m| {
                for v in m.mut_coords().mut_iter() {
                    v.z = time.sin() * (((v.x + time) * 4.0).cos() +
                          time.sin() * ((v.y + time) * 4.0 + time).cos()) / 2.0
                }

                m.recompute_normals();

                true
            }

            time = time + 0.016;
        }
    }
}
