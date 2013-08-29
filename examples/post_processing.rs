extern mod kiss3d;
extern mod nalgebra;

use std::rand::random;
use nalgebra::traits::rotation::Rotation;
use nalgebra::traits::translation::Translation;
use nalgebra::vec::Vec3;
use kiss3d::window;
use kiss3d::post_processing::post_processing_effect::PostProcessingEffect;
use kiss3d::post_processing::waves::Waves;
use kiss3d::post_processing::grayscales::Grayscales;

fn main() {
    do window::Window::spawn("Kiss3d: cube") |window| {
        let c = window.add_cube(1.0, 1.0, 1.0).set_color(random(), random(), random());
        let b = window.add_sphere(0.5).set_color(random(), random(), random());
        let p = window.add_cone(1.0, 0.5).set_color(random(), random(), random());
        let y = window.add_cylinder(1.0, 0.5).set_color(random(), random(), random());
        let a = window.add_capsule(1.0, 0.5).set_color(random(), random(), random());

        c.transformation().translate_by(&Vec3::new(2.0, 0.0, 0.0));
        b.transformation().translate_by(&Vec3::new(4.0, 0.0, 0.0));
        p.transformation().translate_by(&Vec3::new(-2.0, 0.0, 0.0));
        y.transformation().translate_by(&Vec3::new(-4.0, 0.0, 0.0));
        a.transformation().translate_by(&Vec3::new(0.0, 0.0, 0.0));

        let counter = @mut 0u;

        let effects = [
            Some(@mut Waves::new() as @mut PostProcessingEffect),
            Some(@mut Grayscales::new() as @mut PostProcessingEffect),
            None
        ];

        do window.set_loop_callback {
            if *counter % 200 == 0 {
                window.set_post_processing_effect(effects[*counter / 20 % effects.len()]);
            }

            *counter = *counter + 1;
        }

        window.set_light(window::StickToCamera);
        window.set_framerate_limit(Some(60));
    }
}
