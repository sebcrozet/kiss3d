//! Cameras for 2D rendering.

pub use self::fixed_view::FixedView;
pub use self::planar_camera::PlanarCamera;
pub use self::sidescroll::Sidescroll;

mod fixed_view;
mod planar_camera;
mod sidescroll;
