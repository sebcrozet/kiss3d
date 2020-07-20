use controls::Controls;
use kiss3d::light::Light;
use kiss3d::{camera::FirstPerson, event::Key, window::Window};
use kiss3d_iced::{IcedContext, Settings};
use nalgebra::Point3;

mod controls;

fn main() {
    let controls = Controls::new();
    let mut window: Window<IcedContext<_>> = Window::new_with_ui("Kiss3d: UI", controls);
    window.set_background_color(1.0, 1.0, 1.0);
    let mut c = window.add_cube(1.0, 1.0, 1.0);
    c.set_color(1.0, 0.0, 0.0);

    window.set_light(Light::StickToCamera);

    let eye = Point3::new(3.0f32, 5.0, 5.0);
    let at = Point3::origin();
    let mut first_person = FirstPerson::new(eye, at);
    first_person.rebind_up_key(Some(Key::W));
    first_person.rebind_down_key(Some(Key::S));
    first_person.rebind_left_key(Some(Key::A));
    first_person.rebind_right_key(Some(Key::D));

    // Render loop.
    while window.render_with_camera(&mut first_person) {
        let color = window.ui().program().background_color();
        window.set_background_color(color.r, color.g, color.b);
    }
}
