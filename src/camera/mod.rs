pub use camera::private::camera::Camera;
pub use camera::private::arc_ball::ArcBall;
pub use camera::private::first_person::FirstPerson;

pub mod private {
    #[path = "../camera.rs"]
    pub mod camera;

    #[path = "../arc_ball.rs"]
    pub mod arc_ball;

    #[path = "../first_person.rs"]
    pub mod first_person;
}
