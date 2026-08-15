#[cfg(not(feature = "egui"))]
#[kiss3d::main]
async fn main() {
    panic!("The 'egui' feature must be enabled for this example to work.")
}

#[cfg(feature = "egui")]
#[kiss3d::main]
async fn main() {
    use kiss3d::prelude::*;
    use std::collections::VecDeque;

    /// Number of recent events kept in the event log.
    const EVENT_LOG_LEN: usize = 20;

    let mut window = Window::new("Kiss3d: egui UI").await;
    let mut camera = OrbitCamera3d::new(Vec3::new(0.0, 0.5, 1.0), Vec3::ZERO);
    let mut scene = SceneNode3d::empty();
    scene
        .add_light(Light::point(100.0))
        .set_position(Vec3::new(0.0, 10.0, 10.0));

    window.set_background_color(LIGHT_STEEL_BLUE);

    let mut cube = scene.add_cube(0.2, 0.2, 0.2).set_color(RED);

    // UI state
    let mut rotation_speed = 0.014;
    let mut text = String::from("Edit text here!");
    let mut multiline_text = String::from("Multiple lines.\nPress Enter for a new one.");
    let mut opacity = 1.0;
    let mut cube_color = [1.0, 0.0, 0.0];
    // Recent events, kept across frames so a combination like Ctrl+S remains
    // readable after the keys are released.
    let mut event_log: VecDeque<String> = VecDeque::new();

    // Render loop
    while window.render_3d(&mut scene, &mut camera).await {
        // Rotate cube
        let rot_current = Quat::from_axis_angle(Vec3::Y, rotation_speed);
        cube.rotate(rot_current);

        // Update cube color
        cube.set_color(Color::new(
            cube_color[0],
            cube_color[1],
            cube_color[2],
            opacity,
        ));

        // Collect the events, mostly to show the modifiers they carry. Cursor
        // motion is skipped: it would flood the log.
        for event in window.events().iter() {
            if matches!(event.value, WindowEvent::CursorPos(..)) {
                continue;
            }

            event_log.push_back(format!("{:?}", event.value));

            if event_log.len() > EVENT_LOG_LEN {
                event_log.pop_front();
            }
        }

        let mut event_string = event_log
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join("\n");

        // Draw UI
        window.draw_ui(|ctx| {
            egui::Window::new("Kiss3d egui Example")
                .default_width(300.0)
                .show(ctx, |ui| {
                    // Rotation control
                    ui.label("Rotation Speed:");
                    ui.add(egui::Slider::new(&mut rotation_speed, 0.0..=0.1));

                    ui.separator();

                    // Opacity control
                    ui.label("Opacity:");
                    ui.add(egui::Slider::new(&mut opacity, 0.0..=1.0));

                    // Text Input
                    ui.label("Single-line:");
                    ui.add(egui::TextEdit::singleline(&mut text));

                    ui.label("Multi-line:");
                    ui.add(
                        egui::TextEdit::multiline(&mut multiline_text)
                            .desired_rows(3)
                            .desired_width(f32::INFINITY),
                    );

                    // Color picker
                    ui.label("Cube Color:");

                    ui.horizontal(|ui| {
                        ui.color_edit_button_rgb(&mut cube_color);
                        if ui.button("Randomize").clicked() {
                            // Randomize cube color
                            cube_color = [rand::random(), rand::random(), rand::random()];
                        }
                    });

                    // Event log: hold a modifier while typing or clicking to see
                    // it reported (e.g. "Key(S, Press, Modifiers(Control))").
                    ui.separator();
                    ui.label("Recent events:");
                    egui::ScrollArea::vertical()
                        .max_height(150.0)
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut event_string)
                                    .interactive(false)
                                    .desired_width(f32::INFINITY),
                            );
                        });
                });
        });
    }
}
