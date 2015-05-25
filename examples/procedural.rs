extern crate time;
extern crate rand;
extern crate ncollide_procedural;
extern crate ncollide_transformation;
extern crate kiss3d;
extern crate nalgebra as na;

use std::path::Path;
use na::{Pnt2, Pnt3, Vec2, Vec3, Translation};
use ncollide_procedural::{Polyline, TriMesh};
use ncollide_procedural::path::{PolylinePath, PolylinePattern, StrokePattern, ArrowheadCap};
use kiss3d::window::Window;
use kiss3d::light::Light;

fn main() {
    let mut window = Window::new("Kiss3d: procedural");

    /*
     * A cube.
     */
    let cube  = ncollide_procedural::cuboid(&Vec3::new(0.7f32, 0.2, 0.4));
    let mut c = window.add_trimesh(cube, na::one());
    c.append_translation(&Vec3::new(1.0, 0.0, 0.0));
    c.set_texture_from_file(&Path::new("media/kitten.png"), "kitten");

    /*
     * A sphere.
     */
    let sphere = ncollide_procedural::sphere(0.4f32, 20, 20, true);
    let mut s  = window.add_trimesh(sphere, na::one());
    s.set_texture_with_name("kitten");

    /*
     * A capsule.
     */
    let capsule = ncollide_procedural::capsule(&0.4f32, &0.4f32, 20, 20);
    let mut c   = window.add_trimesh(capsule, na::one());
    c.append_translation(&Vec3::new(-1.0, 0.0, 0.0));
    c.set_color(0.0, 0.0, 1.0);

    /*
     * Triangulation.
     */
    let to_triangulate = ncollide_transformation::triangulate(&[
        Pnt3::new(5.0f32, 0.0, 0.0), Pnt3::new(6.1, 0.0, 0.5), Pnt3::new(7.4, 0.0, 0.5), Pnt3::new(8.2, 0.0, 0.0),
        Pnt3::new(5.1f32, 1.0, 0.0), Pnt3::new(6.2, 1.5, 0.5), Pnt3::new(7.2, 1.0, 0.5), Pnt3::new(8.0, 1.3, 0.0),
        Pnt3::new(5.3f32, 2.0, 0.0), Pnt3::new(6.1, 2.2, 0.5), Pnt3::new(7.3, 2.0, 0.5), Pnt3::new(8.2, 2.4, 0.0),
        Pnt3::new(5.2f32, 3.0, 0.0), Pnt3::new(6.1, 2.9, 0.5), Pnt3::new(7.4, 3.0, 0.5), Pnt3::new(8.0, 3.1, 0.0)
    ]);
    let mut t = window.add_trimesh(to_triangulate, na::one());
    t.set_surface_rendering_activation(false);
    t.set_lines_width(2.0);
    t.set_color(0.0, 1.0, 0.0);

    /*
     * A (non-rational) bicubic Bézier surface.
     */
    let control_points = [
        Pnt3::new(0.0f32, 0.0, 0.0), Pnt3::new(1.0, 0.0, 2.0), Pnt3::new(2.0, 0.0, 2.0), Pnt3::new(3.0, 0.0, 0.0),
        Pnt3::new(0.0f32, 1.0, 2.0), Pnt3::new(1.0, 1.0, 3.0), Pnt3::new(2.0, 1.0, 3.0), Pnt3::new(3.0, 1.0, 2.0),
        Pnt3::new(0.0f32, 2.0, 2.0), Pnt3::new(1.0, 2.0, 3.0), Pnt3::new(2.0, 2.0, 3.0), Pnt3::new(3.0, 2.0, 2.0),
        Pnt3::new(0.0f32, 3.0, 0.0), Pnt3::new(1.0, 3.0, 2.0), Pnt3::new(2.0, 3.0, 2.0), Pnt3::new(3.0, 3.0, 0.0)
    ];
    let bezier = ncollide_procedural::bezier_surface(&control_points, 4, 4, 100, 100);
    let mut b  = window.add_trimesh(bezier, na::one());
    b.append_translation(&Vec3::new(-1.5, -1.5, 0.0));
    b.enable_backface_culling(false);

    // XXX: replace by an `add_mesh`.
    let mut control_polyhedra_gfx = window.add_quad_with_vertices(&control_points, 4, 4);
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
        Pnt3::new(0.0f32, 1.0, 0.0),
        Pnt3::new(2.0f32, 4.0, 2.0),
        Pnt3::new(2.0f32, 1.0, 4.0),
        Pnt3::new(4.0f32, 4.0, 6.0),
        Pnt3::new(2.0f32, 1.0, 8.0),
        Pnt3::new(2.0f32, 4.0, 10.0),
        Pnt3::new(0.0f32, 1.0, 12.0),
        Pnt3::new(-2.0f32, 4.0, 10.0),
        Pnt3::new(-2.0f32, 1.0, 8.0),
        Pnt3::new(-4.0f32, 4.0, 6.0),
        Pnt3::new(-2.0f32, 1.0, 4.0),
        Pnt3::new(-2.0f32, 4.0, 2.0),
    ];
    let bezier      = ncollide_procedural::bezier_curve(&control_points, 100);
    let mut path    = PolylinePath::new(&bezier);
    let pattern     = ncollide_procedural::unit_circle(100);
    let start_cap   = ArrowheadCap::new(1.5f32, 2.0, 0.0);
    let end_cap     = ArrowheadCap::new(2.0f32, 2.0, 0.5);
    let mut pattern = PolylinePattern::new(&pattern, true, start_cap, end_cap);
    let mesh        = pattern.stroke(&mut path);
    let mut m       = window.add_trimesh(mesh, Vec3::new(0.5f32, 0.5, 0.5));
    m.append_translation(&Vec3::new(4.0, -1.0, 0.0));
    m.set_color(1.0, 1.0, 0.0);

    /*
     * Convex hull of 100,000 random 3d points.
     */
    let mut points = Vec::new();
    for _ in 0usize .. 100000 {
        points.push(rand::random::<Pnt3<f32>>() * 2.0f32);
    }

    let chull  = ncollide_transformation::convex_hull3(&points[..]);
    let mut mhull = window.add_trimesh(chull, na::one());
    let mut mpts  = window.add_trimesh(TriMesh::new(points, None, None, None), na::one());
    mhull.append_translation(&Vec3::new(0.0, 2.0, -1.0));
    mhull.set_color(0.0, 1.0, 0.0);
    mhull.set_lines_width(2.0);
    mhull.set_surface_rendering_activation(false);
    mhull.set_points_size(10.0);
    mpts.set_color(0.0, 0.0, 1.0);
    mpts.append_translation(&Vec3::new(0.0, 2.0, -1.0));
    mpts.set_points_size(2.0);
    mpts.set_surface_rendering_activation(false);

    /*
     * Convex hull of 100,000 random 2d points.
     */
    let mut points = Vec::new();
    let origin     = Pnt2::new(3.0f32, 2.0);
    for _ in 0usize .. 100000 {
        points.push(origin + rand::random::<Vec2<f32>>() * 2.0f32);
    }

    let points   = &points[..];
    let polyline = ncollide_transformation::convex_hull2(points);

    /*
     *
     * Rendering.
     *
     */
    window.set_light(Light::StickToCamera);

    while window.render() {
        draw_polyline(&mut window, &polyline, points);
    }
}

fn draw_polyline(window: &mut Window, polyline: &Polyline<Pnt2<f32>>, points: &[Pnt2<f32>]) {
    for pt in polyline.coords[..].windows(2) {
        window.draw_line(&Pnt3::new(pt[0].x, pt[0].y, 0.0), &Pnt3::new(pt[1].x, pt[1].y, 0.0), &Pnt3::new(0.0, 1.0, 0.0));
    }

    let last = polyline.coords.len() - 1;
    window.draw_line(&Pnt3::new(polyline.coords[0].x, polyline.coords[0].y, 0.0),
                     &Pnt3::new(polyline.coords[last].x, polyline.coords[last].y, 0.0),
                     &Pnt3::new(0.0, 1.0, 0.0));

    for pt in points.iter() {
            window.draw_point(&Pnt3::new(pt.x, pt.y, 0.0), &Pnt3::new(0.0, 0.0, 1.0));
    }

    for pt in polyline.coords[..].iter() {
        window.draw_point(&Pnt3::new(pt.x, pt.y, 0.0), &Pnt3::new(1.0, 0.0, 0.0));
    }

}
