//! Procedural mesh generation.

pub use procedural::bezier::bezier_surface;
pub use procedural::capsule::capsule;
pub use procedural::cone::{unit_cone, cone};
pub use procedural::cube::{cube, unit_cube};
pub use procedural::cylinder::{unit_cylinder, cylinder};
pub use procedural::mesh_descr::{MeshDescr, IndexBuffer, UnifiedIndexBuffer, SplitIndexBuffer};
pub use procedural::procedural_generator::ProceduralGenerator;
pub use procedural::quad::{quad, unit_quad, quad_with_vertices};
pub use procedural::sphere::{sphere, unit_sphere};

pub use procedural::bezier_generator::BezierGenerator;
pub use procedural::capsule_generator::CapsuleGenerator;
pub use procedural::cone_generator::ConeGenerator;
pub use procedural::cube_generator::CubeGenerator;
pub use procedural::cylinder_generator::CylinderGenerator;
pub use procedural::quad_generator::QuadGenerator;
pub use procedural::sphere_generator::SphereGenerator;

pub mod utils;
pub mod procedural_generator;
mod mesh_descr;

mod bezier;
mod capsule;
mod cone;
mod cube;
mod cylinder;
mod quad;
mod sphere;

mod bezier_generator;
mod capsule_generator;
mod cone_generator;
mod cube_generator;
mod cylinder_generator;
mod quad_generator;
mod sphere_generator;
