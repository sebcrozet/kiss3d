use camera::Camera;
use planar_camera::PlanarCamera;
use post_processing::PostProcessingEffect;
use window::Window;
use renderer::Renderer;

/// Trait implemented by objects describing state of an application.
///
/// It is passed to the window's render loop. Its methods are called at each
/// render loop to update the application state, and customize the cameras and
/// post-processing effects to be used by the renderer.
pub trait State: 'static {
    /// Method called at each render loop before a rendering.
    fn step(&mut self, window: &mut Window);

    /// Unless `cameras_and_effect_and_renderer` is implemented, this method called at each render loop to retrieve
    /// the cameras and post-processing effects to be used for the next render.
    #[deprecated(note = "This will be replaced by `.cameras_and_effect_and_renderer` which is more flexible.")]
    fn cameras_and_effect(
        &mut self,
    ) -> (
        Option<&mut Camera>,
        Option<&mut PlanarCamera>,
        Option<&mut PostProcessingEffect>,
    ) {
        (None, None, None)
    }

    /// Method called at each render loop to retrieve the cameras, custom renderer, and post-processing effect to be used for the next render.
    fn cameras_and_effect_and_renderer(&mut self) -> (
        Option<&mut Camera>,
        Option<&mut PlanarCamera>,
        Option<&mut Renderer>,
        Option<&mut PostProcessingEffect>,
    ) {
        #[allow(deprecated)]
        let res = self.cameras_and_effect(); // For backward-compatibility.
        (res.0, res.1, None, res.2)
    }
}

impl State for () {
    fn step(&mut self, _: &mut Window) {}
}
