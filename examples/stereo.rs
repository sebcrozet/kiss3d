extern mod kiss3d;
extern mod nalgebra;
extern mod glfw;

//use nalgebra::mat::Rotation;
use nalgebra::na::Vec3;
use kiss3d::window;
use kiss3d::light;
use kiss3d::event::KeyReleased;
use kiss3d::camera::{Camera, FirstPersonStereo};
use kiss3d::post_processing::{PostProcessingEffect, OculusStereo};
//use kiss3d::post_processing::grayscales::Grayscales;

fn main() {
    do window::Window::spawn_size("kiss3d_stereo", 1280, 800) |window| {
        let mut c = window.add_cube(1.0, 1.0, 1.0);

        let     eye                 = Vec3::new(0.0f32, 0.0, 10.0);
        let     at                  = Vec3::new(0.0f32, 0.0, 0.0);
        let mut first_person_stereo = FirstPersonStereo::new(eye, at, 0.3f32);
        window.set_camera(&mut first_person_stereo as &mut Camera);

        // Position the window correctly. -6/-26 takes care of icewm default
        // window decoration. Should probably just disable decorations (since
        // the top title is obscured anyway).
        window.glfw_window().set_pos(-6, -26);
        c.set_color(1.0, 0.0, 0.0);

        window.set_light(light::StickToCamera);

        let mut oculus_stereo = OculusStereo::new();
        window.set_post_processing_effect(Some(&mut oculus_stereo as &mut PostProcessingEffect));
        let mut using_shader = true;

        window.render_loop(|w| {
            fn update_ipd(camera: &mut FirstPersonStereo, val: f32) -> bool {
                let ipd = camera.ipd();
                camera.set_ipd(ipd + val);

                true
            }

            w.poll_events(|w, event| {
                match *event {
                    KeyReleased(key) => {
                        match key {
                            glfw::Key1 => {
                                update_ipd(&mut first_person_stereo, 0.1f32)
                            },
                            glfw::Key2 => {
                                update_ipd(&mut first_person_stereo, -0.1f32)
                            },
                            glfw::KeyS => {
                                using_shader = match using_shader {
                                    false =>  {
                                        w.set_post_processing_effect(Some(&mut oculus_stereo as &mut PostProcessingEffect));
                                        true
                                    },
                                    true => {
                                        w.set_post_processing_effect(None);
                                        false
                                    },
                                };
                                false
                            },
                            _ => {
                                true
                            },
                        }
                    }
                    _ => { true }
                }
            })
        })
    }
}
