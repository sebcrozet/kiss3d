// This whole file is strongly inspired by: https://github.com/jeaye/q3/blob/master/src/client/ui/ttf/glyph.rs
// available under the BSD-3 licence.
// It has been modified to work with gl-rs, nalgebra, and rust-freetype

use na::Vec2;

/// A ttf glyph.
pub struct Glyph {
    #[doc(hidden)]
    pub tex:        Vec2<f32>,
    #[doc(hidden)]
    pub advance:    Vec2<f32>,
    #[doc(hidden)]
    pub dimensions: Vec2<f32>,
    #[doc(hidden)]
    pub offset:     Vec2<f32>,
    #[doc(hidden)]
    pub buffer:     Vec<u8>
}

impl Glyph {
    /// Creates a new empty glyph.
    pub fn new(tex:        Vec2<f32>,
               advance:    Vec2<f32>,
               dimensions: Vec2<f32>,
               offset:     Vec2<f32>,
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
