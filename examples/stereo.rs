extern crate glfw;
extern crate nalgebra as na;
extern crate kiss3d;

use glfw::{Key, Action, WindowEvent};
use na::Pnt3;
use kiss3d::window::Window;
use kiss3d::light::Light;
use kiss3d::camera::FirstPersonStereo;
use kiss3d::post_processing::OculusStereo;

fn main() {
    let mut window = Window::new_with_size("Kiss3d: stereo", 1280, 800);

    let mut c = window.add_cube(1.0, 1.0, 1.0);

    let eye        = Pnt3::new(0.0f32, 0.0, 10.0);
    let at         = Pnt3::new(0.0f32, 0.0, 0.0);
    let mut camera = FirstPersonStereo::new(eye, at, 0.3f32);

    // Position the window correctly. -6/-26 takes care of icewm default
    // window decoration. Should probably just disable decorations (since
    // the top title is obscured anyway).
    window.glfw_window_mut().set_pos(-6, -26);
    c.set_color(1.0, 0.0, 0.0);

    window.set_light(Light::StickToCamera);

    let mut oculus_stereo = OculusStereo::new();

    while window.render_with_camera_and_effect(&mut camera, &mut oculus_stereo) {
        for event in window.events().iter() {
            match event.value {
                WindowEvent::Key(Key::Num1, _, Action::Release, _) => {
                    let ipd = camera.ipd();
                    camera.set_ipd(ipd + 0.1f32);
                },
                WindowEvent::Key(Key::Num2, _, Action::Release, _) => {
                    let ipd = camera.ipd();
                    camera.set_ipd(ipd - 0.1f32);
                },
                _ => { }
            }
        }
    }
}
