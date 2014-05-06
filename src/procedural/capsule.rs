use nalgebra::na;
use nalgebra::na::Cast;
use procedural::{MeshDescr, UnifiedIndexBuffer};
use procedural::{sphere, utils};

/// Generates a capsule.
pub fn capsule<N: Float + Cast<f64>>(caps_diameter:   &N,
                                     cylinder_height: &N,
                                     ntheta_subdiv:   u32,
                                     nphi_subdiv:     u32)
                                     -> MeshDescr<N> {
    let top = sphere::unit_hemisphere::<N>(ntheta_subdiv, nphi_subdiv);
    let MeshDescr { coords, normals, indices, .. } = top.clone();
    let mut bottom_coords  = coords;
    let mut bottom_normals = normals.unwrap();
    let mut bottom_indices = indices.unwrap_unified();
    utils::reverse_clockwising(bottom_indices.as_mut_slice());

    let MeshDescr { coords, normals, indices, .. } = top;
    let mut top_coords  = coords;
    let top_normals     = normals.unwrap();
    let mut top_indices = indices.unwrap_unified();

    let half_height = *cylinder_height * na::cast(0.5);

    // shift the top
    for coord in top_coords.mut_iter() {
        coord.x = coord.x * *caps_diameter;
        coord.y = coord.y * *caps_diameter + half_height;
        coord.z = coord.z * *caps_diameter;
    }

    // flip + shift the bottom
    for coord in bottom_coords.mut_iter() {
        coord.x = coord.x * *caps_diameter;
        coord.y = -(coord.y * *caps_diameter) - half_height;
        coord.z = coord.z * *caps_diameter;
    }

    // flip the bottom normals
    for normal in bottom_normals.mut_iter() {
        normal.y = -normal.y;
    }

    // shift the top index buffer
    let base_top_coords = bottom_coords.len() as u32;

    for idx in top_indices.mut_iter() {
        idx.x = idx.x + base_top_coords;
        idx.y = idx.y + base_top_coords;
        idx.z = idx.z + base_top_coords;
    }

    // merge all buffers
    bottom_coords.push_all_move(top_coords);
    bottom_normals.push_all_move(top_normals);
    bottom_indices.push_all_move(top_indices);

    // attach the two caps
    utils::push_ring_indices(0, base_top_coords, ntheta_subdiv, &mut bottom_indices);

    // FIXME: uvs
    MeshDescr::new(bottom_coords,
                   Some(bottom_normals),
                   None,
                   Some(UnifiedIndexBuffer(bottom_indices)))
}
