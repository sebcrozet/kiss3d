//! kiss3d on Android: a spinning cube and an egui tap counter.
//!
//! The cube proves the surface and render loop; the counter proves touch
//! input reaches egui. Everything else (lifecycle, event loop, entry point)
//! is exercised implicitly by getting this far.

use kiss3d::egui;
use kiss3d::prelude::*;

#[kiss3d::main]
async fn main() {
    // Logcat is the only place output goes on device, and a silent panic is
    // undebuggable, so both logging and panics are routed there.
    #[cfg(target_os = "android")]
    {
        android_logger::init_once(
            android_logger::Config::default().with_max_level(log::LevelFilter::Info),
        );
        std::panic::set_hook(Box::new(|info| log::error!("panic: {info}")));
    }
    log::info!("kiss3d android example: starting");

    // No MSAA: the emulator's gfxstream GLES rejects a multisampled float
    // framebuffer as incomplete, which silently blacks out the 3D pass.
    // Real devices take the Vulkan backend and are unaffected.
    let setup = CanvasSetup {
        samples: NumSamples::One,
        ..CanvasSetup::default()
    };
    let mut window = Window::new_with_setup("kiss3d android", 1080, 2400, setup).await;
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

    log::info!("kiss3d android example: window open, entering render loop");
    while window.render_3d(&mut scene, &mut camera).await {
        c.rotate(rot);
        window.draw_ui(|ctx| {
            egui::Window::new("touch check").show(ctx, |ui| {
                if ui.button(format!("taps: {taps}")).clicked() {
                    taps += 1;
                    log::info!("tap registered: {taps}");
                }
                // Focusing this summons the system keyboard (see below); what
                // it types lands here and in logcat.
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
