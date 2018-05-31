//! GPU resource managers

// pub use resource::framebuffer_manager::{FramebufferManager, RenderTarget, OffscreenBuffers};
// pub use resource::texture_manager::{Texture, TextureManager};
// pub use resource::material::Material;
// pub use resource::material_manager::MaterialManager;
// pub use resource::mesh_manager::MeshManager;
pub use resource::effect::{Effect, ShaderAttribute, ShaderUniform};
pub use resource::gl_primitive::{GLPrimitive, PrimitiveArray};
pub use resource::gpu_vector::{AllocationType, BufferType, GPUVec};
pub use resource::mesh::Mesh;

// mod framebuffer_manager;
// mod texture_manager;
// mod mesh_manager;
// mod material_manager;
// pub mod material;
mod effect;
mod gl_primitive;
mod gpu_vector;
mod mesh;
