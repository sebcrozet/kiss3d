use camera::{Camera, Camera2};
use post_processing::PostProcessingEffect;
use window::Window;

pub trait State: 'static {
    fn step(&mut self, window: &mut Window);
    fn cameras_and_effect(
        &mut self,
    ) -> (
        Option<&mut Camera>,
        Option<&mut Camera2>,
        Option<&mut PostProcessingEffect>,
    ) {
        (None, None, None)
    }
}

impl State for () {
    fn step(&mut self, _: &mut Window) {}
}
