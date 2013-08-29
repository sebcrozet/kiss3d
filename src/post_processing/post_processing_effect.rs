use gl::types::*;
use shaders_manager::ShadersManager;

/// Trait of post processing effect. One post-processing effect can be used at a time. It is
/// executed once the scene has been rendered on a texture.
pub trait PostProcessingEffect {
    /// Updates the post processing effect.
    fn update(&mut self, dt: f64);
    /// Render the effect.
    ///
    /// # Arguments:
    ///     * `shaders_manager` - manager to switch between the different shaders.
    ///     * `fbo_texture` - id to the texture containing the last scene drawn.
    fn draw(&self, shaders_manager: &mut ShadersManager, fbo_texture: GLuint);
}
