//! Lights.

use na::Point3;

/// The light configuration.
#[derive(Clone)]
pub enum Light {
    /// A light with an absolute world position.
    Absolute(Point3<f32>),
    /// A light superimposed with the camera position.
    StickToCamera,
}
