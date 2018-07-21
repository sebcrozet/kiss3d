use std::borrow::Borrow;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Once, ONCE_INIT};

use rusttype;

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
        let font = rusttype::Font::from_bytes(memory.to_vec()).unwrap();
        Some(Rc::new(Font { font }))
    }

    /// Instanciate a default font.
    pub fn default() -> Rc<Font> {
        const DATA: &'static [u8] = include_bytes!("WorkSans-Regular.ttf");
        static mut DEFAULT_FONT_SINGLETON: Option<Rc<Font>> = None;
        static INIT: Once = ONCE_INIT;

        unsafe {
            INIT.call_once(|| {
                DEFAULT_FONT_SINGLETON =
                    Some(Font::from_bytes(DATA).expect("Default font creation failed."));
            });

            DEFAULT_FONT_SINGLETON.clone().unwrap()
        }
    }

    /// The underlying rusttype font.
    #[inline]
    pub fn font(&self) -> &rusttype::Font<'static> {
        &self.font
    }

    /// The unique identifier of the specified font instance.
    #[inline]
    pub fn uid(font: &Rc<Font>) -> usize {
        (*font).borrow() as *const Font as usize
    }
}
