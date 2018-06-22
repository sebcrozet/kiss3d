//! GPU resource managers

pub use context::Texture;
pub use resource::effect::{Effect, ShaderAttribute, ShaderUniform};
pub use resource::framebuffer_manager::{FramebufferManager, OffscreenBuffers, RenderTarget};
pub use resource::gl_primitive::{GLPrimitive, PrimitiveArray};
pub use resource::gpu_vector::{AllocationType, BufferType, GPUVec};
pub use resource::material::{Material, Material2};
pub use resource::material_manager::MaterialManager;
pub use resource::material_manager2::MaterialManager2;
pub use resource::mesh::Mesh;
pub use resource::mesh2::Mesh2;
pub use resource::mesh_manager::MeshManager;
pub use resource::mesh_manager2::MeshManager2;
pub use resource::texture_manager::TextureManager;

mod effect;
mod framebuffer_manager;
mod gl_primitive;
mod gpu_vector;
pub mod material;
mod material_manager;
mod material_manager2;
mod mesh;
mod mesh2;
mod mesh_manager;
mod mesh_manager2;
mod texture_manager;
