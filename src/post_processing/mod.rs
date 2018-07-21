//! Post-processing effects.

pub use post_processing::grayscales::Grayscales;
pub use post_processing::oculus_stereo::OculusStereo;
pub use post_processing::post_processing_effect::PostProcessingEffect;
#[cfg(feature = "opengl3_2")]
pub use post_processing::sobel_edge_highlight::SobelEdgeHighlight;
pub use post_processing::waves::Waves;

mod grayscales;
mod oculus_stereo;
pub mod post_processing_effect;
#[cfg(feature = "opengl3_2")]
mod sobel_edge_highlight;
mod waves;
