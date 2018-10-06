//! A resource manager to load textures.

use image::{self, DynamicImage};
use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use context::{Context, Texture};

#[path = "../error.rs"]
mod error;

/// Wrapping parameters for a texture.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum TextureWrapping {
    /// Repeats the texture when a texture coordinate is out of bounds.
    Repeat,
    /// Repeats the mirrored texture when a texture coordinate is out of bounds.
    MirroredRepeat,
    /// Repeats the nearest edge point texture color when a texture coordinate is out of bounds.
    ClampToEdge,
}

impl Into<u32> for TextureWrapping {
    #[inline]
    fn into(self) -> u32 {
        match self {
            TextureWrapping::Repeat => Context::REPEAT,
            TextureWrapping::MirroredRepeat => Context::MIRRORED_REPEAT,
            TextureWrapping::ClampToEdge => Context::CLAMP_TO_EDGE
        }
    }
}

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

    /// Sets the wrapping of this texture along the `s` texture coordinate.
    pub fn set_wrapping_s(&mut self, wrapping: TextureWrapping) {
        let ctxt = Context::get();
        verify!(ctxt.bind_texture(Context::TEXTURE_2D, Some(&self)));
        let wrap: u32 = wrapping.into();
        verify!(ctxt.tex_parameteri(
                Context::TEXTURE_2D,
                Context::TEXTURE_WRAP_S,
                wrap as i32
            ));
    }

    /// Sets the wrapping of this texture along the `t` texture coordinate.
    pub fn set_wrapping_t(&mut self, wrapping: TextureWrapping) {
        let ctxt = Context::get();
        verify!(ctxt.bind_texture(Context::TEXTURE_2D, Some(&self)));
        let wrap: u32 = wrapping.into();
        verify!(ctxt.tex_parameteri(
                Context::TEXTURE_2D,
                Context::TEXTURE_WRAP_T,
                wrap as i32
            ));
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            let ctxt = Context::get();
            if ctxt.is_texture(Some(self)) {
                verify!(Context::get().delete_texture(Some(self)));
            }
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

    /// Allocates a new texture read from a `DynamicImage` object.
    /// 
    /// If a texture with same name exists, nothing is created and the old texture is returned.
    pub fn add_image(&mut self, dynamic_image: DynamicImage, name: &str) -> Rc<Texture> {
        self.textures
        .entry(name.to_string())
        .or_insert_with(|| TextureManager::load_texture_into_context(dynamic_image).unwrap())
        .clone()
    }

    /// Allocates a new texture and tries to decode it from bytes array
    /// Panics if unable to do so
    /// If a texture with same name exists, nothing is created and the old texture is returned.
    pub fn add_image_from_memory(&mut self, image_data: &[u8], name: &str) -> Rc<Texture> {
        self.add_image(image::load_from_memory(image_data).expect("Invalid data"), name)
    }

    /// Allocates a new texture read from a file.
    fn load_texture_from_file(path: &Path) -> Rc<Texture> {
        TextureManager::load_texture_into_context(image::open(path).unwrap())
        .expect(path.to_str().unwrap())
    }

    fn load_texture_into_context(dynamic_image: DynamicImage) -> Result<Rc<Texture>, &'static str> {
        let ctxt = Context::get();
        let tex = Texture::new();

        unsafe {
            verify!(ctxt.active_texture(Context::TEXTURE0));
            verify!(ctxt.bind_texture(Context::TEXTURE_2D, Some(&*tex)));

            match dynamic_image {
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
                    return Err("Failed to load texture, unsuported pixel format.");
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
        Ok(tex)
    }

    /// Allocates a new texture read from a file. If a texture with same name exists, nothing is
    /// created and the old texture is returned.
    pub fn add(&mut self, path: &Path, name: &str) -> Rc<Texture> {
        self.textures
            .entry(name.to_string())
            .or_insert_with(|| TextureManager::load_texture_from_file(path))
            .clone()
    }
}
