//! Trait implemented by every post-processing effect.

use resource::RenderTarget;

/// Trait of post processing effect.
///
/// One post-processing effect can be used at a time. It is executed once the scene has been
/// rendered on a texture.
pub trait PostProcessingEffect {
    /// Updates the post processing effect.
    fn update(&mut self, dt: f32, w: f32, h: f32, znear: f32, zfar: f32);
    /// Render the effect.
    ///
    /// # Arguments:
    /// * `shader_manager` - manager to switch between the different shaders.
    /// * `fbo_texture` - id to the texture containing the last scene drawn.
    /// * `fbo_depth` - the depth buffer as a texture.
    fn draw(&mut self, target: &RenderTarget);
}
