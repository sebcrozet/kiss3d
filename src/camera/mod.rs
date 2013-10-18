//! Camera trait with some commonimplementations.

pub use camera::camera::Camera;
pub use camera::arc_ball::ArcBall;
pub use camera::first_person::FirstPerson;
pub use camera::first_person_stereo::FirstPersonStereo;

#[doc(hidden)]
pub mod camera;
#[doc(hidden)]
pub mod arc_ball;
#[doc(hidden)]
pub mod first_person;
#[doc(hidden)]
pub mod first_person_stereo;
