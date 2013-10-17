extern mod kiss3d;
extern mod nalgebra;
extern mod glfw;

//use nalgebra::mat::Rotation;
use nalgebra::na::Vec3;
use kiss3d::window;
use kiss3d::event::KeyReleased;
use kiss3d::camera::{Camera, FirstPersonStereo};
use kiss3d::post_processing::post_processing_effect::PostProcessingEffect;
use kiss3d::post_processing::oculus_stereo::OculusStereo;
//use kiss3d::post_processing::grayscales::Grayscales;

#[start]
fn start(argc: int, argv: **u8) -> int {
    std::rt::start_on_main_thread(argc, argv, main)
}

fn main() {
    do window::Window::spawn_size("kiss3d_stereo", 1280, 800) |window| {
        let mut c = window.add_cube(1.0, 1.0, 1.0);
        //c.position(

        let eye = Vec3::new(00.0f32, 0.0, 10.0);
        let at  = Vec3::new(00.0f32, 0.0, 0.0);
        let first_person_stereo = @mut FirstPersonStereo::new(eye, at, 0.3f32);
        let camera = first_person_stereo as @mut Camera;
        window.set_camera(camera);

        // Position the window correctly. -6/-26 takes care of icewm default
        // window decoration. Should probably just disable decorations (since
        // the top title is obscured anyway).
        window.glfw_window().set_pos(-6, -26);
        c.set_color(1.0, 0.0, 0.0);

        window.set_light(window::StickToCamera);

        let effect = Some(@mut OculusStereo::new() as @mut PostProcessingEffect);
        //let effect = Some(@mut Grayscales::new() as @mut PostProcessingEffect);
        window.set_post_processing_effect(effect);
        let mut using_shader = true;

        do window.render_loop |w| {
            //c.rotate_by(&Vec3::new(0.0f32, 0.014, 0.0))
            fn update_ipd(camera: @mut FirstPersonStereo, val: f32) -> bool {
                //  cannot borrow `*camera` as immutable because it is also borrowed as mutable
                let ipd = camera.ipd();
                camera.set_ipd(ipd + val);

                true
            }
            do w.poll_events |w, event| {
                match *event {
                    KeyReleased(key) => {
                        match key {
                            glfw::Key1 => {
                                update_ipd(first_person_stereo, 0.1f32)
                            },
                            glfw::Key2 => {
                                update_ipd(first_person_stereo, -0.1f32)
                            },
                            glfw::KeyS => {
                                using_shader = match using_shader {
                                    false =>  {
                                        w.set_post_processing_effect(effect);
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
            }
        }
    }
}
