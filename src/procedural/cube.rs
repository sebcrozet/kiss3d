use nalgebra::na;
use nalgebra::na::{Cast, Vec3, Vec2};
use procedural::{MeshDescr, SplitIndexBuffer};

/**
 * Generates a cube geometry with a split index buffer.
 *
 * # Arguments:
 * * `extents` - the extents of the cube.
 */
pub fn cube<N: Num + Clone + Cast<f64>>(extents: &Vec3<N>) -> MeshDescr<N> {
    let mut cube = unit_cube();

    cube.scale_by(extents);

    cube
}

/**
 * Generates a cube geometry with a split index buffer.
 *
 * The cube is centered at the origin, and has its half extents set to 0.5.
 */
pub fn unit_cube<N: Num + Clone + Cast<f64>>() -> MeshDescr<N> {
    let mut coords  = Vec::new();
    let mut uvs     = Vec::new();
    let mut normals = Vec::new();
    let mut faces   = Vec::new();

    let _0_5: N = na::cast(0.5);
    let m0_5: N = -_0_5;
    let _1:   N = na::one();
    let m1:   N = -_1;
    let _0:   N = na::zero();

    coords.push(Vec3::new(m0_5.clone(), m0_5.clone(), _0_5.clone()));
    coords.push(Vec3::new(m0_5.clone(), m0_5.clone(), m0_5.clone()));
    coords.push(Vec3::new(_0_5.clone(), m0_5.clone(), m0_5.clone()));
    coords.push(Vec3::new(_0_5.clone(), m0_5.clone(), _0_5.clone()));
    coords.push(Vec3::new(m0_5.clone(), _0_5.clone(), _0_5.clone()));
    coords.push(Vec3::new(m0_5.clone(), _0_5.clone(), m0_5.clone()));
    coords.push(Vec3::new(_0_5.clone(), _0_5.clone(), m0_5.clone()));
    coords.push(Vec3::new(_0_5.clone(), _0_5.clone(), _0_5.clone()));

    uvs.push(Vec2::new(_0.clone(), _0.clone()));
    uvs.push(Vec2::new(_1.clone(), _0.clone()));
    uvs.push(Vec2::new(_0.clone(), _1.clone()));
    uvs.push(Vec2::new(_1.clone(), _1.clone()));

    normals.push(Vec3::new(m1.clone(), _0.clone(), _0.clone()));
    normals.push(Vec3::new(_0.clone(), _0.clone(), m1.clone()));
    normals.push(Vec3::new(_1.clone(), _0.clone(), _0.clone()));
    normals.push(Vec3::new(_0.clone(), _0.clone(), _1.clone()));
    normals.push(Vec3::new(_0.clone(), m1.clone(), _0.clone()));
    normals.push(Vec3::new(_0.clone(), _1.clone(), _0.clone()));

    faces.push(Vec3::new(Vec3::new(4, 0, 0), Vec3::new(5, 1, 0), Vec3::new(0, 2, 0)));
    faces.push(Vec3::new(Vec3::new(5, 0, 1), Vec3::new(6, 1, 1), Vec3::new(1, 2, 1)));
    faces.push(Vec3::new(Vec3::new(6, 1, 2), Vec3::new(7, 0, 2), Vec3::new(3, 2, 2)));
    faces.push(Vec3::new(Vec3::new(7, 1, 3), Vec3::new(4, 0, 3), Vec3::new(0, 2, 3)));
    faces.push(Vec3::new(Vec3::new(0, 2, 4), Vec3::new(1, 0, 4), Vec3::new(2, 1, 4)));
    faces.push(Vec3::new(Vec3::new(7, 3, 5), Vec3::new(6, 1, 5), Vec3::new(5, 0, 5)));
    faces.push(Vec3::new(Vec3::new(5, 1, 0), Vec3::new(1, 3, 0), Vec3::new(0, 2, 0)));
    faces.push(Vec3::new(Vec3::new(6, 1, 1), Vec3::new(2, 3, 1), Vec3::new(1, 2, 1)));
    faces.push(Vec3::new(Vec3::new(2, 3, 2), Vec3::new(6, 1, 2), Vec3::new(3, 2, 2)));
    faces.push(Vec3::new(Vec3::new(3, 3, 3), Vec3::new(7, 1, 3), Vec3::new(0, 2, 3)));
    faces.push(Vec3::new(Vec3::new(3, 3, 4), Vec3::new(0, 2, 4), Vec3::new(2, 1, 4)));
    faces.push(Vec3::new(Vec3::new(4, 2, 5), Vec3::new(7, 3, 5), Vec3::new(5, 0, 5)));

    MeshDescr::new(coords, Some(normals), Some(uvs), Some(SplitIndexBuffer(faces)))
}
