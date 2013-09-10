use std::local_data;
use extra::rc::Rc;
use std::cast;
use std::hashmap::HashMap;
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
        let id: GLuint = 0;

        unsafe { verify!(gl::GenTextures(1, &id)); }

        Rc::from_send(Texture { id: id })
    }

    /// The opengl-provided texture id.
    pub fn id(&self) -> GLuint {
        self.id
    }
}

impl Drop for Texture {
    fn drop(&self) {
       unsafe { verify!(gl::DeleteTextures(1, &self.id)); }
    }
}

local_data_key!(KEY_TEXTURE_MANAGER: @mut TexturesManager)

/// Inits the texture manager, and put in on TLS.
pub fn init_singleton() {
    if local_data::get(KEY_TEXTURE_MANAGER, |tm| tm.is_none()) {
        local_data::set(KEY_TEXTURE_MANAGER, @mut TexturesManager::new())
    }
}

/// Gets the texture manager.
pub fn singleton() -> @mut TexturesManager {
    local_data::get(KEY_TEXTURE_MANAGER, |tm| *tm.unwrap())
}

/// The textures manager. It keeps a cache of already-loaded textures, and can load new textures.
pub struct TexturesManager {
    priv textures: HashMap<~str, Rc<Texture>>,
}

impl TexturesManager {
    /// Creates a new texture manager.
    pub fn new() -> TexturesManager {
        TexturesManager {
            textures: HashMap::new()
        }
    }

    /// Get a texture with the specified path. Returns `None` if the texture is not loaded.
    pub fn get(&mut self, path: &str) -> Option<Rc<Texture>> {
        self.textures.find(&path.to_owned()).map(|&t| t.clone())
    }

    /// Allocates a new unconfigured texture. If a texture with same name exists, nothing is
    /// created and the old texture is returned.
    pub fn add_empty(&mut self, name: &str) -> Rc<Texture> {
        self.textures.find_or_insert_with(name.to_owned(), |_| Texture::new()).clone()
    }

    /// Allocates a new texture read from a file. If a texture with same name exists, nothing is
    /// created and the old texture is returned.
    pub fn add(&mut self, path: &str) -> Rc<Texture> {
        let tex = self.textures.find_or_insert_with(path.to_owned(), |_| Texture::new());

        unsafe {
            match image::load_with_depth(path.to_owned(), 3, false) {
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
                    fail!("Failed to load texture " + path);
                }
            }
        }

        tex.clone()
    }
}
