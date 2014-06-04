extern crate native;
extern crate ncollide = "ncollide3df32";
extern crate kiss3d;
extern crate nalgebra;

use nalgebra::na;
use nalgebra::na::{Vec3, Translation};
use ncollide::procedural::path::{PolylinePath, PolylinePattern, StrokePattern, ArrowheadCap};
use ncollide::procedural;
use ncollide::utils;
use kiss3d::window::Window;
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
        let mut c = window.add_trimesh(cube, na::one());
        c.append_translation(&Vec3::new(1.0, 0.0, 0.0));
        c.set_texture_from_file(&Path::new("media/kitten.png"), "kitten");

        /*
         * A sphere.
         */
        let sphere = procedural::sphere(&0.4f32, 20, 20);
        let mut s  = window.add_trimesh(sphere, na::one());
        s.set_texture_with_name("kitten");

        /*
         * A capsule.
         */
        let capsule = procedural::capsule(&0.4f32, &0.4f32, 20, 20);
        let mut c   = window.add_trimesh(capsule, na::one());
        c.append_translation(&Vec3::new(-1.0, 0.0, 0.0));
        c.set_color(0.0, 0.0, 1.0);

        /*
         * Triangulation.
         */
        let to_triangulate = utils::triangulate([
            Vec3::new(5.0f32, 0.0, 0.0), Vec3::new(6.1, 0.0, 0.5), Vec3::new(7.4, 0.0, 0.5), Vec3::new(8.2, 0.0, 0.0),
            Vec3::new(5.1f32, 1.0, 0.0), Vec3::new(6.2, 1.5, 0.5), Vec3::new(7.2, 1.0, 0.5), Vec3::new(8.0, 1.3, 0.0),
            Vec3::new(5.3f32, 2.0, 0.0), Vec3::new(6.1, 2.2, 0.5), Vec3::new(7.3, 2.0, 0.5), Vec3::new(8.2, 2.4, 0.0),
            Vec3::new(5.2f32, 3.0, 0.0), Vec3::new(6.1, 2.9, 0.5), Vec3::new(7.4, 3.0, 0.5), Vec3::new(8.0, 3.1, 0.0)
        ]);
        let mut t = window.add_trimesh(to_triangulate, na::one());
        t.set_surface_rendering_activation(false);
        t.set_lines_width(2.0);
        t.set_color(0.0, 1.0, 0.0);

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
        let mut b  = window.add_trimesh(bezier, na::one());
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

        /*
         * Path stroke.
         */
        let control_points = [
            Vec3::new(0.0f32, 1.0, 0.0),
            Vec3::new(2.0f32, 4.0, 2.0),
            Vec3::new(2.0f32, 1.0, 4.0),
            Vec3::new(4.0f32, 4.0, 6.0),
            Vec3::new(2.0f32, 1.0, 8.0),
            Vec3::new(2.0f32, 4.0, 10.0),
            Vec3::new(0.0f32, 1.0, 12.0),
            Vec3::new(-2.0f32, 4.0, 10.0),
            Vec3::new(-2.0f32, 1.0, 8.0),
            Vec3::new(-4.0f32, 4.0, 6.0),
            Vec3::new(-2.0f32, 1.0, 4.0),
            Vec3::new(-2.0f32, 4.0, 2.0),
        ];
        let bezier      = procedural::bezier_curve(control_points, 100);
        let mut path    = PolylinePath::new(&bezier);
        let pattern     = procedural::unit_circle(100);
        let start_cap   = ArrowheadCap::new(1.5f32, 2.0, 0.0);
        let end_cap     = ArrowheadCap::new(2.0f32, 2.0, 0.5);
        let mut pattern = PolylinePattern::new(&pattern, true, start_cap, end_cap);
        let mesh        = pattern.stroke(&mut path);
        let mut m = window.add_trimesh(mesh, Vec3::new(0.5f32, 0.5, 0.5));
        m.append_translation(&Vec3::new(4.0, -1.0, 0.0));
        m.set_color(1.0, 1.0, 0.0);

        /*
         *
         * Rendering.
         *
         */
        window.set_light(light::StickToCamera);

        window.render_loop(|_| {
        })
    })
}
