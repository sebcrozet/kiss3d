//! Camera trait with some common implementations.

pub use camera::camera::Camera;
pub use camera::arc_ball::ArcBall;
pub use camera::first_person::FirstPerson;
pub use camera::first_person_stereo::FirstPersonStereo;

#[doc(hidden)]
pub mod camera;
mod arc_ball;
mod first_person;
mod first_person_stereo;
