//! Structures that a gpu buffer may contain.

use std::mem;
use gl;
use gl::types::*;
use na::{Pnt2, Pnt3, Vec2, Vec3, Mat2, Mat3, Mat4, Rot2, Rot3};

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
        verify!(gl::Uniform1f(location as GLint, self.clone()));
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
        verify!(gl::Uniform1i(location as GLint, self.clone()));
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
        verify!(gl::Uniform1ui(location as GLint, self.clone()));
    }
}

/*
 *
 * Impl for matrices
 *
 */
impl GLPrimitive for Mat2<GLfloat> {
    #[inline]
    fn gl_type(_: Option<Mat2<GLfloat>>) -> GLuint {
        gl::FLOAT
    }

    #[inline]
    fn size(_: Option<Mat2<GLfloat>>) -> GLuint {
        4
    }

    #[inline]
    fn upload(&self, location: GLuint) {
        unsafe {
            verify!(gl::UniformMatrix2fv(location as GLint, 1, gl::FALSE, mem::transmute(self)));
        }
    }
}

impl GLPrimitive for Rot2<GLfloat> {
    #[inline]
    fn gl_type(_: Option<Rot2<GLfloat>>) -> GLuint {
        gl::FLOAT
    }

    #[inline]
    fn size(_: Option<Rot2<GLfloat>>) -> GLuint {
        4
    }

    #[inline]
    fn upload(&self, location: GLuint) {
        unsafe {
            verify!(gl::UniformMatrix2fv(location as GLint, 1, gl::FALSE, mem::transmute(self)));
        }
    }
}

impl GLPrimitive for Mat3<GLfloat> {
    #[inline]
    fn gl_type(_: Option<Mat3<GLfloat>>) -> GLuint {
        gl::FLOAT
    }

    #[inline]
    fn size(_: Option<Mat3<GLfloat>>) -> GLuint {
        9
    }

    #[inline]
    fn upload(&self, location: GLuint) {
        unsafe {
            verify!(gl::UniformMatrix3fv(location as GLint, 1, gl::FALSE, mem::transmute(self)));
        }
    }
}

impl GLPrimitive for Rot3<GLfloat> {
    #[inline]
    fn gl_type(_: Option<Rot3<GLfloat>>) -> GLuint {
        gl::FLOAT
    }

    #[inline]
    fn size(_: Option<Rot3<GLfloat>>) -> GLuint {
        9
    }

    #[inline]
    fn upload(&self, location: GLuint) {
        unsafe {
            verify!(gl::UniformMatrix3fv(location as GLint, 1, gl::FALSE, mem::transmute(self)));
        }
    }
}

impl GLPrimitive for Mat4<GLfloat> {
    #[inline]
    fn gl_type(_: Option<Mat4<GLfloat>>) -> GLuint {
        gl::FLOAT
    }

    #[inline]
    fn size(_: Option<Mat4<GLfloat>>) -> GLuint {
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
impl GLPrimitive for Vec3<GLfloat> {
    #[inline]
    fn gl_type(_: Option<Vec3<GLfloat>>) -> GLuint {
        gl::FLOAT
    }

    #[inline]
    fn size(_: Option<Vec3<GLfloat>>) -> GLuint {
        3
    }

    #[inline]
    fn upload(&self, location: GLuint) {
        verify!(gl::Uniform3f(location as GLint, self.x, self.y, self.z));
    }
}

impl GLPrimitive for Vec2<GLfloat> {
    #[inline]
    fn gl_type(_: Option<Vec2<GLfloat>>) -> GLuint {
        gl::FLOAT
    }

    #[inline]
    fn size(_: Option<Vec2<GLfloat>>) -> GLuint {
        2
    }

    #[inline]
    fn upload(&self, location: GLuint) {
        verify!(gl::Uniform2f(location as GLint, self.x, self.y));
    }
}

impl GLPrimitive for Vec2<GLuint> {
    #[inline]
    fn gl_type(_: Option<Vec2<GLuint>>) -> GLuint {
        gl::UNSIGNED_INT
    }

    #[inline]
    fn size(_: Option<Vec2<GLuint>>) -> GLuint {
        2
    }

    #[inline]
    fn upload(&self, location: GLuint) {
        verify!(gl::Uniform2ui(location as GLint, self.x, self.y));
    }
}

impl GLPrimitive for Vec3<GLuint> {
    #[inline]
    fn gl_type(_: Option<Vec3<GLuint>>) -> GLuint {
        gl::UNSIGNED_INT
    }

    #[inline]
    fn size(_: Option<Vec3<GLuint>>) -> GLuint {
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
impl GLPrimitive for Pnt3<GLfloat> {
    #[inline]
    fn gl_type(_: Option<Pnt3<GLfloat>>) -> GLuint {
        gl::FLOAT
    }

    #[inline]
    fn size(_: Option<Pnt3<GLfloat>>) -> GLuint {
        3
    }

    #[inline]
    fn upload(&self, location: GLuint) {
        verify!(gl::Uniform3f(location as GLint, self.x, self.y, self.z));
    }
}

impl GLPrimitive for Pnt2<GLfloat> {
    #[inline]
    fn gl_type(_: Option<Pnt2<GLfloat>>) -> GLuint {
        gl::FLOAT
    }

    #[inline]
    fn size(_: Option<Pnt2<GLfloat>>) -> GLuint {
        2
    }

    #[inline]
    fn upload(&self, location: GLuint) {
        verify!(gl::Uniform2f(location as GLint, self.x, self.y));
    }
}

impl GLPrimitive for Pnt2<GLuint> {
    #[inline]
    fn gl_type(_: Option<Pnt2<GLuint>>) -> GLuint {
        gl::UNSIGNED_INT
    }

    #[inline]
    fn size(_: Option<Pnt2<GLuint>>) -> GLuint {
        2
    }

    #[inline]
    fn upload(&self, location: GLuint) {
        verify!(gl::Uniform2ui(location as GLint, self.x, self.y));
    }
}

impl GLPrimitive for Pnt3<GLuint> {
    #[inline]
    fn gl_type(_: Option<Pnt3<GLuint>>) -> GLuint {
        gl::UNSIGNED_INT
    }

    #[inline]
    fn size(_: Option<Pnt3<GLuint>>) -> GLuint {
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
        verify!(gl::Uniform3f(location as GLint, self.val0(), self.val1(), self.val2()));
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
        verify!(gl::Uniform2f(location as GLint, self.val0(), self.val1()));
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
        verify!(gl::Uniform2ui(location as GLint, self.val0(), self.val1()));
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
        verify!(gl::Uniform3ui(location as GLint, self.val0(), self.val1(), self.val2()));
    }
}
