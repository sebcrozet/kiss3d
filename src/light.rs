//! Lights.

use na::Pnt3;
use gl::types::GLfloat;

/// The light configuration.
#[deriving(Clone)]
pub enum Light {
    /// A light with an absolute world position.
    Absolute(Pnt3<GLfloat>),
    /// A light superimposed with the camera position.
    StickToCamera
}

