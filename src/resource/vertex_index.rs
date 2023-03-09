use crate::context::Context;

#[cfg(not(feature = "vertex_index_u32"))]
pub type VertexIndex = u16;
#[cfg(feature = "vertex_index_u32")]
pub type VertexIndex = u32;

#[cfg(not(feature = "vertex_index_u32"))]
pub const VERTEX_INDEX_TYPE: u32 = Context::UNSIGNED_SHORT;
#[cfg(feature = "vertex_index_u32")]
pub const VERTEX_INDEX_TYPE: u32 = Context::UNSIGNED_INT;
