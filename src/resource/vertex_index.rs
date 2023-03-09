#[cfg(not(feature = "vertex_index_u32"))]
pub type VertexIndex = u16;
#[cfg(feature = "vertex_index_u32")]
pub type VertexIndex = u32;
