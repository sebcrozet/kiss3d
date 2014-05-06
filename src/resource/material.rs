//! Trait implemented by materials.

use nalgebra::na::{Vec3, Iso3};
use camera::Camera;
use light::Light;
use scene::ObjectData;
use resource::Mesh;

/// Trait implemented by materials.
pub trait Material {
    // FIXME: add the number of the current pass?
    /// Renders an object using this material.
    fn render(&mut self,
              pass:      uint,
              transform: &Iso3<f32>,
              scale:     &Vec3<f32>,
              camera:    &mut Camera,    // FIXME: replace those two arguments by
              light:     &Light,         // a structure with all environment datas
              data:      &ObjectData,
              mesh:      &mut Mesh);
}
