//! Structures that a gpu buffer may contain.

use std::mem;
use gl::{self, types::*};
use na::{Point2, Point3, Vector2, Vector3, Matrix2, Matrix3, Matrix4, Rotation2, Rotation3};

#[path = "../error.rs"]
mod error;

/// Trait implemented by structures that can be uploaded to a uniform or contained by a gpu array.
pub trait GLPrimitive: Copy {
    /// The opengl primitive type of this structure content.
    fn gl_type(_type: Option<Self>) -> GLuint;
    /// The number of elements of type `self.gl_type()` this structure stores.
    fn size(_type: Option<Self>) -> GLuint;
    /// Uploads the element to a gpu location.
    fn upload(&self, location: GLuint);
}

/*
 *
 * Impl for primitive types
 *
 */
impl GLPrimitive for GLfloat {
    #[inline]
    fn gl_type(_: Option<GLfloat>) -> GLuint {
        gl::FLOAT
    }

    #[inline]
    fn size(_: Option<GLfloat>) -> GLuint {
        1
    }

    #[inline]
    fn upload(&self, location: GLuint) {
        verify!(gl::Uniform1f(location as GLint, *self));
    }
}

impl GLPrimitive for GLint {
    #[inline]
    fn gl_type(_: Option<GLint>) -> GLuint {
        gl::INT
    }

    #[inline]
    fn size(_: Option<GLint>) -> GLuint {
        1
    }

    #[inline]
    fn upload(&self, location: GLuint) {
        verify!(gl::Uniform1i(location as GLint, *self));
    }
}

impl GLPrimitive for GLuint {
    #[inline]
    fn gl_type(_: Option<GLuint>) -> GLuint {
        gl::UNSIGNED_INT
    }

    #[inline]
    fn size(_: Option<GLuint>) -> GLuint {
        1
    }

    #[inline]
    fn upload(&self, location: GLuint) {
        verify!(gl::Uniform1ui(location as GLint, *self));
    }
}

/*
 *
 * Impl for matrices
 *
 */
impl GLPrimitive for Matrix2<GLfloat> {
    #[inline]
    fn gl_type(_: Option<Matrix2<GLfloat>>) -> GLuint {
        gl::FLOAT
    }

    #[inline]
    fn size(_: Option<Matrix2<GLfloat>>) -> GLuint {
        4
    }

    #[inline]
    fn upload(&self, location: GLuint) {
        unsafe {
            verify!(gl::UniformMatrix2fv(location as GLint, 1, gl::FALSE, mem::transmute(self)));
        }
    }
}

impl GLPrimitive for Rotation2<GLfloat> {
    #[inline]
    fn gl_type(_: Option<Rotation2<GLfloat>>) -> GLuint {
        gl::FLOAT
    }

    #[inline]
    fn size(_: Option<Rotation2<GLfloat>>) -> GLuint {
        4
    }

    #[inline]
    fn upload(&self, location: GLuint) {
        unsafe {
            verify!(gl::UniformMatrix2fv(location as GLint, 1, gl::FALSE, mem::transmute(self)));
        }
    }
}

impl GLPrimitive for Matrix3<GLfloat> {
    #[inline]
    fn gl_type(_: Option<Matrix3<GLfloat>>) -> GLuint {
        gl::FLOAT
    }

    #[inline]
    fn size(_: Option<Matrix3<GLfloat>>) -> GLuint {
        9
    }

    #[inline]
    fn upload(&self, location: GLuint) {
        unsafe {
            verify!(gl::UniformMatrix3fv(location as GLint, 1, gl::FALSE, mem::transmute(self)));
        }
    }
}

impl GLPrimitive for Rotation3<GLfloat> {
    #[inline]
    fn gl_type(_: Option<Rotation3<GLfloat>>) -> GLuint {
        gl::FLOAT
    }

    #[inline]
    fn size(_: Option<Rotation3<GLfloat>>) -> GLuint {
        9
    }

    #[inline]
    fn upload(&self, location: GLuint) {
        unsafe {
            verify!(gl::UniformMatrix3fv(location as GLint, 1, gl::FALSE, mem::transmute(self)));
        }
    }
}

impl GLPrimitive for Matrix4<GLfloat> {
    #[inline]
    fn gl_type(_: Option<Matrix4<GLfloat>>) -> GLuint {
        gl::FLOAT
    }

    #[inline]
    fn size(_: Option<Matrix4<GLfloat>>) -> GLuint {
        16
    }

    #[inline]
    fn upload(&self, location: GLuint) {
        unsafe {
            verify!(gl::UniformMatrix4fv(location as GLint, 1, gl::FALSE, mem::transmute(self)));
        }
    }
}
/*
 *
 * Impl for vectors
 *
 */
impl GLPrimitive for Vector3<GLfloat> {
    #[inline]
    fn gl_type(_: Option<Vector3<GLfloat>>) -> GLuint {
        gl::FLOAT
    }

    #[inline]
    fn size(_: Option<Vector3<GLfloat>>) -> GLuint {
        3
    }

    #[inline]
    fn upload(&self, location: GLuint) {
        verify!(gl::Uniform3f(location as GLint, self.x, self.y, self.z));
    }
}

impl GLPrimitive for Vector2<GLfloat> {
    #[inline]
    fn gl_type(_: Option<Vector2<GLfloat>>) -> GLuint {
        gl::FLOAT
    }

    #[inline]
    fn size(_: Option<Vector2<GLfloat>>) -> GLuint {
        2
    }

    #[inline]
    fn upload(&self, location: GLuint) {
        verify!(gl::Uniform2f(location as GLint, self.x, self.y));
    }
}

impl GLPrimitive for Vector2<GLuint> {
    #[inline]
    fn gl_type(_: Option<Vector2<GLuint>>) -> GLuint {
        gl::UNSIGNED_INT
    }

    #[inline]
    fn size(_: Option<Vector2<GLuint>>) -> GLuint {
        2
    }

    #[inline]
    fn upload(&self, location: GLuint) {
        verify!(gl::Uniform2ui(location as GLint, self.x, self.y));
    }
}

impl GLPrimitive for Vector3<GLuint> {
    #[inline]
    fn gl_type(_: Option<Vector3<GLuint>>) -> GLuint {
        gl::UNSIGNED_INT
    }

    #[inline]
    fn size(_: Option<Vector3<GLuint>>) -> GLuint {
        3
    }

    #[inline]
    fn upload(&self, location: GLuint) {
        verify!(gl::Uniform3ui(location as GLint, self.x, self.y, self.z));
    }
}

/*
 *
 * Impl for points
 *
 */
impl GLPrimitive for Point3<GLfloat> {
    #[inline]
    fn gl_type(_: Option<Point3<GLfloat>>) -> GLuint {
        gl::FLOAT
    }

    #[inline]
    fn size(_: Option<Point3<GLfloat>>) -> GLuint {
        3
    }

    #[inline]
    fn upload(&self, location: GLuint) {
        verify!(gl::Uniform3f(location as GLint, self.x, self.y, self.z));
    }
}

impl GLPrimitive for Point2<GLfloat> {
    #[inline]
    fn gl_type(_: Option<Point2<GLfloat>>) -> GLuint {
        gl::FLOAT
    }

    #[inline]
    fn size(_: Option<Point2<GLfloat>>) -> GLuint {
        2
    }

    #[inline]
    fn upload(&self, location: GLuint) {
        verify!(gl::Uniform2f(location as GLint, self.x, self.y));
    }
}

impl GLPrimitive for Point2<GLuint> {
    #[inline]
    fn gl_type(_: Option<Point2<GLuint>>) -> GLuint {
        gl::UNSIGNED_INT
    }

    #[inline]
    fn size(_: Option<Point2<GLuint>>) -> GLuint {
        2
    }

    #[inline]
    fn upload(&self, location: GLuint) {
        verify!(gl::Uniform2ui(location as GLint, self.x, self.y));
    }
}

impl GLPrimitive for Point3<GLuint> {
    #[inline]
    fn gl_type(_: Option<Point3<GLuint>>) -> GLuint {
        gl::UNSIGNED_INT
    }

    #[inline]
    fn size(_: Option<Point3<GLuint>>) -> GLuint {
        3
    }

    #[inline]
    fn upload(&self, location: GLuint) {
        verify!(gl::Uniform3ui(location as GLint, self.x, self.y, self.z));
    }
}

/*
 *
 * Impl for tuples
 *
 */
impl GLPrimitive for (GLfloat, GLfloat, GLfloat) {
    #[inline]
    fn gl_type(_: Option<(GLfloat, GLfloat, GLfloat)>) -> GLuint {
        gl::FLOAT
    }

    #[inline]
    fn size(_: Option<(GLfloat, GLfloat, GLfloat)>) -> GLuint {
        3
    }

    #[inline]
    fn upload(&self, location: GLuint) {
        verify!(gl::Uniform3f(location as GLint, self.0, self.1, self.2));
    }
}

impl GLPrimitive for (GLfloat, GLfloat) {
    #[inline]
    fn gl_type(_: Option<(GLfloat, GLfloat)>) -> GLuint {
        gl::FLOAT
    }

    #[inline]
    fn size(_: Option<(GLfloat, GLfloat)>) -> GLuint {
        2
    }

    #[inline]
    fn upload(&self, location: GLuint) {
        verify!(gl::Uniform2f(location as GLint, self.0, self.1));
    }
}

impl GLPrimitive for (GLuint, GLuint) {
    #[inline]
    fn gl_type(_: Option<(GLuint, GLuint)>) -> GLuint {
        gl::UNSIGNED_INT
    }

    #[inline]
    fn size(_: Option<(GLuint, GLuint)>) -> GLuint {
        2
    }

    #[inline]
    fn upload(&self, location: GLuint) {
        verify!(gl::Uniform2ui(location as GLint, self.0, self.1));
    }
}

impl GLPrimitive for (GLuint, GLuint, GLuint) {
    #[inline]
    fn gl_type(_: Option<(GLuint, GLuint, GLuint)>) -> GLuint {
        gl::UNSIGNED_INT
    }

    #[inline]
    fn size(_: Option<(GLuint, GLuint, GLuint)>) -> GLuint {
        3
    }

    #[inline]
    fn upload(&self, location: GLuint) {
        verify!(gl::Uniform3ui(location as GLint, self.0, self.1, self.2));
    }
}
