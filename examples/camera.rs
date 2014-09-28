extern crate native;
extern crate glfw;
extern crate kiss3d;
extern crate "nalgebra" as na;


use na::Vec3;
use kiss3d::window::Window;
use kiss3d::camera::{ArcBall, FirstPerson};
use kiss3d::light;

#[start]
fn start(argc: int, argv: *const *const u8) -> int {
    native::start(argc, argv, main)
}

fn main() {
    let eye              = Vec3::new(10.0f32, 10.0, 10.0);
    let at               = na::zero();
    let mut first_person = FirstPerson::new(eye, at);
    let mut arc_ball     = ArcBall::new(eye, at);
    let mut use_arc_ball = true;

    let mut window = Window::new("Kiss3d: camera");
    window.set_light(light::StickToCamera);

    while !window.should_close() {
        // rotate the arc-ball camera.
        let curr_yaw = arc_ball.yaw();
        arc_ball.set_yaw(curr_yaw + 0.05);

        // update the current camera.
        for event in window.events().iter() {
            match event.value {
                glfw::KeyEvent(key, _, glfw::Release, _) => {
                    if key == glfw::Key1 {
                        use_arc_ball = true
                    }
                    else if key == glfw::Key2 {
                        use_arc_ball = false
                    }
                }
                _ => { }
            }
        }

        window.draw_line(&na::zero(), &Vec3::x(), &Vec3::x());
        window.draw_line(&na::zero(), &Vec3::y(), &Vec3::y());
        window.draw_line(&na::zero(), &Vec3::z(), &Vec3::z());

        if use_arc_ball {
            window.render_with_camera(&mut arc_ball);
        }
        else {
            window.render_with_camera(&mut first_person);
        }
    }
}
