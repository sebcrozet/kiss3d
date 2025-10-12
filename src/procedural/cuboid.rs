use super::RenderPolyline;
use super::{IndexBuffer, RenderMesh};
use na;
use na::{Point2, Point3, Vector2, Vector3};


/**
 * Generates a cuboid shape with a split index buffer.
 *
 * # Arguments:
 * * `extents` - the extents of the cuboid.
 */
pub fn cuboid(extents: &Vector3<f32>) -> RenderMesh {
    let mut cuboid = unit_cuboid();
    cuboid.scale_by(extents);

    cuboid
}

/**
 * Generates a cuboid shape with a split index buffer.
 *
 * The cuboid is centered at the origin, and has its half extents set to 0.5.
 */
pub fn unit_cuboid() -> RenderMesh {
    let mut coords = Vec::with_capacity(8);
    let mut uvs = Vec::with_capacity(4);
    let mut normals = Vec::with_capacity(6);
    let mut faces = Vec::with_capacity(12);

    let _1 = 1.0;
    let m1 = -_1;
    let _0 = 0.0;

    coords.push(Point3::new(-0.5, -0.5, 0.5));
    coords.push(Point3::new(-0.5, -0.5, -0.5));
    coords.push(Point3::new(0.5, -0.5, -0.5));
    coords.push(Point3::new(0.5, -0.5, 0.5));
    coords.push(Point3::new(-0.5, 0.5, 0.5));
    coords.push(Point3::new(-0.5, 0.5, -0.5));
    coords.push(Point3::new(0.5, 0.5, -0.5));
    coords.push(Point3::new(0.5, 0.5, 0.5));

    uvs.push(Point2::new(_0, _1));
    uvs.push(Point2::new(_1, _1));
    uvs.push(Point2::new(_0, _0));
    uvs.push(Point2::new(_1, _0));

    normals.push(Vector3::new(m1, _0, _0));
    normals.push(Vector3::new(_0, _0, m1));
    normals.push(Vector3::new(_1, _0, _0));
    normals.push(Vector3::new(_0, _0, _1));
    normals.push(Vector3::new(_0, m1, _0));
    normals.push(Vector3::new(_0, _1, _0));

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

/// The contour of a cuboid lying on the x-y plane.
pub fn rectangle(extents: &Vector2<f32>) -> RenderPolyline {
    let mut rectangle = unit_rectangle();

    rectangle.scale_by(extents);

    rectangle
}

/// The contour of a unit cuboid lying on the x-y plane.
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
