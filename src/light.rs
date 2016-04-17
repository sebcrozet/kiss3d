//! Lights.

use na::Point3;
use gl::types::GLfloat;

/// The light configuration.
#[derive(Clone)]
pub enum Light {
    /// A light with an absolute world position.
    Absolute(Point3<GLfloat>),
    /// A light superimposed with the camera position.
    StickToCamera
}

