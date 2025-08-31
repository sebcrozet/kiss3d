use std::borrow::Borrow;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::{Arc,  OnceLock};

use rusttype;


/// A ttf font.
pub struct Font {
    font: rusttype::Font<'static>,
}

impl Font {
    /// Loads a new ttf font from a file.
    pub fn new(path: &Path) -> Option<Font> {
        let mut memory = Vec::new();
        let mut file = File::open(path).unwrap();
        let _ = file.read_to_end(&mut memory).unwrap();
        Font::from_bytes(&memory)
    }

    /// Loads a new ttf font from the memory.
    pub fn from_bytes(memory: &[u8]) -> Option<Font> {
        let font = rusttype::Font::from_bytes(memory.to_vec()).unwrap();
        Some(Font { font })
    }

    /// Instanciate a default font.
    pub fn default() -> Arc<Font> {
        const DATA: &[u8] = include_bytes!("WorkSans-Regular.ttf");
        static DEFAULT_FONT_SINGLETON: OnceLock<Arc<Font>> = OnceLock::new();

        DEFAULT_FONT_SINGLETON.get_or_init(|| {
            Arc::new(
                Font::from_bytes(DATA).expect("Default font creation failed.")
            )
        }).clone()
    }

    /// The underlying rusttype font.
    #[inline]
    pub fn font(&self) -> &rusttype::Font<'static> {
        &self.font
    }

    /// The unique identifier of the specified font instance.
    #[inline]
    pub fn uid(font: &Arc<Font>) -> usize {
        (*font).borrow() as *const Font as usize
    }
}
