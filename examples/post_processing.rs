extern mod kiss3d;
extern mod nalgebra;

use std::rand::random;
use nalgebra::na::{Vec3, Translation};
use kiss3d::window;
use kiss3d::post_processing::post_processing_effect::PostProcessingEffect;
use kiss3d::post_processing::waves::Waves;
use kiss3d::post_processing::grayscales::Grayscales;
use kiss3d::post_processing::sobel_edge_highlight::SobelEdgeHighlight;

fn main() {
    do window::Window::spawn("Kiss3d: cube") |window| {
        let mut c = window.add_cube(1.0, 1.0, 1.0);
        let mut b = window.add_sphere(0.5);
        let mut p = window.add_cone(1.0, 0.5);
        let mut y = window.add_cylinder(1.0, 0.5);
        let mut a = window.add_capsule(1.0, 0.5);

        c.append_translation(&Vec3::new(2.0, 0.0, 0.0));
        b.append_translation(&Vec3::new(4.0, 0.0, 0.0));
        p.append_translation(&Vec3::new(-2.0, 0.0, 0.0));
        y.append_translation(&Vec3::new(-4.0, 0.0, 0.0));
        a.append_translation(&Vec3::new(0.0, 0.0, 0.0));

        c.set_color(random(), random(), random());
        b.set_color(random(), random(), random());
        p.set_color(random(), random(), random());
        y.set_color(random(), random(), random());
        a.set_color(random(), random(), random());

        let mut sobel = SobelEdgeHighlight::new(4.0);
        let mut waves = Waves::new();
        let mut grays = Grayscales::new();

        window.set_background_color(1.0, 1.0, 1.0);
        window.set_light(window::StickToCamera);
        window.set_framerate_limit(Some(60));

        let mut time    = 0u;
        let mut counter = 0u;

        window.render_loop(|w| {
            if time % 200 == 0 {
                match counter {
                    0 => w.set_post_processing_effect(None),
                    1 => w.set_post_processing_effect(Some(&mut sobel as &mut PostProcessingEffect)),
                    2 => w.set_post_processing_effect(Some(&mut waves as &mut PostProcessingEffect)),
                    3 => w.set_post_processing_effect(Some(&mut grays as &mut PostProcessingEffect)),
                    _ => unreachable!()
                }

                time    = 0;
                counter = (counter + 1) % 4;
            }

            time = time + 1;
        })
    }
}
