//! Built-in geometries, shaders and effects.

pub use builtin::object_material::{OBJECT_VERTEX_SRC, OBJECT_FRAGMENT_SRC, ObjectMaterial};
pub use builtin::normals_material::{NORMAL_VERTEX_SRC, NORMAL_FRAGMENT_SRC, NormalsMaterial};
pub use builtin::uvs_material::{UVS_VERTEX_SRC, UVS_FRAGMENT_SRC, UvsMaterial};

mod object_material;
mod normals_material;
mod uvs_material;
