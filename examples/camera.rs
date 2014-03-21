extern crate native;
extern crate glfw;
extern crate kiss3d;
extern crate nalgebra;


use nalgebra::na::Vec3;
use nalgebra::na;
use kiss3d::window::Window;
use kiss3d::camera::{Camera, ArcBall, FirstPerson};
use kiss3d::light;

#[start]
fn start(argc: int, argv: **u8) -> int {
    native::start(argc, argv, main)
}

fn main()
{
    Window::spawn("Kiss3d: camera", proc(window) {
        window.set_light(light::StickToCamera);

        // Replace the default arc-ball camera so that we can control it
        let eye              = Vec3::new(10.0f32, 10.0, 10.0);
        let at               = na::zero();
        let mut arc_ball     = ArcBall::new(eye, at);
        let mut first_person = FirstPerson::new(eye, at);

        window.set_camera(&mut arc_ball as &mut Camera);

        window.render_loop(|w| {
            w.poll_events(|w, event| {
                match *event {
                    glfw::KeyEvent(key, _, glfw::Release, _) => {
                        if key == glfw::Key1 {
                            w.set_camera(&mut arc_ball as &mut Camera)
                        }
                        else if key == glfw::Key2 {
                            w.set_camera(&mut first_person as &mut Camera)
                        }
                    }
                    _ => { }
                }
                true
            });

            w.draw_line(&na::zero(), &Vec3::x(), &Vec3::x());
            w.draw_line(&na::zero(), &Vec3::y(), &Vec3::y());
            w.draw_line(&na::zero(), &Vec3::z(), &Vec3::z());

            let curr_yaw = arc_ball.yaw();

            // rotate the arc-ball camera
            arc_ball.set_yaw(curr_yaw + 0.05);
        });
    })
}
