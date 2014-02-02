//! Built-in geometries, shaders and effects.

pub use builtin::sphere_obj::SPHERE_OBJ;
pub use builtin::cube_obj::CUBE_OBJ;
pub use builtin::cone_obj::CONE_OBJ;
pub use builtin::cylinder_obj::CYLINDER_OBJ;
pub use builtin::capsule_obj::CAPSULE_OBJ;

pub use builtin::object_material::{OBJECT_VERTEX_SRC, OBJECT_FRAGMENT_SRC, ObjectMaterial};

pub mod loader;

mod sphere_obj;
mod cube_obj;
mod cone_obj;
mod cylinder_obj;
mod capsule_obj;

mod object_material;
