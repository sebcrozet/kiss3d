//! GPU resource managers

pub use crate::context::Texture;
pub use crate::resource::effect::{Effect, ShaderAttribute, ShaderUniform};
pub use crate::resource::framebuffer_manager::{
    FramebufferManager, OffscreenBuffers, RenderTarget,
};
pub use crate::resource::gl_primitive::{GLPrimitive, PrimitiveArray};
pub use crate::resource::gpu_vector::{AllocationType, BufferType, GPUVec};
pub use crate::resource::material::{Material, PlanarMaterial};
pub use crate::resource::material_manager::MaterialManager;
pub use crate::resource::mesh::Mesh;
pub use crate::resource::mesh_manager::MeshManager;
pub use crate::resource::planar_material_manager::PlanarMaterialManager;
pub use crate::resource::planar_mesh::PlanarMesh;
pub use crate::resource::planar_mesh_manager::PlanarMeshManager;
pub use crate::resource::texture_manager::{TextureManager, TextureWrapping};

mod effect;
mod framebuffer_manager;
mod gl_primitive;
mod gpu_vector;
pub mod material;
mod material_manager;
mod mesh;
mod mesh_manager;
mod planar_material_manager;
mod planar_mesh;
mod planar_mesh_manager;
mod texture_manager;
