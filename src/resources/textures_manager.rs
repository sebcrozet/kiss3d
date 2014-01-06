//! A resource manager to load textures.

use std::local_data;
use std::cast;
use std::hashmap::HashMap;
use std::rc::Rc;
use gl;
use gl::types::*;
use stb_image::image::ImageU8;
use stb_image::image;

#[path = "../error.rs"]
mod error;

/// A gpu texture. It contains the texture id provided by opengl and is automatically released.
pub struct Texture {
    priv id: GLuint
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

// FIXME: why is this on TLS?
local_data_key!(KEY_TEXTURE_MANAGER: TexturesManager)

/// Gets the texture manager.
pub fn get<T>(f: |&mut TexturesManager| -> T) -> T {
    if local_data::get(KEY_TEXTURE_MANAGER, |tm| tm.is_none()) {
        local_data::set(KEY_TEXTURE_MANAGER, TexturesManager::new())
    }

    local_data::get_mut(KEY_TEXTURE_MANAGER, |tm| f(tm.unwrap()))
}

/// The textures manager. It keeps a cache of already-loaded textures, and can load new textures.
pub struct TexturesManager {
    priv default_texture: Rc<Texture>,
    priv textures:        HashMap<~str, Rc<Texture>>,
}

impl TexturesManager {
    /// Creates a new texture manager.
    pub fn new() -> TexturesManager {
        let default_tex = Texture::new();
        let default_tex_pixels: [ GLfloat, ..3 ] = [ 1.0, 1.0, 1.0 ];
        verify!(gl::ActiveTexture(gl::TEXTURE0));
        verify!(gl::BindTexture(gl::TEXTURE_2D, default_tex.borrow().id()));
        verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_BASE_LEVEL, 0));
        verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, 0));
        verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32));
        verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32));
        verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32));
        verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32));

        unsafe {
            verify!(gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as i32, 1, 1, 0, gl::RGB, gl::FLOAT,
                                   cast::transmute(&default_tex_pixels[0])));
        }

        TexturesManager {
            textures:        HashMap::new(),
            default_texture: default_tex
        }
    }

    /// Gets the default, completely white, texture.
    pub fn get_default(&self) -> Rc<Texture> {
        self.default_texture.clone()
    }

    /// Get a texture with the specified path. Returns `None` if the texture is not loaded.
    pub fn get(&mut self, name: &str) -> Option<Rc<Texture>> {
        self.textures.find(&name.to_owned()).map(|t| t.clone())
    }

    /// Allocates a new unconfigured texture. If a texture with same name exists, nothing is
    /// created and the old texture is returned.
    pub fn add_empty(&mut self, name: &str) -> Rc<Texture> {
        self.textures.find_or_insert_with(name.to_owned(), |_| Texture::new()).clone()
    }

    /// Allocates a new texture read from a file. If a texture with same name exists, nothing is
    /// created and the old texture is returned.
    pub fn add(&mut self, path: &Path, name: &str) -> Rc<Texture> {
        let tex = self.textures.find_or_insert_with(name.to_owned(), |_| Texture::new());

        // FIXME: dont re-load the texture if it already exists!
        unsafe {
            match image::load_with_depth(path.as_str().unwrap().to_owned(), 3, false) {
                ImageU8(image) => {
                    verify!(gl::ActiveTexture(gl::TEXTURE0));
                    verify!(gl::BindTexture(gl::TEXTURE_2D, tex.borrow().id()));

                    verify!(gl::TexImage2D(
                            gl::TEXTURE_2D, 0,
                            gl::RGB as GLint,
                            image.width as GLsizei,
                            image.height as GLsizei,
                            0, gl::RGB, gl::UNSIGNED_BYTE,
                            cast::transmute(&image.data[0])));

                    verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint));
                    verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint));
                    verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint));
                    verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint));
                }
                _ => {
                    fail!("Failed to load texture " + path.as_str().unwrap());
                }
            }
        }

        tex.clone()
    }
}
