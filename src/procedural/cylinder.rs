use super::utils;
use super::{IndexBuffer, RenderMesh};
use na;
use na::{Point2, Vector3};


/// Generates a cylinder with a given height and diameter.
pub fn cylinder(diameter: f32, height: f32, nsubdiv: u32) -> RenderMesh {
    let mut cylinder = unit_cylinder(nsubdiv);

    cylinder.scale_by(&Vector3::new(diameter, height, diameter));

    cylinder
}

/// Generates a cylinder with unit height and diameter.
pub fn unit_cylinder(nsubdiv: u32) -> RenderMesh {
    let two_pi = std::f32::consts::TAU;
    let invsubdiv = 1.0 / (nsubdiv as f32);
    let dtheta = two_pi * invsubdiv;
    let mut coords = Vec::new();
    let mut indices = Vec::new();
    let mut normals: Vec<Vector3<f32>>;

    utils::push_circle(
        0.5,
        nsubdiv,
        dtheta,
        -0.5,
        &mut coords,
    );

    normals = coords.iter().map(|p| p.coords).collect();

    utils::push_circle(
        0.5,
        nsubdiv,
        dtheta,
        0.5,
        &mut coords,
    );

    utils::push_ring_indices(0, nsubdiv, nsubdiv, &mut indices);
    utils::push_filled_circle_indices(0, nsubdiv, &mut indices);
    utils::push_filled_circle_indices(nsubdiv, nsubdiv, &mut indices);

    let len = indices.len();
    let bottom_start_id = len - (nsubdiv as usize - 2);
    utils::reverse_clockwising(&mut indices[bottom_start_id..]);

    let mut indices = utils::split_index_buffer(&indices[..]);

    /*
     * Compute uvs.
     */
    // bottom ring uvs
    let mut uvs = Vec::with_capacity(coords.len());
    let mut curr_u = 0.0;
    for _ in 0..nsubdiv {
        uvs.push(Point2::new(curr_u, na::zero()));
        curr_u = curr_u + invsubdiv;
    }

    // top ring uvs
    curr_u = na::zero();
    for _ in 0..nsubdiv {
        uvs.push(Point2::new(curr_u, 1.0));
        curr_u = curr_u + invsubdiv;
    }

    /*
     * Adjust normals.
     */
    for n in normals.iter_mut() {
        n.x = n.x * 2.0;
        n.y = 0.0;
        n.z = n.z * 2.0;
    }

    normals.push(Vector3::y()); // top cap
    normals.push(-Vector3::y()); // bottom cap
    let nlen = normals.len() as u32;

    let top_start_id = len - 2 * (nsubdiv as usize - 2);

    for i in indices[..top_start_id].iter_mut() {
        if i.x.y >= nsubdiv {
            i.x.y = i.x.y - nsubdiv;
        }
        if i.y.y >= nsubdiv {
            i.y.y = i.y.y - nsubdiv;
        }
        if i.z.y >= nsubdiv {
            i.z.y = i.z.y - nsubdiv;
        }
    }

    for i in indices[top_start_id..bottom_start_id].iter_mut() {
        i.x.y = nlen - 2;
        i.y.y = nlen - 2;
        i.z.y = nlen - 2;
    }

    for i in indices[bottom_start_id..].iter_mut() {
        i.x.y = nlen - 1;
        i.y.y = nlen - 1;
        i.z.y = nlen - 1;
    }

    RenderMesh::new(
        coords,
        Some(normals),
        Some(uvs),
        Some(IndexBuffer::Split(indices)),
    )
}
