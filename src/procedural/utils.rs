//! Utilities useful for various generations tasks.

use na;
use na::{Point2, Point3, Vector2, Vector3};
use num::Zero;
use parry3d::utils::HashablePartialEq;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

// FIXME: remove that in favor of `push_xy_circle` ?
/// Pushes a discretized counterclockwise circle to a buffer.
#[inline]
pub fn push_circle(radius: f32, nsubdiv: u32, dtheta: f32, y: f32, out: &mut Vec<Point3<f32>>) {
    let mut curr_theta = 0.0f32;

    for _ in 0..nsubdiv {
        out.push(Point3::new(
            curr_theta.cos() * radius,
            y,
            curr_theta.sin() * radius,
        ));
        curr_theta += dtheta;
    }
}

/// Pushes a discretized counterclockwise circle to a buffer.
/// The circle is contained on the plane spanned by the `x` and `y` axis.
#[inline]
pub fn push_xy_arc(radius: f32, nsubdiv: u32, dtheta: f32, out: &mut Vec<Point2<f32>>) {
    let mut curr_theta = 0.0f32;

    for _ in 0..nsubdiv {
        let mut pt_coords = Vector2::zeros();

        pt_coords[0] = curr_theta.cos() * radius;
        pt_coords[1] = curr_theta.sin() * radius;
        out.push(Point2::from(pt_coords));

        curr_theta += dtheta;
    }
}

/// Creates the faces from two circles with the same discretization.
#[inline]
pub fn push_ring_indices(
    base_lower_circle: u32,
    base_upper_circle: u32,
    nsubdiv: u32,
    out: &mut Vec<Point3<u32>>,
) {
    push_open_ring_indices(base_lower_circle, base_upper_circle, nsubdiv, out);

    // adjust the last two triangles
    push_rectangle_indices(
        base_upper_circle,
        base_upper_circle + nsubdiv - 1,
        base_lower_circle,
        base_lower_circle + nsubdiv - 1,
        out,
    );
}

/// Creates the faces from two circles with the same discretization.
#[inline]
pub fn push_open_ring_indices(
    base_lower_circle: u32,
    base_upper_circle: u32,
    nsubdiv: u32,
    out: &mut Vec<Point3<u32>>,
) {
    assert!(nsubdiv > 0);

    for i in 0..nsubdiv - 1 {
        let bl_i = base_lower_circle + i;
        let bu_i = base_upper_circle + i;
        push_rectangle_indices(bu_i + 1, bu_i, bl_i + 1, bl_i, out);
    }
}

/// Creates the faces from a circle and a point that is shared by all triangle.
#[inline]
pub fn push_degenerate_top_ring_indices(
    base_circle: u32,
    point: u32,
    nsubdiv: u32,
    out: &mut Vec<Point3<u32>>,
) {
    push_degenerate_open_top_ring_indices(base_circle, point, nsubdiv, out);

    out.push(Point3::new(base_circle + nsubdiv - 1, point, base_circle));
}

/// Creates the faces from a circle and a point that is shared by all triangle.
#[inline]
pub fn push_degenerate_open_top_ring_indices(
    base_circle: u32,
    point: u32,
    nsubdiv: u32,
    out: &mut Vec<Point3<u32>>,
) {
    assert!(nsubdiv > 0);

    for i in 0..nsubdiv - 1 {
        out.push(Point3::new(base_circle + i, point, base_circle + i + 1));
    }
}

/// Pushes indices so that a circle is filled with triangles. Each triangle will have the
/// `base_circle` point in common.
/// Pushes `nsubdiv - 2` elements to `out`.
#[inline]
pub fn push_filled_circle_indices(base_circle: u32, nsubdiv: u32, out: &mut Vec<Point3<u32>>) {
    for i in base_circle + 1..base_circle + nsubdiv - 1 {
        out.push(Point3::new(base_circle, i, i + 1));
    }
}

/// Given four corner points, pushes to two counterclockwise triangles to `out`.
///
/// # Arguments:
/// * `ul` - the up-left point.
/// * `dl` - the down-left point.
/// * `dr` - the down-left point.
/// * `ur` - the up-left point.
#[inline]
pub fn push_rectangle_indices(ul: u32, ur: u32, dl: u32, dr: u32, out: &mut Vec<Point3<u32>>) {
    out.push(Point3::new(ul, dl, dr));
    out.push(Point3::new(dr, ur, ul));
}

/// Reverses the clockwising of a set of faces.
#[inline]
pub fn reverse_clockwising(indices: &mut [Point3<u32>]) {
    for i in indices.iter_mut() {
        i.coords.swap((0, 0), (1, 0));
    }
}

/// Duplicates the indices of each triangle on the given index buffer.
///
/// For example: [ (0.0, 1.0, 2.0) ] becomes: [ (0.0, 0.0, 0.0), (1.0, 1.0, 1.0), (2.0, 2.0, 2.0)].
#[inline]
pub fn split_index_buffer(indices: &[Point3<u32>]) -> Vec<Point3<Point3<u32>>> {
    let mut resi = Vec::new();

    for vertex in indices.iter() {
        resi.push(Point3::new(
            Point3::new(vertex.x, vertex.x, vertex.x),
            Point3::new(vertex.y, vertex.y, vertex.y),
            Point3::new(vertex.z, vertex.z, vertex.z),
        ));
    }

    resi
}

/// Duplicates the indices of each triangle on the given index buffer, giving the same id to each
/// identical vertex.
#[inline]
pub fn split_index_buffer_and_recover_topology(
    indices: &[Point3<u32>],
    coords: &[Point3<f32>],
) -> (Vec<Point3<Point3<u32>>>, Vec<Point3<f32>>) {
    let mut vtx_to_id = HashMap::default(); // with_hasher(DeterministicState::new());
    let mut new_coords = Vec::with_capacity(coords.len());
    let mut out = Vec::with_capacity(indices.len());

    fn resolve_coord_id(
        coord: &Point3<f32>,
        vtx_to_id: &mut HashMap<HashablePartialEq<Point3<f32>>, u32>,
        new_coords: &mut Vec<Point3<f32>>,
    ) -> u32 {
        let key = unsafe { HashablePartialEq::new(*coord) };
        let id = match vtx_to_id.entry(key) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(new_coords.len() as u32),
        };

        if *id == new_coords.len() as u32 {
            new_coords.push(*coord);
        }

        *id
    }

    for t in indices.iter() {
        let va = resolve_coord_id(&coords[t.x as usize], &mut vtx_to_id, &mut new_coords);
        let oa = t.x;

        let vb = resolve_coord_id(&coords[t.y as usize], &mut vtx_to_id, &mut new_coords);
        let ob = t.y;

        let vc = resolve_coord_id(&coords[t.z as usize], &mut vtx_to_id, &mut new_coords);
        let oc = t.z;

        out.push(Point3::new(
            Point3::new(va, oa, oa),
            Point3::new(vb, ob, ob),
            Point3::new(vc, oc, oc),
        ));
    }

    new_coords.shrink_to_fit();

    (out, new_coords)
}

// FIXME: check at compile-time that we are in 3D?
/// Computes the normals of a set of vertices.
#[inline]
pub fn compute_normals(
    coordinates: &[Point3<f32>],
    faces: &[Point3<u32>],
    normals: &mut Vec<Vector3<f32>>,
) {
    let mut divisor: Vec<f32> = vec![0.0; coordinates.len()];

    // Shrink the output buffer if it is too big.
    if normals.len() > coordinates.len() {
        normals.truncate(coordinates.len())
    }

    // Reinit all normals to zero.
    normals.clear();
    normals.extend(std::iter::repeat_n(
        na::zero::<Vector3<f32>>(),
        coordinates.len(),
    ));

    // Accumulate normals ...
    for f in faces.iter() {
        let edge1 = coordinates[f.y as usize] - coordinates[f.x as usize];
        let edge2 = coordinates[f.z as usize] - coordinates[f.x as usize];
        let cross = edge1.cross(&edge2);

        let normal = if !cross.is_zero() {
            cross.normalize()
        } else {
            cross
        };

        normals[f.x as usize] += normal;
        normals[f.y as usize] += normal;
        normals[f.z as usize] += normal;

        divisor[f.x as usize] += 1.0;
        divisor[f.y as usize] += 1.0;
        divisor[f.z as usize] += 1.0;
    }

    // ... and compute the mean
    for (n, divisor) in normals.iter_mut().zip(divisor.iter()) {
        *n /= *divisor
    }
}
