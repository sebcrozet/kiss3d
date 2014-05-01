//! Procedural mesh generation.

pub use procedural::mesh_descr::{MeshDescr, IndexBuffer, UnifiedIndexBuffer, SplitIndexBuffer};
pub use procedural::cube::{cube, unit_cube};
pub use procedural::sphere::{sphere, unit_sphere};
pub use procedural::capsule::capsule;
pub use procedural::quad::{quad, unit_quad};

mod utils;
mod mesh_descr;
mod cube;
mod sphere;
mod capsule;
mod quad;
