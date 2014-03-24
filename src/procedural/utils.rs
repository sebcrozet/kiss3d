use std::mem;
use nalgebra::na;
use nalgebra::na::Vec3;

/// Pushes a discretized circle to a buffer.
pub fn push_circle<N: Float>(radius: N, nsubdiv: u32, dtheta: N, y: N, out: &mut Vec<Vec3<N>>) {
    let mut curr_theta: N = na::zero();

    for _ in range(0, nsubdiv) {
        out.push(Vec3::new(curr_theta.cos() * radius, y.clone(), curr_theta.sin() * radius));
        curr_theta = curr_theta + dtheta;
    }
}

/// Creates the faces from two circles with the same discretization.
pub fn push_ring_indices(base_lower_circle: u32,
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

/// Creates the faces from a circle and a point that is shared by all triangle.
pub fn push_degenerate_top_ring_indices(base_circle: u32,
                                        point:       u32,
                                        nsubdiv:     u32,
                                        out:         &mut Vec<Vec3<u32>>) {
    assert!(nsubdiv > 0);

    for i in range(0, nsubdiv - 1) {
        out.push(Vec3::new(base_circle + i, point, base_circle + i + 1));
    }

    out.push(Vec3::new(base_circle + nsubdiv - 1, point, base_circle));
}

/// Reverses the clockwising of a set of faces.
pub fn reverce_clockwising(indices: &mut [Vec3<u32>]) {
    for i in indices.mut_iter() {
        mem::swap(&mut i.x, &mut i.y);
    }
}
