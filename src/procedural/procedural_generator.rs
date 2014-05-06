//! Trait implemented by all procedural generators.

use procedural::MeshDescr;

/// Trait implemented by all procedural generators.
pub trait ProceduralGenerator<N> {
    /// Generates a static mesh representation of the geometry.
    fn generate(&self) -> MeshDescr<N>;
}
