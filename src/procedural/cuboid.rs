use super::RenderPolyline;
use super::{IndexBuffer, RenderMesh};
use na;
use na::{Point2, Point3, Vector2, Vector3};

/// Generates a cuboid (box) mesh with the specified extents.
///
/// Creates a rectangular box mesh centered at the origin with the given dimensions.
/// The mesh includes normals and UV coordinates.
///
/// # Arguments
/// * `extents` - The full dimensions of the cuboid along each axis (width, height, depth)
///
/// # Returns
/// A `RenderMesh` containing the cuboid geometry with split index buffer
///
/// # Example
/// ```no_run
/// # use kiss3d::procedural::cuboid;
/// # use nalgebra::Vector3;
/// // Create a 2x3x4 box
/// let box_mesh = cuboid(&Vector3::new(2.0, 3.0, 4.0));
/// ```
pub fn cuboid(extents: &Vector3<f32>) -> RenderMesh {
    let mut cuboid = unit_cuboid();
    cuboid.scale_by(extents);

    cuboid
}

/// Generates a unit cuboid mesh.
///
/// Creates a cube mesh centered at the origin with dimensions 1×1×1 (half-extents of 0.5).
/// The mesh includes normals and UV coordinates.
///
/// # Returns
/// A `RenderMesh` containing the unit cuboid geometry
///
/// # Example
/// ```no_run
/// # use kiss3d::procedural::unit_cuboid;
/// // Create a unit cube (1x1x1)
/// let cube_mesh = unit_cuboid();
/// ```
pub fn unit_cuboid() -> RenderMesh {
    let mut coords = Vec::with_capacity(8);
    let mut uvs = Vec::with_capacity(4);
    let mut normals = Vec::with_capacity(6);
    let mut faces = Vec::with_capacity(12);

    coords.push(Point3::new(-0.5, -0.5, 0.5));
    coords.push(Point3::new(-0.5, -0.5, -0.5));
    coords.push(Point3::new(0.5, -0.5, -0.5));
    coords.push(Point3::new(0.5, -0.5, 0.5));
    coords.push(Point3::new(-0.5, 0.5, 0.5));
    coords.push(Point3::new(-0.5, 0.5, -0.5));
    coords.push(Point3::new(0.5, 0.5, -0.5));
    coords.push(Point3::new(0.5, 0.5, 0.5));

    uvs.push(Point2::new(0.0, 1.0));
    uvs.push(Point2::new(1.0, 1.0));
    uvs.push(Point2::new(0.0, 0.0));
    uvs.push(Point2::new(1.0, 0.0));

    normals.push(Vector3::new(-1.0, 0.0, 0.0));
    normals.push(Vector3::new(0.0, 0.0, -1.0));
    normals.push(Vector3::new(1.0, 0.0, 0.0));
    normals.push(Vector3::new(0.0, 0.0, 1.0));
    normals.push(Vector3::new(0.0, -1.0, 0.0));
    normals.push(Vector3::new(0.0, 1.0, 0.0));

    faces.push(Point3::new(
        Point3::new(4, 0, 0),
        Point3::new(5, 0, 1),
        Point3::new(0, 0, 2),
    ));
    faces.push(Point3::new(
        Point3::new(5, 0, 1),
        Point3::new(1, 0, 3),
        Point3::new(0, 0, 2),
    ));

    faces.push(Point3::new(
        Point3::new(5, 1, 0),
        Point3::new(6, 1, 1),
        Point3::new(1, 1, 2),
    ));
    faces.push(Point3::new(
        Point3::new(6, 1, 1),
        Point3::new(2, 1, 3),
        Point3::new(1, 1, 2),
    ));

    faces.push(Point3::new(
        Point3::new(6, 2, 1),
        Point3::new(7, 2, 0),
        Point3::new(3, 2, 2),
    ));
    faces.push(Point3::new(
        Point3::new(2, 2, 3),
        Point3::new(6, 2, 1),
        Point3::new(3, 2, 2),
    ));

    faces.push(Point3::new(
        Point3::new(7, 3, 1),
        Point3::new(4, 3, 0),
        Point3::new(0, 3, 2),
    ));
    faces.push(Point3::new(
        Point3::new(3, 3, 3),
        Point3::new(7, 3, 1),
        Point3::new(0, 3, 2),
    ));

    faces.push(Point3::new(
        Point3::new(0, 4, 2),
        Point3::new(1, 4, 0),
        Point3::new(2, 4, 1),
    ));
    faces.push(Point3::new(
        Point3::new(3, 4, 3),
        Point3::new(0, 4, 2),
        Point3::new(2, 4, 1),
    ));

    faces.push(Point3::new(
        Point3::new(7, 5, 3),
        Point3::new(6, 5, 1),
        Point3::new(5, 5, 0),
    ));
    faces.push(Point3::new(
        Point3::new(4, 5, 2),
        Point3::new(7, 5, 3),
        Point3::new(5, 5, 0),
    ));

    RenderMesh::new(
        coords,
        Some(normals),
        Some(uvs),
        Some(IndexBuffer::Split(faces)),
    )
}

/// Generates a 2D rectangle polyline with the specified extents.
///
/// Creates the outline of a rectangle lying on the XY plane, centered at the origin.
///
/// # Arguments
/// * `extents` - The dimensions of the rectangle (width, height)
///
/// # Returns
/// A `RenderPolyline` containing the rectangle's outline
///
/// # Example
/// ```no_run
/// # use kiss3d::procedural::rectangle;
/// # use nalgebra::Vector2;
/// // Create a 4x2 rectangle outline
/// let rect_polyline = rectangle(&Vector2::new(4.0, 2.0));
/// ```
pub fn rectangle(extents: &Vector2<f32>) -> RenderPolyline {
    let mut rectangle = unit_rectangle();

    rectangle.scale_by(extents);

    rectangle
}

/// Generates a unit rectangle polyline on the XY plane.
///
/// Creates the outline of a 1×1 rectangle centered at the origin on the XY plane.
///
/// # Returns
/// A `RenderPolyline` containing the unit rectangle's outline
///
/// # Example
/// ```no_run
/// # use kiss3d::procedural::unit_rectangle;
/// // Create a unit square outline (1x1)
/// let rect_polyline = unit_rectangle();
/// ```
pub fn unit_rectangle() -> RenderPolyline {
    let mut p_ul = Point2::origin();
    let mut p_ur = Point2::origin();
    let mut p_dl = Point2::origin();
    let mut p_dr = Point2::origin();

    p_dl[0] = -0.5;
    p_dl[1] = -0.5;
    p_dr[0] = 0.5;
    p_dr[1] = -0.5;
    p_ur[0] = 0.5;
    p_ur[1] = 0.5;
    p_ul[0] = -0.5;
    p_ul[1] = 0.5;

    RenderPolyline::new(vec![p_ur, p_ul, p_dl, p_dr], None)
}
