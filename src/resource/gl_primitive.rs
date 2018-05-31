//! Structures that a gpu buffer may contain.

use context::{self, Context, UniformLocation};
use na::{Matrix2, Matrix3, Matrix4, Point2, Point3, Rotation2, Rotation3, Vector2, Vector3};
use std::mem;

#[path = "../error.rs"]
mod error;

/// Trait implemented by structures that can be uploaded to a uniform or contained by a gpu array.
pub trait GLPrimitive: Copy {
    /// The Opengl primitive type of this structure content.
    fn gl_type(_type: Option<Self>) -> u32;
    /// The number of elements of type `self.gl_type()` this structure stores.
    fn size(_type: Option<Self>) -> u32;
    /// Uploads the element to a gpu location.
    fn upload(&self, location: &UniformLocation);
}

/*
 *
 * Impl for primitive types
 *
 */
impl GLPrimitive for f32 {
    #[inline]
    fn gl_type(_: Option<f32>) -> u32 {
        Context::FLOAT
    }

    #[inline]
    fn size(_: Option<f32>) -> u32 {
        1
    }

    #[inline]
    fn upload(&self, location: &UniformLocation) {
        verify!(Context::get().uniform1f(Some(location), self.clone()));
    }
}

impl GLPrimitive for i32 {
    #[inline]
    fn gl_type(_: Option<i32>) -> u32 {
        Context::INT
    }

    #[inline]
    fn size(_: Option<i32>) -> u32 {
        1
    }

    #[inline]
    fn upload(&self, location: &UniformLocation) {
        verify!(Context::get().uniform1i(Some(location), self.clone()));
    }
}

// // impl GLPrimitive for u32 {
// //     #[inline]
// //     fn gl_type(_: Option<u32>) -> u32 {
// //         gl::UNSIGNED_INT
// //     }

// //     #[inline]
// //     fn size(_: Option<u32>) -> u32 {
// //         1
// //     }

// //     #[inline]
// //     fn upload(&self, location: &UniformLocation) {
// //         verify!(Context::get().uniform1ui(Some(location), self.clone()));
// //     }
// // }

/*
 *
 * Impl for matrices
 *
 */
impl GLPrimitive for Matrix2<f32> {
    #[inline]
    fn gl_type(_: Option<Matrix2<f32>>) -> u32 {
        Context::FLOAT
    }

    #[inline]
    fn size(_: Option<Matrix2<f32>>) -> u32 {
        4
    }

    #[inline]
    fn upload(&self, location: &UniformLocation) {
        unsafe {
            verify!(Context::get().uniform_matrix2fv(Some(location), false, self));
        }
    }
}

impl GLPrimitive for Rotation2<f32> {
    #[inline]
    fn gl_type(_: Option<Rotation2<f32>>) -> u32 {
        Context::FLOAT
    }

    #[inline]
    fn size(_: Option<Rotation2<f32>>) -> u32 {
        4
    }

    #[inline]
    fn upload(&self, location: &UniformLocation) {
        unsafe {
            verify!(Context::get().uniform_matrix2fv(Some(location), false, self.matrix()));
        }
    }
}

impl GLPrimitive for Matrix3<f32> {
    #[inline]
    fn gl_type(_: Option<Matrix3<f32>>) -> u32 {
        Context::FLOAT
    }

    #[inline]
    fn size(_: Option<Matrix3<f32>>) -> u32 {
        9
    }

    #[inline]
    fn upload(&self, location: &UniformLocation) {
        unsafe {
            verify!(Context::get().uniform_matrix3fv(Some(location), false, self));
        }
    }
}

impl GLPrimitive for Rotation3<f32> {
    #[inline]
    fn gl_type(_: Option<Rotation3<f32>>) -> u32 {
        Context::FLOAT
    }

    #[inline]
    fn size(_: Option<Rotation3<f32>>) -> u32 {
        9
    }

    #[inline]
    fn upload(&self, location: &UniformLocation) {
        unsafe {
            verify!(Context::get().uniform_matrix3fv(Some(location), false, self.matrix()));
        }
    }
}

impl GLPrimitive for Matrix4<f32> {
    #[inline]
    fn gl_type(_: Option<Matrix4<f32>>) -> u32 {
        Context::FLOAT
    }

    #[inline]
    fn size(_: Option<Matrix4<f32>>) -> u32 {
        16
    }

    #[inline]
    fn upload(&self, location: &UniformLocation) {
        unsafe {
            verify!(Context::get().uniform_matrix4fv(Some(location), false, self));
        }
    }
}

/*
 *
 * Impl for vectors
 *
 */
impl GLPrimitive for Vector3<f32> {
    #[inline]
    fn gl_type(_: Option<Vector3<f32>>) -> u32 {
        Context::FLOAT
    }

    #[inline]
    fn size(_: Option<Vector3<f32>>) -> u32 {
        3
    }

    #[inline]
    fn upload(&self, location: &UniformLocation) {
        verify!(Context::get().uniform3f(Some(location), self.x, self.y, self.z));
    }
}

impl GLPrimitive for Vector2<f32> {
    #[inline]
    fn gl_type(_: Option<Vector2<f32>>) -> u32 {
        Context::FLOAT
    }

    #[inline]
    fn size(_: Option<Vector2<f32>>) -> u32 {
        2
    }

    #[inline]
    fn upload(&self, location: &UniformLocation) {
        verify!(Context::get().uniform2f(Some(location), self.x, self.y));
    }
}

// impl GLPrimitive for Vector2<u32> {
//     #[inline]
//     fn gl_type(_: Option<Vector2<u32>>) -> u32 {
//         gl::UNSIGNED_INT
//     }

//     #[inline]
//     fn size(_: Option<Vector2<u32>>) -> u32 {
//         2
//     }

//     #[inline]
//     fn upload(&self, location: &UniformLocation) {
//         verify!(Context::get().uniform2ui(Some(location), self.x, self.y));
//     }
// }

// impl GLPrimitive for Vector3<u32> {
//     #[inline]
//     fn gl_type(_: Option<Vector3<u32>>) -> u32 {
//         gl::UNSIGNED_INT
//     }

//     #[inline]
//     fn size(_: Option<Vector3<u32>>) -> u32 {
//         3
//     }

//     #[inline]
//     fn upload(&self, location: &UniformLocation) {
//         verify!(Context::get().uniform3ui(Some(location), self.x, self.y, self.z));
//     }
// }

/*
 *
 * Impl for points
 *
 */
impl GLPrimitive for Point3<f32> {
    #[inline]
    fn gl_type(_: Option<Point3<f32>>) -> u32 {
        Context::FLOAT
    }

    #[inline]
    fn size(_: Option<Point3<f32>>) -> u32 {
        3
    }

    #[inline]
    fn upload(&self, location: &UniformLocation) {
        verify!(Context::get().uniform3f(Some(location), self.x, self.y, self.z));
    }
}

impl GLPrimitive for Point2<f32> {
    #[inline]
    fn gl_type(_: Option<Point2<f32>>) -> u32 {
        Context::FLOAT
    }

    #[inline]
    fn size(_: Option<Point2<f32>>) -> u32 {
        2
    }

    #[inline]
    fn upload(&self, location: &UniformLocation) {
        verify!(Context::get().uniform2f(Some(location), self.x, self.y));
    }
}

// impl GLPrimitive for Point2<u32> {
//     #[inline]
//     fn gl_type(_: Option<Point2<u32>>) -> u32 {
//         gl::UNSIGNED_INT
//     }

//     #[inline]
//     fn size(_: Option<Point2<u32>>) -> u32 {
//         2
//     }

//     #[inline]
//     fn upload(&self, location: &UniformLocation) {
//         verify!(Context::get().uniform2ui(Some(location), self.x, self.y));
//     }
// }

// impl GLPrimitive for Point3<u32> {
//     #[inline]
//     fn gl_type(_: Option<Point3<u32>>) -> u32 {
//         gl::UNSIGNED_INT
//     }

//     #[inline]
//     fn size(_: Option<Point3<u32>>) -> u32 {
//         3
//     }

//     #[inline]
//     fn upload(&self, location: &UniformLocation) {
//         verify!(Context::get().uniform3ui(Some(location), self.x, self.y, self.z));
//     }
// }

/*
 *
 * Impl for tuples
 *
 */
impl GLPrimitive for (f32, f32, f32) {
    #[inline]
    fn gl_type(_: Option<(f32, f32, f32)>) -> u32 {
        Context::FLOAT
    }

    #[inline]
    fn size(_: Option<(f32, f32, f32)>) -> u32 {
        3
    }

    #[inline]
    fn upload(&self, location: &UniformLocation) {
        verify!(Context::get().uniform3f(Some(location), self.0, self.1, self.2));
    }
}

impl GLPrimitive for (f32, f32) {
    #[inline]
    fn gl_type(_: Option<(f32, f32)>) -> u32 {
        Context::FLOAT
    }

    #[inline]
    fn size(_: Option<(f32, f32)>) -> u32 {
        2
    }

    #[inline]
    fn upload(&self, location: &UniformLocation) {
        verify!(Context::get().uniform2f(Some(location), self.0, self.1));
    }
}

// impl GLPrimitive for (u32, u32) {
//     #[inline]
//     fn gl_type(_: Option<(u32, u32)>) -> u32 {
//         gl::UNSIGNED_INT
//     }

//     #[inline]
//     fn size(_: Option<(u32, u32)>) -> u32 {
//         2
//     }

//     #[inline]
//     fn upload(&self, location: &UniformLocation) {
//         verify!(Context::get().uniform2ui(Some(location), self.0, self.1));
//     }
// }

// impl GLPrimitive for (u32, u32, u32) {
//     #[inline]
//     fn gl_type(_: Option<(u32, u32, u32)>) -> u32 {
//         gl::UNSIGNED_INT
//     }

//     #[inline]
//     fn size(_: Option<(u32, u32, u32)>) -> u32 {
//         3
//     }

//     #[inline]
//     fn upload(&self, location: &UniformLocation) {
//         verify!(Context::get().uniform3ui(Some(location), self.0, self.1, self.2));
//     }
// }
