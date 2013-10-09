extern mod kiss3d;
extern mod nalgebra;
extern mod glfw;

//use nalgebra::mat::Rotation;
use nalgebra::na::Vec3;
use kiss3d::window;
use kiss3d::event::KeyReleased;
use kiss3d::camera::{Camera, FirstPersonStereo};
use kiss3d::post_processing::post_processing_effect::PostProcessingEffect;
use kiss3d::post_processing::grayscales::Grayscales;
use kiss3d::post_processing::waves::Waves;

#[start]
fn start(argc: int, argv: **u8) -> int {
    std::rt::start_on_main_thread(argc, argv, main)
}

fn main() {
    do window::Window::spawn_size("kiss3d_stereo", 1280, 800) |window| {
        let mut c = window.add_cube(1.0, 1.0, 1.0);
        //c.position(

        let eye = Vec3::new(00.0f64, 0.0, 10.0);
        let at  = Vec3::new(00.0f64, 0.0, 0.0);
        let camera = @mut FirstPersonStereo::new(eye, at, 0.3f64);
        window.set_camera(camera  as @mut Camera);

        // Position the window correctly. -6/-26 takes care of icewm default
        // window decoration. Should probably just disable decorations (since
        // the top title is obscured anyway).
        window.glfw_window().set_pos(-6, -26);
        c.set_color(1.0, 0.0, 0.0);

        window.set_light(window::StickToCamera);

        //let effect = Some(@mut OculusCorrection::new() as @mut PostProcessingEffect);
        let effect = Some(@mut Grayscales::new() as @mut PostProcessingEffect);
        window.set_post_processing_effect(effect);

        do window.render_loop |w| {
            //c.rotate_by(&Vec3::new(0.0f64, 0.014, 0.0))
            do w.poll_events |w, event| {
                match *event {
                    KeyReleased(key) => {
                        match key {
                            glfw::Key1 => {
                                //  cannot borrow `*camera` as immutable because it is also borrowed as mutable
                                let ipd = camera.ipd();
                                camera.set_ipd(ipd + 0.1);
                                println(fmt!("ipd = %f", camera.ipd() as f64));
                                true
                            },
                            glfw::Key2 => {
                                let ipd = camera.ipd();
                                camera.set_ipd(ipd - 0.1);
                                println(fmt!("ipd = %f", camera.ipd() as f64));
                                true
                            },
                            _ => {
                                false
                            }
                        }
                    }
                    _ => { true }
                }
            }
        }
    }
}
