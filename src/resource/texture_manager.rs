//! A resource manager to load textures.

use image::{self, DynamicImage};
use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::mem;
use std::path::Path;
use std::rc::Rc;

use context::{Context, Texture};

#[path = "../error.rs"]
mod error;

impl Texture {
    /// Allocates a new texture on the gpu. The texture is not configured.
    pub fn new() -> Rc<Texture> {
        let tex = verify!(
            Context::get()
                .create_texture()
                .expect("Could not create texture.")
        );
        Rc::new(tex)
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            verify!(Context::get().delete_texture(Some(self)));
        }
    }
}

thread_local!(static KEY_TEXTURE_MANAGER: RefCell<TextureManager> = RefCell::new(TextureManager::new()));

/// The texture manager.
///
/// It keeps a cache of already-loaded textures, and can load new textures.
pub struct TextureManager {
    default_texture: Rc<Texture>,
    textures: HashMap<String, Rc<Texture>>,
}

impl TextureManager {
    /// Creates a new texture manager.
    pub fn new() -> TextureManager {
        let ctxt = Context::get();
        let default_tex = Texture::new();
        let default_tex_pixels: [u8; 12] = [255; 12];
        verify!(ctxt.active_texture(Context::TEXTURE0));
        verify!(ctxt.bind_texture(Context::TEXTURE_2D, Some(&*default_tex)));
        // verify!(ctxt.tex_parameteri(Context::TEXTURE_2D, Context::TEXTURE_BASE_LEVEL, 0));
        // verify!(ctxt.tex_parameteri(Context::TEXTURE_2D, Context::TEXTURE_MAX_LEVEL, 0));
        verify!(ctxt.tex_parameteri(
            Context::TEXTURE_2D,
            Context::TEXTURE_WRAP_S,
            Context::REPEAT as i32
        ));
        verify!(ctxt.tex_parameteri(
            Context::TEXTURE_2D,
            Context::TEXTURE_WRAP_T,
            Context::REPEAT as i32
        ));
        verify!(ctxt.tex_parameteri(
            Context::TEXTURE_2D,
            Context::TEXTURE_MAG_FILTER,
            Context::LINEAR as i32
        ));
        verify!(ctxt.tex_parameteri(
            Context::TEXTURE_2D,
            Context::TEXTURE_MIN_FILTER,
            Context::LINEAR_MIPMAP_LINEAR as i32
        ));

        verify!(ctxt.tex_image2d(
            Context::TEXTURE_2D,
            0,
            Context::RGB as i32,
            1,
            1,
            0,
            Context::RGB,
            Some(&default_tex_pixels)
        ));

        TextureManager {
            textures: HashMap::new(),
            default_texture: default_tex,
        }
    }

    /// Mutably applies a function to the texture manager.
    pub fn get_global_manager<T, F: FnMut(&mut TextureManager) -> T>(mut f: F) -> T {
        KEY_TEXTURE_MANAGER.with(|manager| f(&mut *manager.borrow_mut()))
    }

    /// Gets the default, completely white, texture.
    pub fn get_default(&self) -> Rc<Texture> {
        self.default_texture.clone()
    }

    /// Get a texture with the specified name. Returns `None` if the texture is not registered.
    pub fn get(&mut self, name: &str) -> Option<Rc<Texture>> {
        self.textures.get(&name.to_string()).map(|t| t.clone())
    }

    /// Allocates a new texture that is not yet configured.
    ///
    /// If a texture with same name exists, nothing is created and the old texture is returned.
    pub fn add_empty(&mut self, name: &str) -> Rc<Texture> {
        match self.textures.entry(name.to_string()) {
            Entry::Occupied(entry) => entry.into_mut().clone(),
            Entry::Vacant(entry) => entry.insert(Texture::new()).clone(),
        }
    }

    /// Allocates a new texture read from a file.
    fn load_texture(path: &Path) -> Rc<Texture> {
        let ctxt = Context::get();
        let tex = Texture::new();

        unsafe {
            verify!(ctxt.active_texture(Context::TEXTURE0));
            verify!(ctxt.bind_texture(Context::TEXTURE_2D, Some(&*tex)));

            match image::open(path).unwrap() {
                DynamicImage::ImageRgb8(image) => {
                    verify!(ctxt.tex_image2d(
                        Context::TEXTURE_2D,
                        0,
                        Context::RGB as i32,
                        image.width() as i32,
                        image.height() as i32,
                        0,
                        Context::RGB,
                        Some(&image.into_raw()[..])
                    ));
                }
                DynamicImage::ImageRgba8(image) => {
                    verify!(ctxt.tex_image2d(
                        Context::TEXTURE_2D,
                        0,
                        Context::RGBA as i32,
                        image.width() as i32,
                        image.height() as i32,
                        0,
                        Context::RGBA,
                        Some(&image.into_raw()[..])
                    ));
                }
                _ => {
                    panic!(
                        "Failed to load texture {}, unsuported pixel format.",
                        path.to_str().unwrap()
                    );
                }
            }

            verify!(ctxt.tex_parameteri(
                Context::TEXTURE_2D,
                Context::TEXTURE_WRAP_S,
                Context::CLAMP_TO_EDGE as i32
            ));
            verify!(ctxt.tex_parameteri(
                Context::TEXTURE_2D,
                Context::TEXTURE_WRAP_T,
                Context::CLAMP_TO_EDGE as i32
            ));
            verify!(ctxt.tex_parameteri(
                Context::TEXTURE_2D,
                Context::TEXTURE_MIN_FILTER,
                Context::LINEAR as i32
            ));
            verify!(ctxt.tex_parameteri(
                Context::TEXTURE_2D,
                Context::TEXTURE_MAG_FILTER,
                Context::LINEAR as i32
            ));
        }

        tex
    }

    /// Allocates a new texture read from a file. If a texture with same name exists, nothing is
    /// created and the old texture is returned.
    pub fn add(&mut self, path: &Path, name: &str) -> Rc<Texture> {
        self.textures
            .entry(name.to_string())
            .or_insert_with(|| TextureManager::load_texture(path))
            .clone()
    }
}
