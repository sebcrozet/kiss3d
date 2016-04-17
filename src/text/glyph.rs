// This whole file is strongly inspired by: https://github.com/jeaye/q3/blob/master/src/client/ui/ttf/glyph.rs
// available under the BSD-3 licence.
// It has been modified to work with gl-rs, nalgebra, and rust-freetype

use na::Vector2;

/// A ttf glyph.
pub struct Glyph {
    #[doc(hidden)]
    pub tex:        Vector2<f32>,
    #[doc(hidden)]
    pub advance:    Vector2<f32>,
    #[doc(hidden)]
    pub dimensions: Vector2<f32>,
    #[doc(hidden)]
    pub offset:     Vector2<f32>,
    #[doc(hidden)]
    pub buffer:     Vec<u8>
}

impl Glyph {
    /// Creates a new empty glyph.
    pub fn new(tex:        Vector2<f32>,
               advance:    Vector2<f32>,
               dimensions: Vector2<f32>,
               offset:     Vector2<f32>,
               buffer:     Vec<u8>)
               -> Glyph {
        Glyph {
            tex:        tex,
            advance:    advance,
            dimensions: dimensions,
            offset:     offset,
            buffer:     buffer
        }
    }
}
