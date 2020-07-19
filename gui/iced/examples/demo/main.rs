use controls::Controls;
use iced_graphics::{Point, Size};
use iced_native::{program, Debug};
use kiss3d::light::Light;
use kiss3d::window::Window;
use kiss3d_iced::{Backend, IcedContext, Renderer, Settings, Viewport};

mod controls;

fn main() {
    let controls = Controls::new();
    let mut window: Window<IcedContext<_>> = Window::new_with_ui("Kiss3d: UI", controls);
    window.set_background_color(1.0, 1.0, 1.0);
    let mut c = window.add_cube(0.1, 0.1, 0.1);
    c.set_color(1.0, 0.0, 0.0);

    window.set_light(Light::StickToCamera);

    // Render loop.
    while window.render() {
        let color = window.ui().program().background_color();
        window.set_background_color(color.r, color.g, color.b);
    }
}
