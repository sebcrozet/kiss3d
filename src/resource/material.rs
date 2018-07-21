//! Trait implemented by materials.

use camera::Camera;
use light::Light;
use na::{Isometry2, Isometry3, Vector2, Vector3};
use planar_camera::PlanarCamera;
use resource::{Mesh, PlanarMesh};
use scene::{ObjectData, PlanarObjectData};

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
        data: &PlanarObjectData,
        mesh: &mut PlanarMesh,
    );
}
