use nalgebra::na;
use nalgebra::na::{Cast, Vec3};
use procedural::{MeshDescr, SplitIndexBuffer};
use procedural::utils;

/// Generates a cylinder with a given height and diameter.
pub fn cylinder<N: Float + Cast<f64>>(diameter: N, height: N, nsubdiv: u32) -> MeshDescr<N> {
    let mut cylinder = unit_cylinder(nsubdiv);

    cylinder.scale_by(&Vec3::new(diameter, height, diameter));

    cylinder
}

/// Generates a cylinder with unit height and diameter.
pub fn unit_cylinder<N: Float + Cast<f64>>(nsubdiv: u32) -> MeshDescr<N> {
    let two_pi: N   = Float::two_pi();
    let dtheta      = two_pi / na::cast(nsubdiv as f64);
    let mut coords  = Vec::new();
    let mut indices = Vec::new();
    let mut normals;

    utils::push_circle(na::cast(0.5), nsubdiv, dtheta, na::cast(-0.5), &mut coords);

    normals = coords.clone();

    utils::push_circle(na::cast(0.5), nsubdiv, dtheta, na::cast(0.5),  &mut coords);

    utils::push_ring_indices(0, nsubdiv, nsubdiv, &mut indices);
    utils::push_closed_circle_indices(0, nsubdiv, &mut indices);
    utils::push_closed_circle_indices(nsubdiv, nsubdiv, &mut indices);

    let len             = indices.len();
    let bottom_start_id = len - (nsubdiv as uint - 2);
    utils::reverse_clockwising(indices.mut_slice_from(bottom_start_id));

    let mut indices = utils::split_index_buffer(indices.as_slice());

    /*
     * Adjust normals.
     */
    for n in normals.mut_iter() {
        n.x = n.x * na::cast(2.0);
        n.y = na::zero();
        n.z = n.z * na::cast(2.0);
    }

    normals.push(Vec3::y());  // top cap
    normals.push(-Vec3::y()); // bottom cap
    let nlen = normals.len() as u32;

    let top_start_id = len - 2 * (nsubdiv as uint - 2);

    for i in indices.mut_slice_to(top_start_id).mut_iter() {
        if i.x.z >= nsubdiv {
            i.x.z = i.x.z - nsubdiv;
        }
        if i.y.z >= nsubdiv {
            i.y.z = i.y.z - nsubdiv;
        }
        if i.z.z >= nsubdiv {
            i.z.z = i.z.z - nsubdiv;
        }
    }

    for i in indices.mut_slice(top_start_id, bottom_start_id).mut_iter() {
        i.x.z = nlen - 2;
        i.y.z = nlen - 2;
        i.z.z = nlen - 2;
    }

    for i in indices.mut_slice_from(bottom_start_id).mut_iter() {
        i.x.z = nlen - 1;
        i.y.z = nlen - 1;
        i.z.z = nlen - 1;
    }

    MeshDescr::new(coords, Some(normals), None, Some(SplitIndexBuffer(indices)))

    // XXX: uvs
}
