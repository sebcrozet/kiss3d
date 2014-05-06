use nalgebra::na::{Cast, Vec3};
use nalgebra::na;
use procedural::{MeshDescr, ProceduralGenerator};
use procedural;

/// Procedural generator of cubes.
pub struct CubeGenerator<N> {
    extents: Vec3<N>
}

impl<N: Float + Clone> CubeGenerator<N> {
    /// Creates a generator of a cube with given extents.
    pub fn new(extents: Vec3<N>) -> CubeGenerator<N> {
        CubeGenerator {
            extents: extents
        }
    }

    /// Creates a generator of a cube with unit extents.
    pub fn new_unit() -> CubeGenerator<N> {
        CubeGenerator::new(na::one())
    }

    /// The cube extents.
    #[inline]
    pub fn extents(&self) -> Vec3<N> {
        self.extents.clone()
    }

    /// Sets the cube extents.
    #[inline]
    pub fn set_extents(&mut self, extents: Vec3<N>) {
        self.extents = extents
    }
}

impl<N: Float + Clone + Cast<f64>> ProceduralGenerator<N> for CubeGenerator<N> {
    fn generate(&self) -> MeshDescr<N> {
        procedural::cube(&self.extents)
    }
}
