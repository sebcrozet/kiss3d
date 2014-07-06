extern crate native;
extern crate ncollide = "ncollide3df32";
extern crate kiss3d;
extern crate nalgebra;

use std::rand;
use nalgebra::na;
use nalgebra::na::{Vec2, Vec3, Translation};
use ncollide::parametric::ParametricSurface;
use ncollide::procedural::{Polyline, TriMesh};
use ncollide::procedural::path::{PolylinePath, PolylinePattern, StrokePattern, ArrowheadCap};
use ncollide::procedural;
use ncollide::utils;
use ncollide::utils::symbolic::{BivariateFn, sin, cos, u, v};
use kiss3d::window::{Window, RenderFrame};
use kiss3d::camera::ArcBall;
use kiss3d::light;

#[start]
fn start(argc: int, argv: *const *const u8) -> int {
    native::start(argc, argv, main)
}

fn main() {
    let mut window = Window::new("Kiss3d: procedural");

    /*
     * A cube.
     */
    let cube  = procedural::cuboid(&Vec3::new(0.7f32, 0.2, 0.4));
    let mut c = window.add_trimesh(cube, na::one());
    c.append_translation(&Vec3::new(1.0, 0.0, 0.0));
    c.set_texture_from_file(&Path::new("media/kitten.png"), "kitten");

    /*
     * A sphere.
     */
    let sphere = procedural::sphere(&0.4f32, 20, 20, true);
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
    let mut m       = window.add_trimesh(mesh, Vec3::new(0.5f32, 0.5, 0.5));
    m.append_translation(&Vec3::new(4.0, -1.0, 0.0));
    m.set_color(1.0, 1.0, 0.0);

    /*
     * Convex hull of 100,000 random 3d points.
     */
    let mut points = Vec::new();
    for _ in range(0u, 100000) {
        points.push(rand::random::<Vec3<f32>>() * 2.0f32);
    }

    let chull  = procedural::convex_hull3d(points.as_slice());
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
    let origin     = Vec2::new(3.0f32, 2.0);
    for _ in range(0u, 100000) {
        points.push(origin + rand::random::<Vec2<f32>>() * 2.0f32);
    }

    let points   = points.as_slice();
    let polyline = procedural::convex_hull2d(points);

    /*
     * Uniform parametric surface mesher.
     */
    let banana = ParametricBananas::new();
    let mesh   = procedural::parametric_surface_uniform(&banana, 100, 100);
    let mut m  = window.add_trimesh(mesh, Vec3::new(0.5, 0.5, 0.5));
    m.set_texture_from_file(&Path::new("media/banana.jpg"), "banana");
    m.append_translation(&Vec3::new(-3.5, 0.0, 0.0));

    /*
     *
     * Rendering.
     *
     */
    window.set_light(light::StickToCamera);

    for mut frame in window.iter() {
        draw_polyline(&mut frame, &polyline, points)
    }
}

fn draw_polyline(frame: &mut RenderFrame<ArcBall>, polyline: &Polyline<f32, Vec2<f32>>, points: &[Vec2<f32>]) {
    for pt in polyline.coords.as_slice().windows(2) {
        frame.draw_line(&Vec3::new(pt[0].x, pt[0].y, 0.0), &Vec3::new(pt[1].x, pt[1].y, 0.0), &Vec3::y());
    }

    let last = polyline.coords.len() - 1;
    frame.draw_line(&Vec3::new(polyline.coords.get(0).x, polyline.coords.get(0).y, 0.0),
                    &Vec3::new(polyline.coords.get(last).x, polyline.coords.get(last).y, 0.0),
                    &Vec3::y());

    for pt in points.iter() {
        frame.draw_point(&Vec3::new(pt.x, pt.y, 0.0), &Vec3::z());
    }

    for pt in polyline.coords.as_slice().iter() {
        frame.draw_point(&Vec3::new(pt.x, pt.y, 0.0), &Vec3::x());
    }

}

// see https://www.pacifict.com/Examples/Example22.html

struct ParametricBananas {
    // we use trait-objects because we really do not want to know the exact type…
    x: Box<BivariateFn<f32, f32>>,
    y: Box<BivariateFn<f32, f32>>,
    z: Box<BivariateFn<f32, f32>>
}

impl ParametricBananas {
    fn new() -> ParametricBananas {
        let pi: f32 = Float::pi();
        let u = u();
        let v = v();
        let x = (sin(u * 2.0f32 * pi) * sin(v * 2.0f32 * pi) + 2.0f32) * sin(v * 3.0f32 * pi);
        let y = sin(v * 2.0f32 * pi) * cos(u * 2.0f32 * pi) + v * 4.0f32 - 2.0f32;
        let z = (sin(u * 2.0f32 * pi) * sin(v * 2.0f32 * pi) + 2.0f32) * cos(v * 3.0f32 * pi);


        ParametricBananas {
            x: box x,
            y: box y,
            z: box z
        }
    }
}

impl ParametricSurface for ParametricBananas {
    fn at(&self, u: f32, v: f32) -> Vec3<f32> {
        Vec3::new(self.x.d0(u, v), self.y.d0(u, v), self.z.d0(u, v))
    }

    fn at_u(&self, u: f32, v: f32) -> Vec3<f32> {
        Vec3::new(self.x.du(u, v), self.y.du(u, v), self.z.du(u, v))
    }

    fn at_v(&self, u: f32, v: f32)  -> Vec3<f32> {
        Vec3::new(self.x.dv(u, v), self.y.dv(u, v), self.z.dv(u, v))
    }

    fn at_uu(&self, u: f32, v: f32) -> Vec3<f32> {
        Vec3::new(self.x.duu(u, v), self.y.duu(u, v), self.z.duu(u, v))
    }

    fn at_vv(&self, u: f32, v: f32) -> Vec3<f32> {
        Vec3::new(self.x.dvv(u, v), self.y.dvv(u, v), self.z.dvv(u, v))
    }

    fn at_uv(&self, u: f32, v: f32) -> Vec3<f32> {
        Vec3::new(self.x.duv(u, v), self.y.duv(u, v), self.z.duv(u, v))
    }

    fn at_uv_nk(&self, u: f32, v: f32, n: uint, k: uint) -> Vec3<f32> {
        Vec3::new(self.x.duv_nk(u, v, n, k), self.y.duv_nk(u, v, n, k), self.z.duv_nk(u, v, n, k))
    }
}
