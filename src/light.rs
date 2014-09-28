//! Lights.

use na::Vec3;
use gl::types::GLfloat;

/// The light configuration.
#[deriving(Clone)]
pub enum Light {
    /// A light with an absolute world position.
    Absolute(Vec3<GLfloat>),
    /// A light superimposed with the camera position.
    StickToCamera
}

