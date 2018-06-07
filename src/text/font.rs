use std::borrow::Borrow;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::rc::Rc;

use rusttype;
use rusttype::gpu_cache::{Cache, CacheBuilder};

use context::{Context, Texture};

#[path = "../error.rs"]
mod error;

/// A ttf font.
pub struct Font {
    font: rusttype::Font<'static>,
}

impl Font {
    /// Loads a new ttf font from a file.
    pub fn new(path: &Path) -> Option<Rc<Font>> {
        let mut memory = Vec::new();
        let mut file = File::open(path).unwrap();
        let _ = file.read_to_end(&mut memory).unwrap();
        Font::from_bytes(&memory)
    }

    /// Loads a new ttf font from the memory.
    pub fn from_bytes(memory: &[u8]) -> Option<Rc<Font>> {
        let ctxt = Context::get();
        let font = rusttype::Font::from_bytes(memory.to_vec()).unwrap();

        Some(Rc::new(Font { font }))
    }

    #[inline]
    pub fn font(&self) -> &rusttype::Font<'static> {
        &self.font
    }

    #[inline]
    pub fn uid(font: &Rc<Font>) -> usize {
        (*font).borrow() as *const Font as usize
    }
}
