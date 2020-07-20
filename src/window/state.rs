use crate::camera::Camera;
use crate::planar_camera::PlanarCamera;
use crate::post_processing::PostProcessingEffect;
use crate::renderer::Renderer;
use crate::window::{NullUiContext, UiContext, Window};

/// Trait implemented by objects describing state of an application.
///
/// It is passed to the window's render loop. Its methods are called at each
/// render loop to update the application state, and customize the cameras and
/// post-processing effects to be used by the renderer.
pub trait State<Ui: UiContext = NullUiContext>: 'static {
    /// Method called at each render loop before a rendering.
    fn step(&mut self, window: &mut Window<Ui>);

    /// Unless `cameras_and_effect_and_renderer` is implemented, this method called at each render loop to retrieve
    /// the cameras and post-processing effects to be used for the next render.
    #[deprecated(
        note = "This will be replaced by `.cameras_and_effect_and_renderer` which is more flexible."
    )]
    fn cameras_and_effect(
        &mut self,
    ) -> (
        Option<&mut dyn Camera>,
        Option<&mut dyn PlanarCamera>,
        Option<&mut dyn PostProcessingEffect>,
    ) {
        (None, None, None)
    }

    /// Method called at each render loop to retrieve the cameras, custom renderer, and post-processing effect to be used for the next render.
    fn cameras_and_effect_and_renderer(
        &mut self,
    ) -> (
        Option<&mut dyn Camera>,
        Option<&mut dyn PlanarCamera>,
        Option<&mut dyn Renderer>,
        Option<&mut dyn PostProcessingEffect>,
    ) {
        #[allow(deprecated)]
        let res = self.cameras_and_effect(); // For backward-compatibility.
        (res.0, res.1, None, res.2)
    }
}

impl<Ui: UiContext> State<Ui> for () {
    fn step(&mut self, _: &mut Window<Ui>) {}
}
