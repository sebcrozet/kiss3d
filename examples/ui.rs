extern crate kiss3d;
extern crate nalgebra as na;

#[cfg(feature = "egui")]
use kiss3d::light::Light;
#[cfg(feature = "egui")]
use kiss3d::window::Window;
#[cfg(feature = "egui")]
use na::{Point2, UnitQuaternion, Vector3};

#[cfg(not(feature = "egui"))]
#[kiss3d::main]
async fn main() {
    panic!("The 'egui' feature must be enabled for this example to work.")
}

#[cfg(feature = "egui")]
#[kiss3d::main]
async fn main() {
    let mut window = Window::new("Kiss3d: egui UI");
    window.set_background_color(0.9, 0.9, 0.9);

    let mut cube = window.add_cube(0.2, 0.2, 0.2);
    cube.set_color(1.0, 0.0, 0.0);

    window.set_light(Light::StickToCamera);

    // UI state
    let mut rotation_speed = 0.014;
    let mut cube_color = [1.0, 0.0, 0.0];
    let mut ball_position = Point2::new(0.0, 0.0);
    let mut text_input = String::from("Hello egui!");
    let mut slider_value = 50.0;
    let mut checkbox = true;

    // Render loop
    while window.render().await {
        // Rotate cube
        let rot_current = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), rotation_speed);
        cube.prepend_to_local_rotation(&rot_current);

        // Update cube color
        cube.set_color(cube_color[0], cube_color[1], cube_color[2]);

        // Draw UI
        window.draw_ui(|ctx| {
            egui::Window::new("Controls")
                .default_width(300.0)
                .show(ctx, |ui| {
                    ui.heading("Kiss3d egui Example");
                    ui.separator();

                    // Rotation control
                    ui.label("Rotation Speed:");
                    ui.add(egui::Slider::new(&mut rotation_speed, 0.0..=0.1));

                    ui.separator();

                    // Color picker
                    ui.label("Cube Color:");
                    ui.color_edit_button_rgb(&mut cube_color);

                    ui.separator();

                    // XY Pad simulation using sliders
                    ui.label("Ball Position:");
                    ui.horizontal(|ui| {
                        ui.label("X:");
                        ui.add(egui::Slider::new(&mut ball_position.x, -2.0..=2.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Y:");
                        ui.add(egui::Slider::new(&mut ball_position.y, -2.0..=2.0));
                    });

                    ui.separator();

                    // Text input
                    ui.label("Text Input:");
                    ui.text_edit_singleline(&mut text_input);
                    ui.label(format!("You typed: {}", text_input));

                    ui.separator();

                    // Slider
                    ui.label("Slider Value:");
                    ui.add(egui::Slider::new(&mut slider_value, 0.0..=100.0));

                    ui.separator();

                    // Checkbox
                    ui.checkbox(&mut checkbox, "Checkbox");

                    ui.separator();

                    // Button
                    if ui.button("Click me!").clicked() {
                        println!("Button clicked!");
                        // Randomize cube color
                        cube_color = [
                            rand::random(),
                            rand::random(),
                            rand::random(),
                        ];
                    }
                });

            // Second window showing text
            egui::Window::new("Info")
                .default_pos([320.0, 10.0])
                .show(ctx, |ui| {
                    ui.label("egui is a simple, fast, and highly portable immediate mode GUI library.");
                    ui.separator();
                    ui.label(format!("Ball position: ({:.2}, {:.2})", ball_position.x, ball_position.y));
                    ui.label(format!("Slider value: {:.1}", slider_value));
                    ui.label(format!("Checkbox: {}", checkbox));
                });
        });

        // Draw a point at the ball position
        window.draw_point(&na::Point3::new(ball_position.x, ball_position.y, 0.0), &na::Point3::new(0.0, 0.0, 1.0));
    }
}
