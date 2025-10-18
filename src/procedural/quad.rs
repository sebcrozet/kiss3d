use super::{IndexBuffer, RenderMesh};
use na::{self, Point2, Point3, Vector3};

/// Generates a double-sided subdivided quad mesh.
///
/// Creates a rectangular quad lying on the XY plane, centered at the origin, with its
/// normal pointing along the Z axis. The quad is subdivided into a grid of triangles,
/// making it suitable for height maps or terrain.
///
/// # Arguments
/// * `width` - The quad width (extent along X axis)
/// * `height` - The quad height (extent along Y axis)
/// * `usubdivs` - Number of horizontal subdivisions (squares along width). Must not be 0.
/// * `vsubdivs` - Number of vertical subdivisions (squares along height). Must not be 0.
///
/// # Returns
/// A `RenderMesh` containing the subdivided quad geometry with normals and UVs
///
/// # Example
/// ```no_run
/// # use kiss3d::procedural::quad;
/// // Create a 10x10 quad with 100 subdivisions for a terrain
/// let terrain_mesh = quad(10.0, 10.0, 100, 100);
/// ```
///
/// # Panics
/// Panics if `usubdivs` or `vsubdivs` is 0.
pub fn quad(width: f32, height: f32, usubdivs: usize, vsubdivs: usize) -> RenderMesh {
    let mut quad = unit_quad(usubdivs, vsubdivs);

    let mut s = Vector3::zeros();
    s[0] = width;
    s[1] = height;
    s[2] = 1.0;

    quad.scale_by(&s);

    quad
}

/// Generates a double-sided quad mesh from a custom grid of vertices.
///
/// Creates a quad with custom vertex positions, useful for creating terrain or
/// deformed surfaces. Normals are automatically computed based on the surface geometry.
///
/// # Arguments
/// * `vertices` - Array of vertex positions defining the surface (must have `nhpoints × nvpoints` elements)
/// * `nhpoints` - Number of points along the horizontal direction (columns)
/// * `nvpoints` - Number of points along the vertical direction (rows)
///
/// # Returns
/// A `RenderMesh` containing the quad geometry with computed normals
///
/// # Example
/// ```no_run
/// # use kiss3d::procedural::quad_with_vertices;
/// # use nalgebra::Point3;
/// // Create a 3x3 grid of vertices for a simple heightmap
/// let vertices = vec![
///     Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0), Point3::new(2.0, 0.0, 0.0),
///     Point3::new(0.0, 0.5, 1.0), Point3::new(1.0, 0.5, 1.0), Point3::new(2.0, 0.5, 1.0),
///     Point3::new(0.0, 0.0, 2.0), Point3::new(1.0, 0.0, 2.0), Point3::new(2.0, 0.0, 2.0),
/// ];
/// let quad_mesh = quad_with_vertices(&vertices, 3, 3);
/// ```
///
/// # Panics
/// Panics if `nhpoints` or `nvpoints` is less than 2.
pub fn quad_with_vertices(
    vertices: &[Point3<f32>],
    nhpoints: usize,
    nvpoints: usize,
) -> RenderMesh {
    assert!(
        nhpoints > 1 && nvpoints > 1,
        "The number of points must be at least 2 in each dimension."
    );

    let mut res = unit_quad(nhpoints - 1, nvpoints - 1);

    for (dest, src) in res.coords.iter_mut().zip(vertices.iter()) {
        *dest = *src;
    }

    res
}

/// Generates a double-sided unit quad mesh.
///
/// Creates a 1×1 quad centered at the origin on the XY plane with its normal pointing
/// along the Z axis. The quad is subdivided into a grid of triangles.
///
/// # Arguments
/// * `usubdivs` - Number of horizontal subdivisions (squares along width). Must not be 0.
/// * `vsubdivs` - Number of vertical subdivisions (squares along height). Must not be 0.
///
/// # Returns
/// A `RenderMesh` containing the unit quad geometry with normals and UVs
///
/// # Example
/// ```no_run
/// # use kiss3d::procedural::unit_quad;
/// // Create a unit quad with 10x10 subdivisions
/// let quad_mesh = unit_quad(10, 10);
/// ```
///
/// # Panics
/// Panics if `usubdivs` or `vsubdivs` is 0.
pub fn unit_quad(usubdivs: usize, vsubdivs: usize) -> RenderMesh {
    assert!(
        usubdivs > 0 && vsubdivs > 0,
        "The number of subdivisions cannot be zero"
    );

    let wstep = 1.0 / (usubdivs as f32);
    let hstep = 1.0 / (vsubdivs as f32);
    let cw = 0.5;
    let ch = 0.5;

    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut triangles = Vec::new();
    let mut tex_coords = Vec::new();

    // create the vertices
    for i in 0usize..vsubdivs + 1 {
        for j in 0usize..usubdivs + 1 {
            let ni: f32 = i as f32;
            let nj: f32 = j as f32;

            let mut v = Point3::origin();
            v[0] = nj * wstep - cw;
            v[1] = ni * hstep - ch;
            vertices.push(v);
            tex_coords.push(Point2::new(1.0 - nj * wstep, 1.0 - ni * hstep))
        }
    }

    // create the normals
    for _ in 0..(vsubdivs + 1) * (usubdivs + 1) {
        let mut n = Vector3::zeros();
        n[0] = 1.0;
        normals.push(n)
    }

    // create triangles
    fn dl_triangle(i: u32, j: u32, ws: u32) -> Point3<u32> {
        Point3::new((i + 1) * ws + j, i * ws + j, (i + 1) * ws + j + 1)
    }

    fn ur_triangle(i: u32, j: u32, ws: u32) -> Point3<u32> {
        Point3::new(i * ws + j, i * ws + (j + 1), (i + 1) * ws + j + 1)
    }

    for i in 0usize..vsubdivs {
        for j in 0usize..usubdivs {
            // build two triangles...
            triangles.push(dl_triangle(i as u32, j as u32, (usubdivs + 1) as u32));
            triangles.push(ur_triangle(i as u32, j as u32, (usubdivs + 1) as u32));
        }
    }

    RenderMesh::new(
        vertices,
        Some(normals),
        Some(tex_coords),
        Some(IndexBuffer::Unified(triangles)),
    )
}
