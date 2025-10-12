use super::utils;
use super::{IndexBuffer, RenderMesh};
use na;
use na::{Point3, Vector3};

/// Generates a cone with a given height and diameter.
pub fn cone(diameter: f32, height: f32, nsubdiv: u32) -> RenderMesh {
    let mut cone = unit_cone(nsubdiv);

    cone.scale_by(&Vector3::new(diameter, height, diameter));

    cone
}

/// Generates a cone with unit height and diameter.
pub fn unit_cone(nsubdiv: u32) -> RenderMesh {
    let two_pi = std::f32::consts::TAU;
    let dtheta = two_pi / (nsubdiv as f32);
    let mut coords = Vec::new();
    let mut indices = Vec::new();
    let mut normals: Vec<Vector3<f32>>;

    utils::push_circle(0.5, nsubdiv, dtheta, -0.5, &mut coords);

    normals = coords.iter().map(|p| p.coords).collect();

    coords.push(Point3::new(na::zero(), 0.5, na::zero()));

    utils::push_degenerate_top_ring_indices(0, coords.len() as u32 - 1, nsubdiv, &mut indices);
    utils::push_filled_circle_indices(0, nsubdiv, &mut indices);

    /*
     * Normals.
     */
    let mut indices = utils::split_index_buffer(&indices[..]);

    // Adjust the normals:
    let shift = 0.05f32 / 0.475;
    let div = (shift * shift + 0.25).sqrt();
    for n in normals.iter_mut() {
        n.y += shift;
        // FIXME: f32 / div does not work?
        n.x /= div;
        n.y /= div;
        n.z /= div;
    }

    // Normal for the basis.
    normals.push(Vector3::new(na::zero(), -1.0, na::zero()));

    let ilen = indices.len();
    let nlen = normals.len() as u32;
    for (id, i) in indices[..ilen - (nsubdiv as usize - 2)]
        .iter_mut()
        .enumerate()
    {
        i.y.y = id as u32;
    }

    for i in indices[ilen - (nsubdiv as usize - 2)..].iter_mut() {
        i.x.y = nlen - 1;
        i.y.y = nlen - 1;
        i.z.y = nlen - 1;
    }

    // Normal for the body.

    RenderMesh::new(
        coords,
        Some(normals),
        None,
        Some(IndexBuffer::Split(indices)),
    )

    // XXX: uvs
}
