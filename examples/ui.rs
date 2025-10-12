extern crate kiss3d;
extern crate nalgebra as na;

#[cfg(feature = "egui")]
use kiss3d::light::Light;
#[cfg(feature = "egui")]
use kiss3d::window::Window;
#[cfg(feature = "egui")]
use na::{Point3, Translation3, UnitQuaternion};

#[cfg(not(feature = "egui"))]
#[kiss3d::main]
async fn main() {
    panic!("The 'egui' feature must be enabled for this example to work.")
}

#[cfg(feature = "egui")]
struct ObjectState {
    position: [f32; 3],
    rotation: [f32; 3], // Euler angles in degrees
    scale: f32,
    color: [f32; 3],
    auto_rotate: bool,
    rotation_speed: f32,
    visible: bool,
}

#[cfg(feature = "egui")]
impl Default for ObjectState {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: 1.0,
            color: [1.0, 1.0, 1.0],
            auto_rotate: false,
            rotation_speed: 1.0,
            visible: true,
        }
    }
}

#[cfg(feature = "egui")]
#[kiss3d::main]
async fn main() {
    let mut window = Window::new("Kiss3d: Interactive 3D Scene with egui");
    window.set_background_color(0.1, 0.1, 0.15);
    window.set_light(Light::StickToCamera);

    // Create multiple objects
    let mut cube = window.add_cube(1.0, 1.0, 1.0);
    let mut sphere = window.add_sphere(0.5);
    let mut cylinder = window.add_cylinder(0.4, 1.5);
    let mut cone = window.add_cone(0.5, 1.0);

    // Position objects initially
    cube.set_local_translation(Translation3::new(-2.0, 0.0, 0.0));
    sphere.set_local_translation(Translation3::new(2.0, 0.0, 0.0));
    cylinder.set_local_translation(Translation3::new(-2.0, 0.0, 2.0));
    cone.set_local_translation(Translation3::new(2.0, 0.0, 2.0));

    // Object states
    let mut cube_state = ObjectState {
        position: [-2.0, 0.0, 0.0],
        color: [1.0, 0.2, 0.2],
        auto_rotate: true,
        rotation_speed: 1.0,
        ..Default::default()
    };
    let mut sphere_state = ObjectState {
        position: [2.0, 0.0, 0.0],
        color: [0.2, 1.0, 0.2],
        auto_rotate: true,
        rotation_speed: 0.5,
        ..Default::default()
    };
    let mut cylinder_state = ObjectState {
        position: [-2.0, 0.0, 2.0],
        color: [0.2, 0.2, 1.0],
        ..Default::default()
    };
    let mut cone_state = ObjectState {
        position: [2.0, 0.0, 2.0],
        color: [1.0, 1.0, 0.2],
        auto_rotate: true,
        rotation_speed: 2.0,
        ..Default::default()
    };

    // Scene settings
    let mut background_color = [0.1, 0.1, 0.15];
    let mut point_size = 5.0;
    let mut line_width = 1.0;
    let mut show_axes = true;
    let mut show_grid = true;
    let mut selected_object = 0;

    // Particle system for fun
    let mut particles: Vec<Point3<f32>> = Vec::new();
    let mut particle_colors: Vec<Point3<f32>> = Vec::new();
    let mut emit_particles = false;
    let mut particle_count = 0;

    // Render loop
    while window.render().await {
        // Update background
        window.set_background_color(background_color[0], background_color[1], background_color[2]);
        window.set_point_size(point_size);
        window.set_line_width(line_width);

        // Update cube
        if cube_state.visible {
            cube.set_visible(true);
            cube.set_local_translation(Translation3::new(
                cube_state.position[0],
                cube_state.position[1],
                cube_state.position[2],
            ));
            cube.set_local_scale(cube_state.scale, cube_state.scale, cube_state.scale);
            cube.set_color(cube_state.color[0], cube_state.color[1], cube_state.color[2]);

            if cube_state.auto_rotate {
                let rot = UnitQuaternion::from_euler_angles(
                    0.0,
                    0.01 * cube_state.rotation_speed,
                    0.0,
                );
                cube.prepend_to_local_rotation(&rot);
                cube_state.rotation[1] += 0.01 * cube_state.rotation_speed * 57.3; // Convert to degrees
                cube_state.rotation[1] %= 360.0;
            } else {
                let rot = UnitQuaternion::from_euler_angles(
                    cube_state.rotation[0].to_radians(),
                    cube_state.rotation[1].to_radians(),
                    cube_state.rotation[2].to_radians(),
                );
                cube.set_local_rotation(rot);
            }
        } else {
            cube.set_visible(false);
        }

        // Update sphere
        if sphere_state.visible {
            sphere.set_visible(true);
            sphere.set_local_translation(Translation3::new(
                sphere_state.position[0],
                sphere_state.position[1],
                sphere_state.position[2],
            ));
            sphere.set_local_scale(sphere_state.scale, sphere_state.scale, sphere_state.scale);
            sphere.set_color(sphere_state.color[0], sphere_state.color[1], sphere_state.color[2]);

            if sphere_state.auto_rotate {
                let rot = UnitQuaternion::from_euler_angles(
                    0.01 * sphere_state.rotation_speed,
                    0.01 * sphere_state.rotation_speed,
                    0.0,
                );
                sphere.prepend_to_local_rotation(&rot);
            }
        } else {
            sphere.set_visible(false);
        }

        // Update cylinder
        if cylinder_state.visible {
            cylinder.set_visible(true);
            cylinder.set_local_translation(Translation3::new(
                cylinder_state.position[0],
                cylinder_state.position[1],
                cylinder_state.position[2],
            ));
            cylinder.set_local_scale(cylinder_state.scale, cylinder_state.scale, cylinder_state.scale);
            cylinder.set_color(cylinder_state.color[0], cylinder_state.color[1], cylinder_state.color[2]);

            let rot = UnitQuaternion::from_euler_angles(
                cylinder_state.rotation[0].to_radians(),
                cylinder_state.rotation[1].to_radians(),
                cylinder_state.rotation[2].to_radians(),
            );
            cylinder.set_local_rotation(rot);
        } else {
            cylinder.set_visible(false);
        }

        // Update cone
        if cone_state.visible {
            cone.set_visible(true);
            cone.set_local_translation(Translation3::new(
                cone_state.position[0],
                cone_state.position[1],
                cone_state.position[2],
            ));
            cone.set_local_scale(cone_state.scale, cone_state.scale, cone_state.scale);
            cone.set_color(cone_state.color[0], cone_state.color[1], cone_state.color[2]);

            if cone_state.auto_rotate {
                let rot = UnitQuaternion::from_euler_angles(
                    0.0,
                    0.0,
                    0.01 * cone_state.rotation_speed,
                );
                cone.prepend_to_local_rotation(&rot);
            }
        } else {
            cone.set_visible(false);
        }

        // Draw coordinate axes if enabled
        if show_axes {
            let axis_length = 5.0;
            window.draw_line(
                &Point3::origin(),
                &Point3::new(axis_length, 0.0, 0.0),
                &Point3::new(1.0, 0.0, 0.0),
            );
            window.draw_line(
                &Point3::origin(),
                &Point3::new(0.0, axis_length, 0.0),
                &Point3::new(0.0, 1.0, 0.0),
            );
            window.draw_line(
                &Point3::origin(),
                &Point3::new(0.0, 0.0, axis_length),
                &Point3::new(0.0, 0.0, 1.0),
            );
        }

        // Draw grid if enabled
        if show_grid {
            let grid_size = 10.0;
            let grid_step = 1.0;
            for i in -10..=10 {
                let i = i as f32 * grid_step;
                window.draw_line(
                    &Point3::new(-grid_size, 0.0, i),
                    &Point3::new(grid_size, 0.0, i),
                    &Point3::new(0.3, 0.3, 0.3),
                );
                window.draw_line(
                    &Point3::new(i, 0.0, -grid_size),
                    &Point3::new(i, 0.0, grid_size),
                    &Point3::new(0.3, 0.3, 0.3),
                );
            }
        }

        // Particle system
        if emit_particles {
            let angle = particle_count as f32 * 0.1;
            let radius = 3.0;
            particles.push(Point3::new(
                angle.cos() * radius,
                (particle_count as f32 * 0.05).sin() * 2.0,
                angle.sin() * radius,
            ));
            particle_colors.push(Point3::new(
                (particle_count as f32 * 0.1).sin() * 0.5 + 0.5,
                (particle_count as f32 * 0.2).cos() * 0.5 + 0.5,
                (particle_count as f32 * 0.15).sin() * 0.5 + 0.5,
            ));
            particle_count += 1;

            // Limit particle count
            if particles.len() > 500 {
                particles.remove(0);
                particle_colors.remove(0);
            }
        }

        // Draw particles
        for (particle, color) in particles.iter().zip(particle_colors.iter()) {
            window.draw_point(particle, color);
        }

        // Draw UI
        window.draw_ui(|ctx| {
            // Main control panel
            egui::Window::new("🎮 Scene Controls")
                .default_width(320.0)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.heading("Interactive 3D Scene");
                    ui.separator();

                    // Object selector
                    ui.label("Select Object:");
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut selected_object, 0, "🟥 Cube");
                        ui.selectable_value(&mut selected_object, 1, "🟢 Sphere");
                        ui.selectable_value(&mut selected_object, 2, "🔵 Cylinder");
                        ui.selectable_value(&mut selected_object, 3, "🟡 Cone");
                    });

                    ui.separator();

                    // Get reference to selected object state
                    let state = match selected_object {
                        0 => &mut cube_state,
                        1 => &mut sphere_state,
                        2 => &mut cylinder_state,
                        _ => &mut cone_state,
                    };

                    ui.checkbox(&mut state.visible, "Visible");

                    if state.visible {
                        ui.label("Position:");
                        ui.horizontal(|ui| {
                            ui.label("X:");
                            ui.add(egui::DragValue::new(&mut state.position[0]).speed(0.1));
                            ui.label("Y:");
                            ui.add(egui::DragValue::new(&mut state.position[1]).speed(0.1));
                            ui.label("Z:");
                            ui.add(egui::DragValue::new(&mut state.position[2]).speed(0.1));
                        });

                        ui.label("Rotation (degrees):");
                        ui.horizontal(|ui| {
                            ui.label("X:");
                            ui.add(egui::DragValue::new(&mut state.rotation[0]).speed(1.0));
                            ui.label("Y:");
                            ui.add(egui::DragValue::new(&mut state.rotation[1]).speed(1.0));
                            ui.label("Z:");
                            ui.add(egui::DragValue::new(&mut state.rotation[2]).speed(1.0));
                        });

                        ui.add(egui::Slider::new(&mut state.scale, 0.1..=3.0).text("Scale"));

                        ui.color_edit_button_rgb(&mut state.color);

                        ui.separator();

                        ui.checkbox(&mut state.auto_rotate, "Auto-rotate");
                        if state.auto_rotate {
                            ui.add(egui::Slider::new(&mut state.rotation_speed, 0.0..=5.0).text("Rotation Speed"));
                        }

                        ui.separator();

                        if ui.button("🎲 Randomize Color").clicked() {
                            state.color = [rand::random(), rand::random(), rand::random()];
                        }

                        if ui.button("🔄 Reset Transform").clicked() {
                            state.rotation = [0.0, 0.0, 0.0];
                            state.scale = 1.0;
                        }
                    }
                });

            // Scene settings panel
            egui::Window::new("🌍 Scene Settings")
                .default_pos([340.0, 10.0])
                .default_width(280.0)
                .show(ctx, |ui| {
                    ui.heading("Environment");
                    ui.separator();

                    ui.label("Background Color:");
                    ui.color_edit_button_rgb(&mut background_color);

                    ui.separator();

                    ui.label("Rendering:");
                    ui.add(egui::Slider::new(&mut point_size, 1.0..=20.0).text("Point Size"));
                    ui.add(egui::Slider::new(&mut line_width, 0.5..=10.0).text("Line Width"));

                    ui.separator();

                    ui.label("Helpers:");
                    ui.checkbox(&mut show_axes, "Show Axes (XYZ)");
                    ui.checkbox(&mut show_grid, "Show Grid");

                    ui.separator();

                    ui.heading("Particle System");
                    ui.checkbox(&mut emit_particles, "Emit Particles");
                    ui.label(format!("Particle count: {}", particles.len()));

                    if ui.button("💥 Clear Particles").clicked() {
                        particles.clear();
                        particle_colors.clear();
                        particle_count = 0;
                    }

                    ui.separator();

                    if ui.button("🎨 Randomize All Colors").clicked() {
                        cube_state.color = [rand::random(), rand::random(), rand::random()];
                        sphere_state.color = [rand::random(), rand::random(), rand::random()];
                        cylinder_state.color = [rand::random(), rand::random(), rand::random()];
                        cone_state.color = [rand::random(), rand::random(), rand::random()];
                    }

                    if ui.button("🌀 Auto-rotate All").clicked() {
                        cube_state.auto_rotate = true;
                        sphere_state.auto_rotate = true;
                        cone_state.auto_rotate = true;
                        cylinder_state.auto_rotate = false; // Keep one static
                    }

                    if ui.button("⏸ Stop All Rotation").clicked() {
                        cube_state.auto_rotate = false;
                        sphere_state.auto_rotate = false;
                        cone_state.auto_rotate = false;
                        cylinder_state.auto_rotate = false;
                    }
                });

            // Info panel
            egui::Window::new("ℹ️ Info")
                .default_pos([10.0, 350.0])
                .default_width(300.0)
                .show(ctx, |ui| {
                    ui.label("🎮 Controls:");
                    ui.monospace("• Left drag: Rotate camera");
                    ui.monospace("• Right drag: Pan camera");
                    ui.monospace("• Scroll: Zoom");
                    ui.monospace("• Enter: Reset camera");
                    ui.separator();

                    ui.label("💡 Tips:");
                    ui.label("• Click on object tabs to edit them");
                    ui.label("• Enable auto-rotate for animation");
                    ui.label("• Try the particle system!");
                    ui.label("• Randomize colors for fun effects");
                });

            // Quick actions panel
            egui::Window::new("⚡ Quick Actions")
                .default_pos([340.0, 400.0])
                .default_width(280.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("🎯 Center All").clicked() {
                            cube_state.position = [-2.0, 0.0, 0.0];
                            sphere_state.position = [2.0, 0.0, 0.0];
                            cylinder_state.position = [-2.0, 0.0, 2.0];
                            cone_state.position = [2.0, 0.0, 2.0];
                        }
                        if ui.button("🎲 Scatter").clicked() {
                            use rand::Rng;
                            let mut rng = rand::thread_rng();
                            cube_state.position = [rng.gen_range(-3.0..3.0), rng.gen_range(-2.0..2.0), rng.gen_range(-3.0..3.0)];
                            sphere_state.position = [rng.gen_range(-3.0..3.0), rng.gen_range(-2.0..2.0), rng.gen_range(-3.0..3.0)];
                            cylinder_state.position = [rng.gen_range(-3.0..3.0), rng.gen_range(-2.0..2.0), rng.gen_range(-3.0..3.0)];
                            cone_state.position = [rng.gen_range(-3.0..3.0), rng.gen_range(-2.0..2.0), rng.gen_range(-3.0..3.0)];
                        }
                    });

                    ui.horizontal(|ui| {
                        if ui.button("📏 Same Size").clicked() {
                            let size = 1.5;
                            cube_state.scale = size;
                            sphere_state.scale = size;
                            cylinder_state.scale = size;
                            cone_state.scale = size;
                        }
                        if ui.button("🎭 Show All").clicked() {
                            cube_state.visible = true;
                            sphere_state.visible = true;
                            cylinder_state.visible = true;
                            cone_state.visible = true;
                        }
                    });

                    ui.separator();

                    ui.label(format!("FPS: ~{:.0}", 1.0 / 0.016)); // Rough estimate
                    ui.label(format!("Objects: {}",
                        (cube_state.visible as i32) +
                        (sphere_state.visible as i32) +
                        (cylinder_state.visible as i32) +
                        (cone_state.visible as i32)
                    ));
                });
        });
    }
}
