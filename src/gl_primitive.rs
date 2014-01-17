//! Structures that a gpu buffer may contain.

use gl;
use gl::types::*;
use nalgebra::na::{Vec2, Vec3};

/// Trait implemented by structures that a gpu buffer may contain.
pub trait GLPrimitive: Pod {
    /// The opengl primitive type of this structure content.
    fn gl_type(Option<Self>) -> GLuint;
    /// The number of elements of type `self.gl_type()` this structure stores.
    fn size(Option<Self>) -> GLuint;
}

/*
 *
 * Impl for 2d and 3d vectors
 *
 */
impl GLPrimitive for Vec3<GLdouble> {
    fn gl_type(_: Option<Vec3<GLdouble>>) -> GLuint {
        gl::DOUBLE
    }

    fn size(_: Option<Vec3<GLdouble>>) -> GLuint {
        3
    }
}

impl GLPrimitive for Vec3<GLfloat> {
    fn gl_type(_: Option<Vec3<GLfloat>>) -> GLuint {
        gl::FLOAT
    }

    fn size(_: Option<Vec3<GLfloat>>) -> GLuint {
        3
    }
}

impl GLPrimitive for Vec2<GLdouble> {
    fn gl_type(_: Option<Vec2<GLdouble>>) -> GLuint {
        gl::DOUBLE
    }

    fn size(_: Option<Vec2<GLdouble>>) -> GLuint {
        2
    }
}

impl GLPrimitive for Vec2<GLfloat> {
    fn gl_type(_: Option<Vec2<GLfloat>>) -> GLuint {
        gl::FLOAT
    }

    fn size(_: Option<Vec2<GLfloat>>) -> GLuint {
        2
    }
}

impl GLPrimitive for Vec2<GLuint> {
    fn gl_type(_: Option<Vec2<GLuint>>) -> GLuint {
        gl::UNSIGNED_INT
    }

    fn size(_: Option<Vec2<GLuint>>) -> GLuint {
        2
    }
}

impl GLPrimitive for Vec3<GLuint> {
    fn gl_type(_: Option<Vec3<GLuint>>) -> GLuint {
        gl::UNSIGNED_INT
    }

    fn size(_: Option<Vec3<GLuint>>) -> GLuint {
        3
    }
}

/*
 *
 * Impl for 2d and 3d tuples
 *
 */
impl GLPrimitive for (GLdouble, GLdouble, GLdouble) {
    fn gl_type(_: Option<(GLdouble, GLdouble, GLdouble)>) -> GLuint {
        gl::DOUBLE
    }

    fn size(_: Option<(GLdouble, GLdouble, GLdouble)>) -> GLuint {
        3
    }
}

impl GLPrimitive for (GLfloat, GLfloat, GLfloat) {
    fn gl_type(_: Option<(GLfloat, GLfloat, GLfloat)>) -> GLuint {
        gl::FLOAT
    }

    fn size(_: Option<(GLfloat, GLfloat, GLfloat)>) -> GLuint {
        3
    }
}

impl GLPrimitive for (GLdouble, GLdouble) {
    fn gl_type(_: Option<(GLdouble, GLdouble)>) -> GLuint {
        gl::DOUBLE
    }

    fn size(_: Option<(GLdouble, GLdouble)>) -> GLuint {
        2
    }
}

impl GLPrimitive for (GLfloat, GLfloat) {
    fn gl_type(_: Option<(GLfloat, GLfloat)>) -> GLuint {
        gl::FLOAT
    }

    fn size(_: Option<(GLfloat, GLfloat)>) -> GLuint {
        2
    }
}

impl GLPrimitive for (GLuint, GLuint) {
    fn gl_type(_: Option<(GLuint, GLuint)>) -> GLuint {
        gl::UNSIGNED_INT
    }

    fn size(_: Option<(GLuint, GLuint)>) -> GLuint {
        2
    }
}

impl GLPrimitive for (GLuint, GLuint, GLuint) {
    fn gl_type(_: Option<(GLuint, GLuint, GLuint)>) -> GLuint {
        gl::UNSIGNED_INT
    }

    fn size(_: Option<(GLuint, GLuint, GLuint)>) -> GLuint {
        3
    }
}
