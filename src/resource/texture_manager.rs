//! A resource manager to load textures.

use image::{self, DynamicImage};
use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use crate::context::{Context, Cubemap, Texture};

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
            TextureWrapping::ClampToEdge => Context::CLAMP_TO_EDGE,
        }
    }
}

/// Cubemap directions
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum CubemapDirection {
    /// +X face
    PositiveX,
    /// -X face
    NegativeX,
    /// +Y face
    PositiveY,
    /// -Y face
    NegativeY,
    /// +Z face
    PositiveZ,
    /// -Z face
    NegativeZ,
}

impl Into<u32> for CubemapDirection {
    #[inline]
    fn into(self) -> u32 {
        match self {
            CubemapDirection::PositiveX => Context::TEXTURE_CUBE_MAP_POSITIVE_X,
            CubemapDirection::NegativeX => Context::TEXTURE_CUBE_MAP_NEGATIVE_X,
            CubemapDirection::PositiveY => Context::TEXTURE_CUBE_MAP_POSITIVE_Y,
            CubemapDirection::NegativeY => Context::TEXTURE_CUBE_MAP_NEGATIVE_Y,
            CubemapDirection::PositiveZ => Context::TEXTURE_CUBE_MAP_POSITIVE_Z,
            CubemapDirection::NegativeZ => Context::TEXTURE_CUBE_MAP_NEGATIVE_Z,
        }
    }
}

impl Cubemap {
    /// Allocates a new texture on the gpu. The texture is not configured.
    pub fn new() -> Rc<Cubemap> {
        let tex = verify!(Context::get()
            .create_cubemap()
            .expect("Could not create texture."));
        Rc::new(tex)
    }

    /// Set the wrappings of this texture for cubemap settings for `s`, `t`, and `r`
    pub fn set_cubemap_wrapping(
        &mut self,
        s: TextureWrapping,
        t: TextureWrapping,
        r: TextureWrapping,
    ) {
        // FIXME: this isn't typesafe right now -- a user could create a texture for a 2D texture
        // and swap it with a cubemap later on.
        let ctxt = Context::get();
        verify!(ctxt.bind_cubemap(Some(&self)));

        let wrap_s: u32 = s.into();
        verify!(ctxt.tex_parameteri(
            Context::TEXTURE_CUBE_MAP,
            Context::TEXTURE_WRAP_S,
            wrap_s as i32
        ));

        let wrap_t: u32 = t.into();
        verify!(ctxt.tex_parameteri(
            Context::TEXTURE_CUBE_MAP,
            Context::TEXTURE_WRAP_T,
            wrap_t as i32
        ));

        let wrap_r: u32 = r.into();
        verify!(ctxt.tex_parameteri(
            Context::TEXTURE_CUBE_MAP,
            Context::TEXTURE_WRAP_R,
            wrap_r as i32
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
        verify!(ctxt.bind_texture(Some(&self)));
        let wrap: u32 = wrapping.into();
        verify!(ctxt.tex_parameteri(Context::TEXTURE_2D, Context::TEXTURE_WRAP_S, wrap as i32));
    }

    /// Sets the wrapping of this texture along the `t` texture coordinate.
    pub fn set_wrapping_t(&mut self, wrapping: TextureWrapping) {
        let ctxt = Context::get();
        verify!(ctxt.bind_texture(Some(&self)));
        let wrap: u32 = wrapping.into();
        verify!(ctxt.tex_parameteri(Context::TEXTURE_2D, Context::TEXTURE_WRAP_T, wrap as i32));
    }
}

impl Drop for Cubemap {
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

#[derive(Clone)]
enum TextureVarient {
    Cubemap(Rc<Cubemap>),
    Texture(Rc<Texture>),
}

impl TextureVarient {
    fn texture(&self) -> Option<Rc<Texture>> {
        match self {
            TextureVarient::Texture(t) => Some(t.clone()),
            _ => None
        }
    }

    fn cubemap(&self) -> Option<Rc<Cubemap>> {
        match self {
            TextureVarient::Cubemap(t) => Some(t.clone()),
            _ => None
        }
    }
}

/// The texture manager.
///
/// It keeps a cache of already-loaded textures, and can load new textures.
pub struct TextureManager {
    default_texture: Rc<Texture>,
    textures: HashMap<String, (TextureVarient, (u32, u32))>,
}

impl TextureManager {
    /// Creates a new texture manager.
    pub fn new() -> TextureManager {
        let ctxt = Context::get();
        let default_tex = Texture::new();
        let default_tex_pixels: [u8; 12] = [255; 12];
        verify!(ctxt.active_texture(Context::TEXTURE0));
        verify!(ctxt.bind_texture(Some(&*default_tex)));
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
        self.textures.get(&name.to_string()).map(|t| t.0.texture().unwrap())
    }

    /// Get a texture (and its size) with the specified name. Returns `None` if the texture is not registered.
    pub fn get_with_size(&mut self, name: &str) -> Option<(Rc<Texture>, (u32, u32))> {
        self.textures
            .get(&name.to_string())
            .map(|t| (t.0.texture().unwrap(), t.1))
    }

    /// Allocates a new texture that is not yet configured.
    ///
    /// If a texture with same name exists, nothing is created and the old texture is returned.
    pub fn add_empty(&mut self, name: &str) -> Rc<Texture> {
        match self.textures.entry(name.to_string()) {
            Entry::Occupied(entry) => entry.into_mut().0.texture().unwrap(),
            Entry::Vacant(entry) => entry.insert((TextureVarient::Texture(Texture::new()), (0, 0))).0.texture().unwrap().clone(),
        }
    }

    /// Allocates a new texture read from a `DynamicImage` object.
    ///
    /// If a texture with same name exists, nothing is created and the old texture is returned.
    pub fn add_image(&mut self, dynamic_image: DynamicImage, name: &str) -> Rc<Texture> {
        self.textures
            .entry(name.to_string())
            .or_insert_with(|| {
                let t = TextureManager::load_texture_into_context(dynamic_image).unwrap();
                (TextureVarient::Texture(t.0), t.1)
            })
            .0.texture().unwrap().clone()
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
    fn load_texture_from_file(path: &Path) -> (Rc<Texture>, (u32, u32)) {
        TextureManager::load_texture_into_context(image::open(path).unwrap())
            .expect(path.to_str().unwrap())
    }

    fn load_cubemap_from_files(
        paths: [&Path; 6],
        directions: [CubemapDirection; 6],
    ) -> (Rc<Cubemap>, (u32, u32)) {
        let imgs: [DynamicImage; 6] = [
            image::open(paths[0]).expect(paths[0].to_str().unwrap()),
            image::open(paths[1]).expect(paths[1].to_str().unwrap()),
            image::open(paths[2]).expect(paths[2].to_str().unwrap()),
            image::open(paths[3]).expect(paths[3].to_str().unwrap()),
            image::open(paths[4]).expect(paths[4].to_str().unwrap()),
            image::open(paths[5]).expect(paths[5].to_str().unwrap()),
        ];

        return TextureManager::load_cubemap_into_context(imgs, directions).unwrap();
    }


    fn load_cubemap_into_context(images: [DynamicImage; 6], directions: [CubemapDirection; 6])
            -> Result<(Rc<Cubemap>, (u32, u32)), &'static str> {
        let ctxt = Context::get();
        let cubemap = Cubemap::new();
        let mut width = 0;
        let mut height = 0;

        unsafe {
            verify!(ctxt.active_texture(Context::TEXTURE0));
            verify!(ctxt.bind_cubemap(Some(&*cubemap)));

            for (dynamic_image, dir) in images.iter().zip(directions.iter()) {
                let u_dir: u32 = (*dir).into();
                match dynamic_image {
                    DynamicImage::ImageRgb8(image) => {
                        width = image.width();
                        height = image.height();

                        verify!(ctxt.tex_image2d(
                            u_dir,
                            0,
                            Context::RGB as i32,
                            image.width() as i32,
                            image.height() as i32,
                            0,
                            Context::RGB,
                            Some(image)
                        ));
                    }
                    DynamicImage::ImageRgba8(image) => {
                        width = image.width();
                        height = image.height();

                        verify!(ctxt.tex_image2d(
                            u_dir,
                            0,
                            Context::RGBA as i32,
                            image.width() as i32,
                            image.height() as i32,
                            0,
                            Context::RGBA,
                            Some(image)
                        ));
                    }
                    _ => {
                        return Err("Failed to load texture, unsuported pixel format.");
                    }
                }
            }

            verify!(ctxt.tex_parameteri(
                Context::TEXTURE_CUBE_MAP,
                Context::TEXTURE_WRAP_S,
                Context::CLAMP_TO_EDGE as i32
            ));
            verify!(ctxt.tex_parameteri(
                Context::TEXTURE_CUBE_MAP,
                Context::TEXTURE_WRAP_T,
                Context::CLAMP_TO_EDGE as i32
            ));
            verify!(ctxt.tex_parameteri(
                Context::TEXTURE_CUBE_MAP,
                Context::TEXTURE_WRAP_R,
                Context::CLAMP_TO_EDGE as i32
            ));
            verify!(ctxt.tex_parameteri(
                Context::TEXTURE_CUBE_MAP,
                Context::TEXTURE_MIN_FILTER,
                Context::LINEAR as i32
            ));
            verify!(ctxt.tex_parameteri(
                Context::TEXTURE_CUBE_MAP,
                Context::TEXTURE_MAG_FILTER,
                Context::LINEAR as i32
            ));
        }
        Ok((cubemap, (width, height)))
    }

    fn load_texture_into_context(
        dynamic_image: DynamicImage,
    ) -> Result<(Rc<Texture>, (u32, u32)), &'static str> {
        let ctxt = Context::get();
        let tex = Texture::new();
        let width;
        let height;

        unsafe {
            verify!(ctxt.active_texture(Context::TEXTURE0));
            verify!(ctxt.bind_texture(Some(&*tex)));

            match dynamic_image {
                DynamicImage::ImageRgb8(image) => {
                    width = image.width();
                    height = image.height();

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
                    width = image.width();
                    height = image.height();

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
        Ok((tex, (width, height)))
    }

    /// Allocates a new texture read from a file. If a texture with same name exists, nothing is
    /// created and the old texture is returned.
    pub fn add(&mut self, path: &Path, name: &str) -> Rc<Texture> {
        self.textures
            .entry(name.to_string())
            .or_insert_with(|| {
                let t = TextureManager::load_texture_from_file(path);
                (TextureVarient::Texture(t.0), t.1)
            })
            .0.texture().unwrap().clone()
    }

    /// Load a cubemap from files
    pub fn add_cubemap(
        &mut self,
        paths: [&Path; 6],
        directions: [CubemapDirection; 6],
        name: &str,
    ) -> Rc<Cubemap> {
        self.textures
            .entry(name.to_string())
            .or_insert_with(|| {
                let t = TextureManager::load_cubemap_from_files(paths, directions);
                (TextureVarient::Cubemap(t.0), t.1)
            })
            .0
            .cubemap().unwrap()
            .clone()
    }
}
