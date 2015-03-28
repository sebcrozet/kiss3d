extern crate glfw;
extern crate kiss3d;
extern crate nalgebra as na;


use glfw::{Key, Action, WindowEvent};
use na::Pnt3;
use kiss3d::window::Window;
use kiss3d::camera::{ArcBall, FirstPerson};
use kiss3d::light::Light;

fn main() {
    let eye              = Pnt3::new(10.0f32, 10.0, 10.0);
    let at               = na::orig();
    let mut first_person = FirstPerson::new(eye, at);
    let mut arc_ball     = ArcBall::new(eye, at);
    let mut use_arc_ball = true;

    let mut window = Window::new("Kiss3d: camera");
    window.set_light(Light::StickToCamera);

    while !window.should_close() {
        // rotate the arc-ball camera.
        let curr_yaw = arc_ball.yaw();
        arc_ball.set_yaw(curr_yaw + 0.05);

        // update the current camera.
        for event in window.events().iter() {
            match event.value {
                WindowEvent::Key(key, _, Action::Release, _) => {
                    if key == Key::Num1 {
                        use_arc_ball = true
                    }
                    else if key == Key::Num2 {
                        use_arc_ball = false
                    }
                }
                _ => { }
            }
        }

        window.draw_line(&na::orig(), &Pnt3::new(1.0, 0.0, 0.0), &Pnt3::new(1.0, 0.0, 0.0));
        window.draw_line(&na::orig(), &Pnt3::new(0.0, 1.0, 0.0), &Pnt3::new(0.0, 1.0, 0.0));
        window.draw_line(&na::orig(), &Pnt3::new(0.0, 0.0, 1.0), &Pnt3::new(0.0, 0.0, 1.0));

        if use_arc_ball {
            window.render_with_camera(&mut arc_ball);
        }
        else {
            window.render_with_camera(&mut first_person);
        }
    }
}
