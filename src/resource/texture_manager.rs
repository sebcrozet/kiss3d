//! A resource manager to load textures.

use image::{self, imageops::FilterType, DynamicImage, GenericImageView};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use crate::{
    context::{Context, Texture},
    verify,
};

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

impl From<TextureWrapping> for u32 {
    #[inline]
    fn from(val: TextureWrapping) -> Self {
        match val {
            TextureWrapping::Repeat => Context::REPEAT,
            TextureWrapping::MirroredRepeat => Context::MIRRORED_REPEAT,
            TextureWrapping::ClampToEdge => Context::CLAMP_TO_EDGE,
        }
    }
}

impl Texture {
    /// Allocates a new texture on the gpu. The texture is not configured.
    pub fn new() -> Rc<Texture> {
        let tex = verify!(Context::get()
            .create_texture()
            .expect("Could not create texture."));
        Rc::new(tex)
    }

    /// Sets the wrapping of this texture along the `s` texture coordinate.
    pub fn set_wrapping_s(&mut self, wrapping: TextureWrapping) {
        let ctxt = Context::get();
        verify!(ctxt.bind_texture(Context::TEXTURE_2D, Some(self)));
        let wrap: u32 = wrapping.into();
        verify!(ctxt.tex_parameteri(Context::TEXTURE_2D, Context::TEXTURE_WRAP_S, wrap as i32));
    }

    /// Sets the wrapping of this texture along the `t` texture coordinate.
    pub fn set_wrapping_t(&mut self, wrapping: TextureWrapping) {
        let ctxt = Context::get();
        verify!(ctxt.bind_texture(Context::TEXTURE_2D, Some(self)));
        let wrap: u32 = wrapping.into();
        verify!(ctxt.tex_parameteri(Context::TEXTURE_2D, Context::TEXTURE_WRAP_T, wrap as i32));
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            let ctxt = Context::get();
            if verify!(ctxt.is_texture(Some(self))) {
                verify!(Context::get().delete_texture(Some(self)));
            }
        }
    }
}

// thread_local!(static KEY_TEXTURE_MANAGER: RefCell<Option<TextureManager>> = RefCell::new(Some(TextureManager::new())));

/// The texture manager.
///
/// It keeps a cache of already-loaded textures, and can load new textures.
pub struct TextureManager {
    default_texture: Rc<Texture>,
    textures: HashMap<String, (Rc<Texture>, (u32, u32))>,
    // If generate_mipmaps is true, mipmaps are generated for textures when they
    // are loaded.
    generate_mipmaps: bool,
}

impl Default for TextureManager {
    fn default() -> Self {
        Self::new()
    }
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
            generate_mipmaps: false,
        }
    }

    /// Mutably applies a function to the texture manager.
    pub fn get_global_manager<T, F: FnMut(&mut TextureManager) -> T>(mut f: F) -> T {
        crate::window::WINDOW_CACHE
            .with(|manager| f(&mut *manager.borrow_mut().texture_manager.as_mut().unwrap()))
    }

    /// Gets the default, completely white, texture.
    pub fn get_default(&self) -> Rc<Texture> {
        self.default_texture.clone()
    }

    /// Get a texture with the specified name. Returns `None` if the texture is not registered.
    pub fn get(&mut self, name: &str) -> Option<Rc<Texture>> {
        self.textures.get(name).map(|t| t.0.clone())
    }

    /// Get a texture (and its size) with the specified name. Returns `None` if the texture is not registered.
    pub fn get_with_size(&mut self, name: &str) -> Option<(Rc<Texture>, (u32, u32))> {
        self.textures.get(name).map(|t| (t.0.clone(), t.1))
    }

    /// Allocates a new texture that is not yet configured.
    ///
    /// If a texture with same name exists, nothing is created and the old texture is returned.
    pub fn add_empty(&mut self, name: &str) -> Rc<Texture> {
        match self.textures.entry(name.to_string()) {
            Entry::Occupied(entry) => entry.into_mut().0.clone(),
            Entry::Vacant(entry) => entry.insert((Texture::new(), (0, 0))).0.clone(),
        }
    }

    /// Allocates a new texture read from a `DynamicImage` object.
    ///
    /// If a texture with same name exists, nothing is created and the old texture is returned.
    pub fn add_image(&mut self, image: DynamicImage, name: &str) -> Rc<Texture> {
        let generate_mipmaps = self.generate_mipmaps;
        self.textures
            .entry(name.to_string())
            .or_insert_with(|| {
                TextureManager::load_texture_into_context(image, generate_mipmaps).unwrap()
            })
            .0
            .clone()
    }

    /// Allocates a new texture and tries to decode it from bytes array
    /// Panics if unable to do so
    /// If a texture with same name exists, nothing is created and the old texture is returned.
    pub fn add_image_from_memory(&mut self, image_data: &[u8], name: &str) -> Rc<Texture> {
        self.add_image(
            image::load_from_memory(image_data).expect("Invalid data"),
            name,
        )
    }

    /// Allocates a new texture read from a file.
    fn load_texture_from_file(path: &Path, generate_mipmaps: bool) -> (Rc<Texture>, (u32, u32)) {
        let image = image::open(path)
            .unwrap_or_else(|e| panic!("Unable to load texture from file {:?}: {:?}", path, e));
        TextureManager::load_texture_into_context(image, generate_mipmaps)
            .unwrap_or_else(|e| panic!("Unable to upload texture {:?}: {:?}", path, e))
    }

    fn load_texture_into_context(
        image: DynamicImage,
        generate_mipmaps: bool,
    ) -> Result<(Rc<Texture>, (u32, u32)), &'static str> {
        let ctxt = Context::get();
        let tex = Texture::new();
        let (width, height) = image.dimensions();

        unsafe {
            verify!(ctxt.active_texture(Context::TEXTURE0));
            verify!(ctxt.bind_texture(Context::TEXTURE_2D, Some(&*tex)));
            TextureManager::call_tex_image2d(&ctxt, &image, 0)?;

            let mut min_filter = Context::LINEAR;
            if generate_mipmaps {
                let (mut w, mut h) = (width, height);
                let mut image = image;

                for level in 1.. {
                    if w == 1 && h == 1 {
                        break;
                    }
                    w = w.div_ceil(2);
                    h = h.div_ceil(2);
                    image = image.resize_exact(w, h, FilterType::CatmullRom);
                    TextureManager::call_tex_image2d(&ctxt, &image, level)?;
                }
                min_filter = Context::LINEAR_MIPMAP_LINEAR;
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
                min_filter as i32,
            ));
            verify!(ctxt.tex_parameteri(
                Context::TEXTURE_2D,
                Context::TEXTURE_MAG_FILTER,
                Context::LINEAR as i32
            ));
        }
        Ok((tex, (width, height)))
    }

    fn call_tex_image2d(
        ctxt: &Context,
        dynamic_image: &DynamicImage,
        level: i32,
    ) -> Result<(), &'static str> {
        let (pixel_format, pixels) = match dynamic_image {
            DynamicImage::ImageRgb8(image) => (Context::RGB, &image.as_raw()[..]),
            DynamicImage::ImageRgba8(image) => (Context::RGBA, &image.as_raw()[..]),
            _ => {
                return Err("Failed to load texture, unsupported pixel format.");
            }
        };
        let (width, height) = dynamic_image.dimensions();

        verify!(ctxt.tex_image2d(
            Context::TEXTURE_2D,
            level,
            pixel_format as i32,
            width as i32,
            height as i32,
            0,
            pixel_format,
            Some(pixels)
        ));
        Ok(())
    }

    /// Allocates a new texture read from a file. If a texture with same name exists, nothing is
    /// created and the old texture is returned.
    pub fn add(&mut self, path: &Path, name: &str) -> Rc<Texture> {
        let generate_mipmaps = self.generate_mipmaps;
        self.textures
            .entry(name.to_string())
            .or_insert_with(|| TextureManager::load_texture_from_file(path, generate_mipmaps))
            .0
            .clone()
    }

    /// Changes whether textures will have mipmaps generated when they are
    /// loaded; does not affect already loaded textures.
    /// Mipmap generation is disabled by default.
    pub fn set_generate_mipmaps(&mut self, enabled: bool) {
        self.generate_mipmaps = enabled;
    }
}
