extern crate native;
extern crate kiss3d;
extern crate nalgebra;

use nalgebra::na;
use nalgebra::na::{Vec3, Translation};
use kiss3d::window::Window;
use kiss3d::procedural;
use kiss3d::light;

#[start]
fn start(argc: int, argv: **u8) -> int {
    native::start(argc, argv, main)
}

fn main() {
    Window::spawn("Kiss3d: procedural", |window| {
        /*
         * A cube.
         */
        let cube  = procedural::cube(&Vec3::new(0.7f32, 0.2, 0.4));
        let mut c = window.add_mesh_descr(cube, na::one());
        c.append_translation(&Vec3::new(1.0, 0.0, 0.0));
        c.set_texture_from_file(&Path::new("media/kitten.png"), "kitten");

        /*
         * A sphere.
         */
        let sphere = procedural::sphere(&0.4f32, 20, 20);
        let mut s  = window.add_mesh_descr(sphere, na::one());
        s.set_texture_with_name("kitten");

        /*
         * A capsule.
         */
        let capsule = procedural::capsule(&0.4f32, &0.4f32, 20, 20);
        let mut c   = window.add_mesh_descr(capsule, na::one());
        c.append_translation(&Vec3::new(-1.0, 0.0, 0.0));
        c.set_color(0.0, 0.0, 1.0);

        /*
         * A (non-rational) bicubic Bézier surface.
         */
        let control_points = [
            Vec3::new(0.0f32, 0.0, 0.0), Vec3::new(1.0, 0.0, 2.0), Vec3::new(2.0, 0.0, 2.0), Vec3::new(3.0, 0.0, 0.0),
            Vec3::new(0.0f32, 1.0, 2.0), Vec3::new(1.0, 1.0, 3.0), Vec3::new(2.0, 1.0, 3.0), Vec3::new(3.0, 1.0, 2.0),
            Vec3::new(0.0f32, 2.0, 2.0), Vec3::new(1.0, 2.0, 3.0), Vec3::new(2.0, 2.0, 3.0), Vec3::new(3.0, 2.0, 2.0),
            Vec3::new(0.0f32, 3.0, 0.0), Vec3::new(1.0, 3.0, 2.0), Vec3::new(2.0, 3.0, 2.0), Vec3::new(3.0, 3.0, 0.0)
        ];
        let bezier = procedural::bezier_surface(control_points, 4, 4, 100, 100);
        let mut b  = window.add_mesh_descr(bezier, na::one());
        b.append_translation(&Vec3::new(-1.5, -1.5, 0.0));
        b.enable_backface_culling(false);

        // XXX: replace by an `add_mesh`.
        let mut control_polyhedra_gfx = window.add_quad_with_vertices(control_points, 4, 4);
        control_polyhedra_gfx.append_translation(&Vec3::new(-1.5, -1.5, 0.0));
        control_polyhedra_gfx.set_color(0.0, 0.0, 1.0);
        control_polyhedra_gfx.set_surface_rendering_activation(false);
        control_polyhedra_gfx.set_lines_width(2.0);

        let mut control_points_gfx = window.add_mesh(control_polyhedra_gfx.data().get_object().mesh().clone(), na::one());
        control_points_gfx.append_translation(&Vec3::new(-1.5, -1.5, 0.0));
        control_points_gfx.set_color(1.0, 0.0, 0.0);
        control_points_gfx.set_surface_rendering_activation(false);
        control_points_gfx.set_points_size(10.0);

        window.set_light(light::StickToCamera);

        window.render_loop(|_| {
        })
    })
}
