//! Procedural mesh generation.

pub use self::bezier::{bezier_curve, bezier_curve_at};
pub use self::bezier::{bezier_surface, bezier_surface_at};
pub use self::capsule::capsule;
pub use self::cone::{cone, unit_cone};
pub use self::cuboid::{cuboid, unit_cuboid};
pub use self::cuboid::{rectangle, unit_rectangle};
pub use self::cylinder::{cylinder, unit_cylinder};
pub use self::quad::{quad, quad_with_vertices, unit_quad};
pub use self::render_mesh::{IndexBuffer, RenderMesh};
pub use self::render_polyline::RenderPolyline;
pub use self::sphere::{circle, unit_circle};
pub use self::sphere::{sphere, unit_hemisphere, unit_sphere};

pub mod path;
mod render_mesh;
mod render_polyline;
pub mod utils;

mod bezier;
mod capsule;
mod cone;
mod cuboid;
mod cylinder;
mod quad;
mod sphere;
