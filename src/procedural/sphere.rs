use super::utils;
use super::RenderPolyline;
use super::{IndexBuffer, RenderMesh};
use na;
use na::{Point2, Point3, Vector3};

/// Generates a UV sphere.
pub fn sphere(
    diameter: f32,
    ntheta_subdiv: u32,
    nphi_subdiv: u32,
    generate_uvs: bool,
) -> RenderMesh {
    let mut sphere = unit_sphere(ntheta_subdiv, nphi_subdiv, generate_uvs);

    sphere.scale_by_scalar(diameter);

    sphere
}

/// Generates a UV sphere centered at the origin and with a unit diameter.
pub fn unit_sphere(ntheta_subdiv: u32, nphi_subdiv: u32, generate_uvs: bool) -> RenderMesh {
    if generate_uvs {
        unit_sphere_with_uvs(ntheta_subdiv, nphi_subdiv)
    } else {
        unit_sphere_without_uvs(ntheta_subdiv, nphi_subdiv)
    }
}

// FIXME: f32{theta,phi}_subdiv are not the right names.
fn unit_sphere_without_uvs(ntheta_subdiv: u32, nphi_subdiv: u32) -> RenderMesh {
    let pi = std::f32::consts::PI;
    let two_pi = std::f32::consts::TAU;
    let pi_two = std::f32::consts::FRAC_PI_2;
    let dtheta = two_pi / (ntheta_subdiv as f32);
    let dphi = pi / (nphi_subdiv as f32);

    let mut coords = Vec::new();
    let mut curr_phi = -pi_two + dphi;

    // coords.
    coords.push(Point3::new(na::zero(), -1.0, na::zero()));

    for _ in 0..nphi_subdiv - 1 {
        utils::push_circle(
            curr_phi.cos(),
            ntheta_subdiv,
            dtheta,
            curr_phi.sin(),
            &mut coords,
        );
        curr_phi += dphi;
    }

    coords.push(Point3::new(na::zero(), 1.0, na::zero()));

    // the normals are the same as the coords.
    let normals: Vec<Vector3<f32>> = coords.iter().map(|p| p.coords).collect();

    // index buffer
    let mut idx = Vec::new();

    utils::push_degenerate_top_ring_indices(1, 0, ntheta_subdiv, &mut idx);

    utils::reverse_clockwising(&mut idx[..]);

    for i in 0..nphi_subdiv - 2 {
        let bottom = 1 + i * ntheta_subdiv;
        let up = bottom + ntheta_subdiv;
        utils::push_ring_indices(bottom, up, ntheta_subdiv, &mut idx);
    }

    utils::push_degenerate_top_ring_indices(
        1 + (nphi_subdiv - 2) * ntheta_subdiv,
        coords.len() as u32 - 1,
        ntheta_subdiv,
        &mut idx,
    );

    let mut res = RenderMesh::new(coords, Some(normals), None, Some(IndexBuffer::Unified(idx)));
    res.scale_by_scalar(0.5);

    res
}

fn unit_sphere_with_uvs(ntheta_subdiv: u32, nphi_subdiv: u32) -> RenderMesh {
    let pi = std::f32::consts::PI;
    let two_pi = std::f32::consts::TAU;
    let pi_two = std::f32::consts::FRAC_PI_2;
    let duvtheta = 1.0 / (ntheta_subdiv as f32); // step of uv.x coordinates.
    let duvphi = 1.0 / (nphi_subdiv as f32); // step of uv.y coordinates.
    let dtheta = two_pi * duvtheta;
    let dphi = pi * duvphi;

    let mut coords = Vec::new();
    let mut curr_phi = -pi_two;

    for _ in 0..nphi_subdiv + 1 {
        utils::push_circle(
            curr_phi.cos(),
            ntheta_subdiv + 1,
            dtheta,
            curr_phi.sin(),
            &mut coords,
        );
        curr_phi += dphi;
    }

    // the normals are the same as the coords
    let normals: Vec<Vector3<f32>> = coords.iter().map(|p| p.coords).collect();

    // index buffer
    let mut idx = Vec::new();

    for i in 0..nphi_subdiv {
        let bottom = i * (ntheta_subdiv + 1);
        let up = bottom + (ntheta_subdiv + 1);
        utils::push_open_ring_indices(bottom, up, ntheta_subdiv + 1, &mut idx);
    }

    let mut uvs = Vec::new();
    let mut curr_uvphi = 0.0;

    for _ in 0..nphi_subdiv + 1 {
        let mut curr_uvtheta = 0.0;

        for _ in 0..ntheta_subdiv + 1 {
            uvs.push(Point2::new(curr_uvtheta, curr_uvphi));
            curr_uvtheta += duvtheta;
        }

        curr_uvphi += duvphi;
    }

    let mut res = RenderMesh::new(
        coords,
        Some(normals),
        Some(uvs),
        Some(IndexBuffer::Unified(idx)),
    );

    res.scale_by_scalar(0.5);

    res
}

/// Creates an hemisphere with a diameter of 1.
pub fn unit_hemisphere(ntheta_subdiv: u32, nphi_subdiv: u32) -> RenderMesh {
    let two_pi = std::f32::consts::TAU;
    let pi_two = std::f32::consts::FRAC_PI_2;
    let dtheta = two_pi / (ntheta_subdiv as f32);
    let dphi = pi_two / (nphi_subdiv as f32);

    let mut coords = Vec::new();
    let mut curr_phi = 0.0f32;

    for _ in 0..nphi_subdiv - 1 {
        utils::push_circle(
            curr_phi.cos(),
            ntheta_subdiv,
            dtheta,
            curr_phi.sin(),
            &mut coords,
        );
        curr_phi += dphi;
    }

    coords.push(Point3::new(na::zero(), 1.0, na::zero()));

    let mut idx = Vec::new();

    for i in 0..nphi_subdiv - 2 {
        utils::push_ring_indices(
            i * ntheta_subdiv,
            (i + 1) * ntheta_subdiv,
            ntheta_subdiv,
            &mut idx,
        );
    }

    utils::push_degenerate_top_ring_indices(
        (nphi_subdiv - 2) * ntheta_subdiv,
        coords.len() as u32 - 1,
        ntheta_subdiv,
        &mut idx,
    );

    // Result
    let normals: Vec<Vector3<f32>> = coords.iter().map(|p| p.coords).collect();
    // FIXME: uvs
    let mut out = RenderMesh::new(coords, Some(normals), None, Some(IndexBuffer::Unified(idx)));

    // set the radius to 0.5
    out.scale_by_scalar(0.5);

    out
}

/// Creates a circle lying on the `(x,y)` plane.
pub fn circle(diameter: f32, nsubdivs: u32) -> RenderPolyline {
    let two_pi = std::f32::consts::TAU;
    let dtheta = two_pi / nsubdivs as f32;

    let mut pts = Vec::with_capacity(nsubdivs as usize);

    utils::push_xy_arc(diameter / 2.0, nsubdivs, dtheta, &mut pts);

    // FIXME: f32ormals

    RenderPolyline::new(pts, None)
}

/// Creates a circle lying on the `(x,y)` plane.
pub fn unit_circle(nsubdivs: u32) -> RenderPolyline {
    // FIXME: do this the other way round?
    circle(1.0, nsubdivs)
}
