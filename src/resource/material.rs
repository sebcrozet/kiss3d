//! Trait implemented by materials.

use camera::{Camera, PlanarCamera};
use light::Light;
use na::{Isometry2, Isometry3, Vector2, Vector3};
use resource::{Mesh, PlanarMesh};
use scene::{ObjectData, ObjectData2};

/// Trait implemented by materials.
pub trait Material {
    // FIXME: add the number of the current pass?
    /// Renders an object using this material.
    fn render(
        &mut self,
        pass: usize,
        transform: &Isometry3<f32>,
        scale: &Vector3<f32>,
        camera: &mut Camera, // FIXME: replace those two arguments by
        light: &Light,       // a structure with all environment datas
        data: &ObjectData,
        mesh: &mut Mesh,
    );
}

/// A material for 2D objects.
pub trait PlanarMaterial {
    /// Render the given planar mesh using this material.
    fn render(
        &mut self,
        transform: &Isometry2<f32>,
        scale: &Vector2<f32>,
        camera: &mut PlanarCamera, // FIXME: replace those two arguments by
        data: &ObjectData2,
        mesh: &mut PlanarMesh,
    );
}
