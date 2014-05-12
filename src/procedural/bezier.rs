use nalgebra::na;
use nalgebra::na::{Cast, Vec3};
use procedural::MeshDescr;
use procedural;

// De-Casteljau algorithm.
fn bezier_curve_at<N: Float + Clone + Cast<f64>>(
                   control_points: &[Vec3<N>],
                   cache:          &mut Vec<Vec3<N>>,
                   t:              &N)
                   -> Vec3<N> {
    cache.grow_set(control_points.len() - 1, &na::zero(), na::zero());

    let cache = cache.as_mut_slice();

    let _1: N = na::mem(1.0);
    let t_1   = _1 - *t;

    unsafe {
        cache.as_mut_slice().copy_memory(control_points);
    }

    for i in range(1u, control_points.len()) {
        for j in range(0u, control_points.len() - i) {
            // XXX cache[j] = cache[j] * t_1 + cache[j + 1] * *t;
            cache[j].x = cache[j].x * t_1 + cache[j + 1].x * *t;
            cache[j].y = cache[j].y * t_1 + cache[j + 1].y * *t;
            cache[j].z = cache[j].z * t_1 + cache[j + 1].z * *t;
        }
    }

    cache[0].clone()
}

fn bezier_surface_at<N: Float + Clone + Cast<f64>>(
                     control_points: &[Vec3<N>],
                     ucache:         &mut Vec<Vec3<N>>,
                     vcache:         &mut Vec<Vec3<N>>,
                     nupoints:       uint,
                     nvpoints:       uint,
                     u:              &N,
                     v:              &N)
                     -> Vec3<N> {
    vcache.grow_set(nvpoints - 1, &na::zero(), na::zero());

    // FIXME: start with u or v, depending on which dimension has more control points.
    let vcache = vcache.as_mut_slice();

    for i in range(0, nvpoints) {
        let start = i * nupoints;
        let end   = start + nupoints;

        vcache[i] = bezier_curve_at(control_points.slice(start, end), ucache, u);
    }

    bezier_curve_at(vcache.slice(0, nvpoints), ucache, v)
}

/// Given a set of control points, generates a (non-rational) Bezier surface.
pub fn bezier_surface<N: Float + Clone + Cast<f64>>(
                      control_points: &[Vec3<N>],
                      nupoints:       uint,
                      nvpoints:       uint,
                      usubdivs:       uint,
                      vsubdivs:       uint)
                      -> MeshDescr<N> {
    assert!(nupoints * nvpoints == control_points.len());

    let mut surface = procedural::unit_quad(usubdivs, vsubdivs);

    {
        let uvs    = surface.uvs.as_ref().unwrap().as_slice();
        let coords = surface.coords.as_mut_slice();

        let mut ucache = Vec::new();
        let mut vcache = Vec::new();

        for j in range(0, vsubdivs + 1) {
            for i in range(0, usubdivs + 1) {
                let id = i + j * (usubdivs + 1);
                coords[id] = bezier_surface_at(control_points,
                                               &mut ucache,
                                               &mut vcache,
                                               nupoints,
                                               nvpoints,
                                               &uvs[id].x,
                                               &uvs[id].y)
            }
        }

        // XXX: compute the normals manually.
        surface.normals = None;
    }

    surface
}
