//! kiss3d on iOS: a spinning cube and an egui tap counter.
//!
//! The cube proves the surface and the inverted render loop; the counter
//! proves touch input reaches egui. `#[kiss3d::main]` expands to
//! `window::run_ios`, which hands this future to winit's UIApplicationMain-
//! owned loop.

use kiss3d::egui;
use kiss3d::prelude::*;

#[kiss3d::main]
async fn main() {
    // The simulator forwards stderr to `simctl launch --console`, so plain
    // env_logger is enough to see wgpu/winit output during bring-up.
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    log::info!("kiss3d ios example: starting");

    let mut window = Window::new("kiss3d ios").await;
    let mut camera = OrbitCamera3d::default();
    let mut scene = SceneNode3d::empty();
    scene
        .add_light(Light::point(100.0))
        .set_position(Vec3::new(0.0, 2.0, -2.0));

    let mut c = scene.add_cube(1.0, 1.0, 1.0).set_color(RED);
    let rot = Quat::from_axis_angle(Vec3::Y, 0.014);
    let mut taps: u32 = 0;
    let mut text = String::new();
    let mut keyboard_shown = false;

    log::info!("kiss3d ios example: window open, entering render loop");
    while window.render_3d(&mut scene, &mut camera).await {
        c.rotate(rot);
        window.draw_ui(|ctx| {
            egui::Window::new("touch check").show(ctx, |ui| {
                if ui.button(format!("taps: {taps}")).clicked() {
                    taps += 1;
                    log::info!("tap registered: {taps}");
                }
                // Focusing this summons the system keyboard (see below); what
                // it types lands here and in the console.
                if ui.text_edit_singleline(&mut text).changed() {
                    log::info!("typed: {text}");
                }
            });
        });
        let wants = window.is_egui_capturing_keyboard();
        if wants != keyboard_shown {
            keyboard_shown = wants;
            log::info!("keyboard visible: {wants}");
            window.set_keyboard_visible(wants);
        }
    }
}
