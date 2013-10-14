extern mod glfw;
extern mod kiss3d;
extern mod nalgebra;

use nalgebra::na::Vec3;
use nalgebra::na;
use kiss3d::window;
use kiss3d::event::KeyReleased;
use kiss3d::camera::{Camera, ArcBall, FirstPerson};

#[start]
fn start(argc: int, argv: **u8) -> int {
    std::rt::start_on_main_thread(argc, argv, main)
}

fn main()
{
    do window::Window::spawn("Kiss3d: camera") |window|
    {
        window.set_light(window::StickToCamera);

        // Replace the default arc-ball camera so that we can control it
        let eye = Vec3::new(10.0f64, 10.0, 10.0);
        let at  = na::zero();
        let arc_ball     = @mut ArcBall::new(eye, at);
        let first_person = @mut FirstPerson::new(eye, at);

        window.set_camera(arc_ball as @mut Camera);

        do window.render_loop |w| {
            do w.poll_events |w, event| {
                match *event {
                    KeyReleased(key) => {
                        if key == glfw::Key1 {
                            w.set_camera(arc_ball as @mut Camera)
                        }
                        else {
                            w.set_camera(first_person as @mut Camera)
                        }
                    }
                    _ => { }
                }
                true
            }

            w.draw_line(&na::zero(), &Vec3::x(), &Vec3::x());
            w.draw_line(&na::zero(), &Vec3::y(), &Vec3::y());
            w.draw_line(&na::zero(), &Vec3::z(), &Vec3::z());

            let curr_yaw = arc_ball.yaw();

            // rotate the arc-ball camera
            arc_ball.set_yaw(curr_yaw + 0.05);
        }
    }
}
