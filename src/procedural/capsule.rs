use super::{sphere, utils};
use super::{IndexBuffer, RenderMesh};
use na;

/// Generates a capsule.
pub fn capsule(
    caps_diameter: f32,
    cylinder_height: f32,
    ntheta_subdiv: u32,
    nphi_subdiv: u32,
) -> RenderMesh
{
    let top = sphere::unit_hemisphere(ntheta_subdiv, nphi_subdiv);
    let RenderMesh {
        coords,
        normals,
        indices,
        ..
    } = top.clone();
    let mut bottom_coords = coords;
    let mut bottom_normals = normals.unwrap();
    let mut bottom_indices = indices.unwrap_unified();
    utils::reverse_clockwising(&mut bottom_indices[..]);

    let RenderMesh {
        coords,
        normals,
        indices,
        ..
    } = top;
    let mut top_coords = coords;
    let top_normals = normals.unwrap();
    let mut top_indices = indices.unwrap_unified();

    let half_height = cylinder_height * 0.5;

    // shift the top
    for coord in top_coords.iter_mut() {
        coord.x = coord.x * caps_diameter;
        coord.y = coord.y * caps_diameter + half_height;
        coord.z = coord.z * caps_diameter;
    }

    // flip + shift the bottom
    for coord in bottom_coords.iter_mut() {
        coord.x = coord.x * caps_diameter;
        coord.y = -(coord.y * caps_diameter) - half_height;
        coord.z = coord.z * caps_diameter;
    }

    // flip the bottom normals
    for normal in bottom_normals.iter_mut() {
        normal.y = -normal.y;
    }

    // shift the top index buffer
    let base_top_coords = bottom_coords.len() as u32;

    for idx in top_indices.iter_mut() {
        idx.x = idx.x + base_top_coords;
        idx.y = idx.y + base_top_coords;
        idx.z = idx.z + base_top_coords;
    }

    // merge all buffers
    bottom_coords.extend(top_coords.into_iter());
    bottom_normals.extend(top_normals.into_iter());
    bottom_indices.extend(top_indices.into_iter());

    // attach the two caps
    utils::push_ring_indices(0, base_top_coords, ntheta_subdiv, &mut bottom_indices);

    // FIXME: uvs
    RenderMesh::new(
        bottom_coords,
        Some(bottom_normals),
        None,
        Some(IndexBuffer::Unified(bottom_indices)),
    )
}
