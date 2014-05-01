use nalgebra::na;
use nalgebra::na::{Cast, Vec3, Vec2};
use procedural::{MeshDescr, UnifiedIndexBuffer};

/// Adds a double-sided quad to the scene. The quad is initially centered at (0, 0, 0). Its normal
/// is the `z` axis. The quad itself is composed of a user-defined number of triangles regularly
/// spaced on a grid. This is the main way to draw height maps.
///
/// # Arguments
/// * `w` - the quad width.
/// * `h` - the quad height.
/// * `wsubdivs` - number of horizontal subdivisions. This correspond to the number of squares
/// which will be placed horizontally on each line. Must not be `0`.
/// * `hsubdivs` - number of vertical subdivisions. This correspond to the number of squares
/// which will be placed vertically on each line. Must not be `0`.
pub fn quad<N: Num + Cast<f64>>(width:    N,
                                height:   N,
                                wsubdivs: uint,
                                hsubdivs: uint)
                                -> MeshDescr<N> {
    let mut quad = unit_quad(wsubdivs, hsubdivs);

    quad.scale_by(&Vec3::new(width, height, na::zero()));

    quad
}

/// Adds a double-sided quad with unit size to the scene. The quad is initially centered at (0, 0,
/// 0). Its normal is the `z` axis. The quad itself is composed of a user-defined number of
///  triangles regularly spaced on a grid. This is the main way to draw height maps.
///
/// # Arguments
/// * `wsubdivs` - number of horizontal subdivisions. This correspond to the number of squares
/// which will be placed horizontally on each line. Must not be `0`.
/// * `hsubdivs` - number of vertical subdivisions. This correspond to the number of squares
/// which will be placed vertically on each line. Must not be `0`.
pub fn unit_quad<N: Num + Cast<f64>>(wsubdivs: uint, hsubdivs: uint) -> MeshDescr<N> {
    assert!(wsubdivs > 0 && hsubdivs > 0, "The number of subdivisions cannot be zero");

    let wstep    = na::one::<N>() / na::cast(wsubdivs as f64);
    let hstep    = na::one::<N>() / na::cast(hsubdivs as f64);
    let cw       = na::cast(0.5);
    let ch       = na::cast(0.5);

    let mut vertices   = Vec::new();
    let mut normals    = Vec::new();
    let mut triangles  = Vec::new();
    let mut tex_coords = Vec::new();

    // create the vertices
    for i in range(0u, hsubdivs + 1) {
        for j in range(0u, wsubdivs + 1) {
            let ni: N = na::cast(i as f64);
            let nj: N = na::cast(j as f64);

            vertices.push(Vec3::new(nj * wstep - cw, ni * hstep - ch, na::zero()));
            tex_coords.push(Vec2::new(na::one::<N>() - nj * wstep, na::one::<N>() - ni * hstep))
        }
    }

    // create the normals
    for _ in range(0, (hsubdivs + 1) * (wsubdivs + 1)) {
        normals.push(Vec3::x())
    }

    // create triangles
    fn dl_triangle(i: u32, j: u32, ws: u32) -> Vec3<u32> {
        Vec3::new((i + 1) * ws + j, i * ws + j, (i + 1) * ws + j + 1)
    }

    fn ur_triangle(i: u32, j: u32, ws: u32) -> Vec3<u32> {
        Vec3::new(i * ws + j, i * ws + (j + 1), (i + 1) * ws + j + 1)
    }

    for i in range(0u, hsubdivs) {
        for j in range(0u, wsubdivs) {
            // build two triangles...
            triangles.push(dl_triangle(i as u32, j as u32, (wsubdivs + 1) as u32));
            triangles.push(ur_triangle(i as u32, j as u32, (wsubdivs + 1) as u32));
        }
    }

    MeshDescr::new(vertices, Some(normals), Some(tex_coords), Some(UnifiedIndexBuffer(triangles)))
}
