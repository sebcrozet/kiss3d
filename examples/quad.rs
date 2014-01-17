extern mod kiss3d;
extern mod nalgebra;

use std::rand::random;
use kiss3d::window;

fn main() {
    do window::Window::spawn("Kiss3d: quad waves") |window| {
        let mut c = window.add_quad(5.0, 4.0, 500, 400);
        
        c.set_color(random(), random(), random());

        let mut time = 0.016f32;

        window.set_light(window::StickToCamera);

        window.render_loop(|_| {
            let     m  = c.mesh();
            let mut bm = m.borrow().borrow_mut();

            {
                let coords = bm.get().coords();

                coords.write(|coords|
                    coords.write(|coords| {
                        for v in coords.mut_iter() {
                            v.z = time.sin() * (((v.x + time) * 4.0).cos() +
                                  time.sin() * ((v.y + time) * 4.0 + time).cos()) / 2.0
                        }
                    })
                );
            }

            bm.get().recompute_normals();

            time = time + 0.016;
        })
    }
}
