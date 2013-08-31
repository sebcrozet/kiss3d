use std::cast;
use std::hashmap::HashMap;
use gl;
use gl::types::*;
use stb_image::image::ImageU8;
use stb_image::image;

#[path = "../error.rs"]
mod error;

pub struct Texture {
    priv id: GLuint
}

impl Texture {
    pub fn new() -> Texture {
        let id: GLuint = 0;

        unsafe { verify!(gl::GenTextures(1, &id)); }

        Texture {
            id: id
        }
    }

    pub fn id(&self) -> GLuint {
        self.id
    }
}

impl Drop for Texture {
    fn drop(&self) {
       unsafe { verify!(gl::DeleteTextures(1, &self.id)); }
    }
}

pub struct TexturesManager {
    priv textures: HashMap<~str, @Texture>,
}

impl TexturesManager {
    pub fn new() -> TexturesManager {
        TexturesManager {
            textures: HashMap::new()
        }
    }

    pub fn get(&mut self, path: &str) -> Option<@Texture> {
        self.textures.find(&path.to_owned()).map(|t| **t)
    }

    pub fn add_empty(&mut self, name: &str) -> @Texture {
        *self.textures.find_or_insert_with(name.to_owned(), |_| @Texture::new())
    }

    pub fn add(&mut self, path: &str) -> @Texture {
        let tex = self.textures.find_or_insert_with(path.to_owned(), |_| @Texture::new());

        unsafe {
            match image::load_with_depth(path.to_owned(), 3, false) {
                ImageU8(image) => {
                    verify!(gl::ActiveTexture(gl::TEXTURE0));
                    verify!(gl::BindTexture(gl::TEXTURE_2D, tex.id));

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

        *tex
    }
}
