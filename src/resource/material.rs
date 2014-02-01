//! Materials definition and shader-related tools.

use camera::Camera;
use light::Light;
use object::ObjectData;
use resource::Mesh;

#[path = "../error.rs"]
mod error;

/// Trait implemented by materials.
pub trait Material {
    // FIXME: add the number of the current pass?
    /// Makes the material active.
    fn render(&mut self,
              pass:   uint,
              camera: &mut Camera,    // FIXME: replace those two arguments by
              light:  &Light,        // a structure with all environment datas
              data:   &ObjectData,
              mesh:   &mut Mesh);
}
