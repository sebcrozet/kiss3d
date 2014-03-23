use std::mem;
use nalgebra::na;
use nalgebra::na::{Cast, Vec3, Vec2};
use procedural::{MeshDescr, UnifiedIndexBuffer};

/// Generates a UV sphere.
pub fn sphere<N: Float + Cast<f64>>(center:        &Vec3<N>,
                                    radius:        &N,
                                    ntheta_subdiv: u32,
                                    nphi_subdiv:   u32)
                                    -> MeshDescr<N> {
    let mut sphere = unit_sphere(ntheta_subdiv, nphi_subdiv);

    sphere.scale_by_scalar(radius);
    sphere.translate_by(center);

    sphere
}

/// Generates a UV sphere centered at the origin and with a unit diameter.
// FIXME: n{theta,phi}_subdiv are not the right names.
pub fn unit_sphere<N: Float + Cast<f64>>(ntheta_subdiv: u32, nphi_subdiv: u32) -> MeshDescr<N> {
    let two_pi: N = Float::two_pi();
    let pi_two: N = Float::frac_pi_2();
    let dtheta    =  two_pi / na::cast(ntheta_subdiv as f64);
    let dphi      =  pi_two / na::cast(nphi_subdiv as f64);

    let mut coords     = Vec::new();
    let mut curr_phi   = -pi_two + dphi;

    // upper hemisphere (incl. equator)
    coords.push(-Vec3::y());

    for _ in range(0, 2 * nphi_subdiv - 1) {
        push_circle(curr_phi.cos(), ntheta_subdiv, dtheta, curr_phi.sin(), &mut coords);
        curr_phi = curr_phi + dphi;
    }

    coords.push(Vec3::y());

    // the normals are the same as the coords
    let normals = coords.clone();

    // index buffer
    let mut idx = Vec::new();
    push_degenerate_top_ring_indices(1, 0, ntheta_subdiv, &mut idx);

    for i in range(0, 2 * nphi_subdiv - 2) {
        push_ring_indices(1 + i * ntheta_subdiv, 1 + (i + 1) * ntheta_subdiv, ntheta_subdiv, &mut idx);
    }

    push_degenerate_top_ring_indices(1 + (2 * nphi_subdiv - 2) * ntheta_subdiv,
                                     coords.len() as u32 - 1,
                                     ntheta_subdiv,
                                     &mut idx);
    {
        let len = idx.len();
        reverce_clockwising(idx.mut_slice_from(len - ntheta_subdiv as uint));
    }

    // uvs
    let mut uvs = Vec::new();

    for coord in coords.iter() {
        uvs.push(ball_uv(coord));
    }

    // Result
    let mut out = MeshDescr::new(coords, Some(normals), Some(uvs), Some(UnifiedIndexBuffer(idx)));

    // set the radius to 0.5
    let _0_5: N = na::cast(0.5);
    out.scale_by_scalar(&_0_5);

    out
}

fn push_circle<N: Float>(radius: N, nsubdiv: u32, dtheta: N, y: N, out: &mut Vec<Vec3<N>>) {
    let mut curr_theta: N = na::zero();

    for _ in range(0, nsubdiv) {
        out.push(Vec3::new(curr_theta.cos() * radius, y.clone(), curr_theta.sin() * radius));
        curr_theta = curr_theta + dtheta;
    }
}

fn push_ring_indices(base_lower_circle: u32,
                     base_upper_circle: u32,
                     nsubdiv:           u32,
                     out:               &mut Vec<Vec3<u32>>) {
    assert!(nsubdiv > 0);

    for i in range(0, nsubdiv - 1) {
        let bli = base_lower_circle + i;
        let bui = base_upper_circle + i;
        out.push(Vec3::new(bli, bui, bui + 1));
        out.push(Vec3::new(bli, bui + 1, bli + 1));
    }

    // adjust the last two triangles
    out.push(Vec3::new(base_lower_circle + nsubdiv - 1, base_upper_circle + nsubdiv - 1, base_upper_circle));
    out.push(Vec3::new(base_lower_circle + nsubdiv - 1, base_upper_circle, base_lower_circle));
}

fn push_degenerate_top_ring_indices(base_circle: u32,
                                    point:       u32,
                                    nsubdiv:     u32,
                                    out:         &mut Vec<Vec3<u32>>) {
    assert!(nsubdiv > 0);

    for i in range(0, nsubdiv - 1) {
        out.push(Vec3::new(base_circle + i, point, base_circle + i + 1));
    }

    out.push(Vec3::new(base_circle + nsubdiv - 1, point, base_circle));
}

fn reverce_clockwising(indices: &mut [Vec3<u32>]) {
    for i in indices.mut_iter() {
        mem::swap(&mut i.x, &mut i.y);
    }
}

fn ball_uv<N: Float + Cast<f64>>(normal: &Vec3<N>) -> Vec2<N> {
    let two_pi: N = Float::two_pi();
    let pi:     N = Float::pi();
    let _0_5:   N = na::cast(0.5f64);
    let uvx       = _0_5 + normal.z.atan2(&normal.x) / two_pi;
    let uvy       = _0_5 - normal.y.asin() / pi;

    Vec2::new(uvx, uvy)
}
