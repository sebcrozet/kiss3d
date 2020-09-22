//! Trait implemented by materials.

use crate::camera::Camera;
use crate::light::Light;
use crate::planar_camera::PlanarCamera;
use crate::resource::{Mesh, PlanarMesh};
use crate::scene::{ObjectData, PlanarObjectData};
use na::{Isometry2, Isometry3, Vector2, Vector3};

/// Trait implemented by materials.
pub trait Material {
    // FIXME: add the number of the current pass?
    /// Renders an object using this material.
    fn render(
        &mut self,
        pass: usize,
        transform: &Isometry3<f32>,
        scale: &Vector3<f32>,
        camera: &mut dyn Camera, // FIXME: replace those two arguments by
        light: &Light,           // a structure with all environment datas
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
        camera: &mut dyn PlanarCamera, // FIXME: replace those two arguments by
        data: &PlanarObjectData,
        mesh: &mut PlanarMesh,
    );
}
