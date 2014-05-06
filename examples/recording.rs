extern crate native;
extern crate kiss3d;
extern crate nalgebra;

use nalgebra::na::Vec3;
use kiss3d::window::Window;
use kiss3d::light;
use kiss3d::utils::Recorder;

#[start]
fn start(argc: int, argv: **u8) -> int {
    native::start(argc, argv, main)
}

fn main() {
    Window::spawn("Kiss3d: recording", |window| {
        let mut c = window.add_cone(0.5, 1.0);

        c.set_color(1.0, 0.0, 0.0);

        window.set_light(light::StickToCamera);

        let mut recorder = Recorder::new(Path::new("test.mpg"),
                                         window.width()  as uint,
                                         window.height() as uint);

        window.render_loop(|window| {
            c.prepend_to_local_rotation(&Vec3::new(0.0f32, 0.014, 0.0));

            recorder.snap(window);
        })
    })
}
