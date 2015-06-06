//! A resource manager to load textures.

use std::cell::RefCell;
use std::mem;
use std::rc::Rc;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::path::Path;
use gl;
use gl::types::*;
use image::{self, DynamicImage};

#[path = "../error.rs"]
mod error;

/// A gpu texture. It contains the texture id provided by opengl and is automatically released.
pub struct Texture {
    id: GLuint
}

impl Texture {
    /// Allocates a new texture on the gpu. The texture is not configured.
    pub fn new() -> Rc<Texture> {
        let mut id: GLuint = 0;

        unsafe { verify!(gl::GenTextures(1, &mut id)); }

        Rc::new(Texture { id: id })
    }

    /// The opengl-provided texture id.
    pub fn id(&self) -> GLuint {
        self.id
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
       unsafe { verify!(gl::DeleteTextures(1, &self.id)); }
    }
}

thread_local!(static KEY_TEXTURE_MANAGER: RefCell<TextureManager> = RefCell::new(TextureManager::new()));

/// The texture manager.
///
/// It keeps a cache of already-loaded textures, and can load new textures.
pub struct TextureManager {
    default_texture: Rc<Texture>,
    textures:        HashMap<String, Rc<Texture>>,
}

impl TextureManager {
    /// Creates a new texture manager.
    pub fn new() -> TextureManager {
        let default_tex = Texture::new();
        let default_tex_pixels: [ GLfloat; 3 ] = [ 1.0, 1.0, 1.0 ];
        verify!(gl::ActiveTexture(gl::TEXTURE0));
        verify!(gl::BindTexture(gl::TEXTURE_2D, default_tex.id()));
        verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_BASE_LEVEL, 0));
        verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, 0));
        verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32));
        verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32));
        verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32));
        verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32));

        unsafe {
            verify!(gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as i32, 1, 1, 0, gl::RGB, gl::FLOAT,
                                   mem::transmute(&default_tex_pixels[0])));
        }

        TextureManager {
            textures:        HashMap::new(),
            default_texture: default_tex
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
            Entry::Vacant(entry)   => entry.insert(Texture::new()).clone()
        }
    }

    /// Allocates a new texture read from a file. If a texture with same name exists, nothing is
    /// created and the old texture is returned.
    pub fn add(&mut self, path: &Path, name: &str) -> Rc<Texture> {
        let tex = match self.textures.entry(name.to_string()) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry)   => entry.insert(Texture::new())
        };

        // FIXME: dont re-load the texture if it already exists!
        unsafe {
            verify!(gl::ActiveTexture(gl::TEXTURE0));
            verify!(gl::BindTexture(gl::TEXTURE_2D, tex.id()));

            match image::open(path).unwrap() {
                DynamicImage::ImageRgb8(image) => {
                        verify!(gl::TexImage2D(
                                gl::TEXTURE_2D, 0,
                                gl::RGB as GLint,
                                image.width() as GLsizei,
                                image.height() as GLsizei,
                                0, gl::RGB, gl::UNSIGNED_BYTE,
                                mem::transmute(&image.into_raw()[0])));
                },
                DynamicImage::ImageRgba8(image) => {
                        verify!(gl::TexImage2D(
                                gl::TEXTURE_2D, 0,
                                gl::RGBA as GLint,
                                image.width() as GLsizei,
                                image.height() as GLsizei,
                                0, gl::RGBA, gl::UNSIGNED_BYTE,
                                mem::transmute(&image.into_raw()[0])));
                }
                _ => {
                    panic!("Failed to load texture {}, unsuported pixel format.", path.to_str().unwrap());
                }
            }

            verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint));
            verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint));
            verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint));
            verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint));
        }

        tex.clone()
    }
}
