//! GPU resource managers

pub use context::Texture;
pub use resource::effect::{Effect, ShaderAttribute, ShaderUniform};
pub use resource::framebuffer_manager::{FramebufferManager, OffscreenBuffers, RenderTarget};
pub use resource::gl_primitive::{GLPrimitive, PrimitiveArray};
pub use resource::gpu_vector::{AllocationType, BufferType, GPUVec};
pub use resource::material::Material;
pub use resource::material_manager::MaterialManager;
pub use resource::mesh::Mesh;
pub use resource::mesh_manager::MeshManager;
pub use resource::texture_manager::TextureManager;

mod effect;
mod framebuffer_manager;
mod gl_primitive;
mod gpu_vector;
pub mod material;
mod material_manager;
mod mesh;
mod mesh_manager;
mod texture_manager;
