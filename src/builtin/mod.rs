//! Built-in geometries, shaders and effects.

pub use self::normals_material::{NormalsMaterial, NORMAL_FRAGMENT_SRC, NORMAL_VERTEX_SRC};
pub use self::object_material::{ObjectMaterial, OBJECT_FRAGMENT_SRC, OBJECT_VERTEX_SRC};
pub use self::uvs_material::{UvsMaterial, UVS_FRAGMENT_SRC, UVS_VERTEX_SRC};

pub use self::planar_object_material::PlanarObjectMaterial;

mod normals_material;
mod object_material;
mod uvs_material;

mod planar_object_material;
