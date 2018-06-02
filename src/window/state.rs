use camera::Camera;
use post_processing::PostProcessingEffect;
use window::Window;

pub trait State: 'static {
    fn step(&mut self, window: &mut Window);
    fn camera_and_effect(&mut self) -> (Option<&mut Camera>, Option<&mut PostProcessingEffect>) {
        (None, None)
    }
}

impl State for () {
    fn step(&mut self, _: &mut Window) {}
}
