//! Specify the type used for vertex indices, which default to `u16` for wasm compatibility
//! reasons. If you need more than 65535 vertices, enable the `vertex_index_u32` feature.
pub use inner::*;

#[cfg(not(feature = "vertex_index_u32"))]
mod inner {
    /// Defaults to `u16`. If you need more than 65535 vertices, enable the `vertex_index_u32` feature.
    pub type VertexIndex = u16;
    /// Tells the glow Context what type is used as the vertex index.
    pub const VERTEX_INDEX_TYPE: u32 = crate::context::Context::UNSIGNED_SHORT;
}

#[cfg(feature = "vertex_index_u32")]
mod inner {
    /// The type used for vertex indices. The feature `vertex_index_u32` enables `u32` indices.
    pub type VertexIndex = u32;
    /// Tells the glow Context what type is used as the vertex index.
    pub const VERTEX_INDEX_TYPE: u32 = crate::context::Context::UNSIGNED_INT;
}
