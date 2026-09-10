//! Egui integration for immediate mode UI.

use egui::RawInput;

use crate::event::{Action, Key, Modifiers, WindowEvent};
use crate::renderer::EguiRenderer;

use super::Window;

pub(crate) struct EguiContext {
    pub(crate) renderer: EguiRenderer,
    pub(crate) raw_input: RawInput,
    /// Whether an egui pass is currently open. A pass is opened lazily by the
    /// first `draw_ui` of the frame and closed by `finish_egui_pass` (called at
    /// render time). This lets several `draw_ui` / `draw_inspector` calls share a
    /// single pass instead of each starting its own (which would overwrite the
    /// previous one's shapes).
    pub(crate) pass_active: bool,
    /// The touch currently driving the egui pointer. Touch screens have no
    /// cursor, so the first finger down plays that role until it lifts;
    /// other fingers are ignored rather than fighting over the pointer.
    pub(crate) pointer_touch_id: Option<u64>,
    /// What a rerun of the open pass begins with: the pass's input less its
    /// events, as `Context::run` hands the later passes of a frame.
    pub(crate) rerun_input: RawInput,
    /// The shape the pointer was last set to. Setting one is a hop to the
    /// main thread, so only a change is worth making.
    pub(crate) cursor: egui::CursorIcon,
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) start_time: std::time::Instant,
}

impl EguiContext {
    pub(crate) fn new() -> Self {
        Self {
            renderer: EguiRenderer::new(),
            raw_input: RawInput::default(),
            pass_active: false,
            pointer_touch_id: None,
            rerun_input: RawInput::default(),
            cursor: egui::CursorIcon::Default,
            #[cfg(not(target_arch = "wasm32"))]
            start_time: std::time::Instant::now(),
        }
    }
}

/// Whether ⌘ (Super) is the platform's command modifier: on macOS, and on
/// any Apple platform reached through a browser. Everywhere else it is Ctrl.
fn command_is_super() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        super::wgpu_canvas::apple_platform()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        cfg!(target_os = "macos")
    }
}

/// egui's view of the modifiers an event arrived with. Read from the event
/// rather than from the key states: those are polled once a frame, so a chord
/// tapped within one frame has already released its modifier by then.
fn egui_modifiers(modifiers: Modifiers) -> egui::Modifiers {
    let ctrl = modifiers.contains(Modifiers::Control);
    let mac_cmd = command_is_super() && modifiers.contains(Modifiers::Super);
    egui::Modifiers {
        alt: modifiers.contains(Modifiers::Alt),
        ctrl,
        shift: modifiers.contains(Modifiers::Shift),
        mac_cmd,
        command: if command_is_super() { mac_cmd } else { ctrl },
    }
}

impl Window {
    /// Retrieves a mutable reference to the egui context.
    ///
    /// Use this to access egui's full API for creating custom UI elements.
    ///
    /// # Returns
    /// A mutable reference to the egui Context
    ///
    /// # Note
    /// Only available when the `egui` feature is enabled.
    pub fn egui_context_mut(&mut self) -> &mut egui::Context {
        self.egui_context.renderer.context_mut()
    }

    /// Retrieves a reference to the egui context.
    ///
    /// Use this to access egui's API for reading UI state.
    ///
    /// # Returns
    /// A reference to the egui Context
    ///
    /// # Note
    /// Only available when the `egui` feature is enabled.
    pub fn egui_context(&self) -> &egui::Context {
        self.egui_context.renderer.context()
    }

    /// Registers a native wgpu texture view with this window's egui renderer,
    /// returning a [`egui::TextureId`] that can be drawn with
    /// `ui.image((id, size))` inside [`Window::draw_ui`] — entirely on the
    /// GPU, no read-back, so it works on the web too.
    ///
    /// Typical use: display the live output of an
    /// [`OffscreenSurface`](crate::window::OffscreenSurface) (its
    /// [`output_view`](crate::window::OffscreenSurface::output_view)) as a
    /// picture-in-picture panel.
    ///
    /// The id stays valid until [`Window::unregister_egui_texture`]. If the
    /// underlying texture is reallocated (e.g. the surface is resized),
    /// re-register the new view.
    pub fn register_egui_texture(
        &mut self,
        view: &wgpu::TextureView,
        filter: wgpu::FilterMode,
    ) -> egui::TextureId {
        self.egui_context
            .renderer
            .register_native_texture(view, filter)
    }

    /// Frees a texture id previously returned by
    /// [`Window::register_egui_texture`].
    pub fn unregister_egui_texture(&mut self, id: egui::TextureId) {
        self.egui_context.renderer.unregister_native_texture(id)
    }

    /// Checks if egui is currently capturing mouse input.
    ///
    /// Returns `true` if the mouse is hovering over or interacting with an egui widget.
    /// This is useful for preventing 3D camera controls from interfering with UI interaction.
    ///
    /// # Returns
    /// `true` if egui wants mouse input, `false` otherwise
    ///
    /// # Note
    /// Only available when the `egui` feature is enabled.
    pub fn is_egui_capturing_mouse(&self) -> bool {
        self.egui_context.renderer.wants_pointer_input()
    }

    /// Checks if egui is currently capturing keyboard input.
    ///
    /// Returns `true` if an egui text field or other widget has keyboard focus.
    /// This is useful for preventing keyboard shortcuts from triggering while typing in UI.
    ///
    /// # Returns
    /// `true` if egui wants keyboard input, `false` otherwise
    ///
    /// # Note
    /// Only available when the `egui` feature is enabled.
    pub fn is_egui_capturing_keyboard(&self) -> bool {
        self.egui_context.renderer.wants_keyboard_input()
    }

    /// Feed a window event to egui for processing.
    pub(crate) fn feed_egui_event(&mut self, event: &WindowEvent) {
        let scale_factor = self.scale_factor() as f32;

        match *event {
            WindowEvent::CursorPos(x, y, _) => {
                // Convert physical pixels to logical coordinates
                let pos = egui::Pos2::new((x as f32) / scale_factor, (y as f32) / scale_factor);
                self.egui_context
                    .raw_input
                    .events
                    .push(egui::Event::PointerMoved(pos));
            }
            WindowEvent::MouseButton(button, action, modifiers) => {
                let button = match button {
                    crate::event::MouseButton::Button1 => egui::PointerButton::Primary,
                    crate::event::MouseButton::Button2 => egui::PointerButton::Secondary,
                    crate::event::MouseButton::Button3 => egui::PointerButton::Middle,
                    _ => return,
                };

                if let Some(pos) = self.cursor_pos() {
                    // Convert physical pixels to logical coordinates
                    let pos = egui::Pos2::new(
                        (pos.0 as f32) / scale_factor,
                        (pos.1 as f32) / scale_factor,
                    );
                    let pressed = action == Action::Press;

                    self.egui_context
                        .raw_input
                        .events
                        .push(egui::Event::PointerButton {
                            pos,
                            button,
                            pressed,
                            modifiers: egui_modifiers(modifiers),
                        });
                }
            }
            WindowEvent::Scroll(x, y, modifiers) => {
                // Use Point unit since kiss3d's scroll values are already scaled
                // (native multiplies LineDelta by 10, WASM applies various scales)
                self.egui_context
                    .raw_input
                    .events
                    .push(egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::Vec2::new(x as f32, y as f32),
                        phase: egui::TouchPhase::Move,
                        modifiers: egui_modifiers(modifiers),
                    });
            }
            WindowEvent::Touch(id, x, y, action, _) => {
                use crate::event::TouchAction;

                let pos = egui::Pos2::new((x as f32) / scale_factor, (y as f32) / scale_factor);
                let events = &mut self.egui_context.raw_input.events;

                // Every touch is reported as a raw touch first: egui derives
                // pinch-zoom and rotate from the full set (MultiTouchInfo), the
                // same way egui-winit feeds it.
                events.push(egui::Event::Touch {
                    device_id: egui::TouchDeviceId(0),
                    id: egui::TouchId(id),
                    phase: match action {
                        TouchAction::Start => egui::TouchPhase::Start,
                        TouchAction::Move => egui::TouchPhase::Move,
                        TouchAction::End => egui::TouchPhase::End,
                        TouchAction::Cancel => egui::TouchPhase::Cancel,
                    },
                    pos,
                    force: None,
                });

                // The first finger down additionally becomes the egui pointer:
                // moved-then-pressed on Start, released-then-gone on End, so
                // widgets see the same sequence a mouse would produce.
                match action {
                    TouchAction::Start => {
                        if self.egui_context.pointer_touch_id.is_none() {
                            self.egui_context.pointer_touch_id = Some(id);
                            events.push(egui::Event::PointerMoved(pos));
                            events.push(egui::Event::PointerButton {
                                pos,
                                button: egui::PointerButton::Primary,
                                pressed: true,
                                modifiers: egui::Modifiers::default(),
                            });
                        }
                    }
                    TouchAction::Move => {
                        if self.egui_context.pointer_touch_id == Some(id) {
                            events.push(egui::Event::PointerMoved(pos));
                        }
                    }
                    TouchAction::End => {
                        if self.egui_context.pointer_touch_id == Some(id) {
                            self.egui_context.pointer_touch_id = None;
                            events.push(egui::Event::PointerButton {
                                pos,
                                button: egui::PointerButton::Primary,
                                pressed: false,
                                modifiers: egui::Modifiers::default(),
                            });
                            events.push(egui::Event::PointerGone);
                        }
                    }
                    TouchAction::Cancel => {
                        if self.egui_context.pointer_touch_id == Some(id) {
                            self.egui_context.pointer_touch_id = None;
                            events.push(egui::Event::PointerGone);
                        }
                    }
                }
            }
            WindowEvent::Char(ch) if !ch.is_control() => {
                self.egui_context
                    .raw_input
                    .events
                    .push(egui::Event::Text(ch.to_string()));
            }
            WindowEvent::Key(key, action, modifiers) => {
                if let Some(egui_key) = self.translate_key_to_egui(key) {
                    self.egui_context.raw_input.events.push(egui::Event::Key {
                        key: egui_key,
                        physical_key: None,
                        pressed: action == Action::Press,
                        repeat: false,
                        modifiers: egui_modifiers(modifiers),
                    });
                }
            }
            _ => {}
        }
    }

    pub(crate) fn translate_key_to_egui(&self, key: Key) -> Option<egui::Key> {
        Some(match key {
            Key::A => egui::Key::A,
            Key::B => egui::Key::B,
            Key::C => egui::Key::C,
            Key::D => egui::Key::D,
            Key::E => egui::Key::E,
            Key::F => egui::Key::F,
            Key::G => egui::Key::G,
            Key::H => egui::Key::H,
            Key::I => egui::Key::I,
            Key::J => egui::Key::J,
            Key::K => egui::Key::K,
            Key::L => egui::Key::L,
            Key::M => egui::Key::M,
            Key::N => egui::Key::N,
            Key::O => egui::Key::O,
            Key::P => egui::Key::P,
            Key::Q => egui::Key::Q,
            Key::R => egui::Key::R,
            Key::S => egui::Key::S,
            Key::T => egui::Key::T,
            Key::U => egui::Key::U,
            Key::V => egui::Key::V,
            Key::W => egui::Key::W,
            Key::X => egui::Key::X,
            Key::Y => egui::Key::Y,
            Key::Z => egui::Key::Z,
            Key::Escape => egui::Key::Escape,
            Key::Tab => egui::Key::Tab,
            Key::Back => egui::Key::Backspace,
            Key::Return => egui::Key::Enter,
            Key::Space => egui::Key::Space,
            Key::Insert => egui::Key::Insert,
            Key::Delete => egui::Key::Delete,
            Key::Home => egui::Key::Home,
            Key::End => egui::Key::End,
            Key::PageUp => egui::Key::PageUp,
            Key::PageDown => egui::Key::PageDown,
            Key::Left => egui::Key::ArrowLeft,
            Key::Up => egui::Key::ArrowUp,
            Key::Right => egui::Key::ArrowRight,
            Key::Down => egui::Key::ArrowDown,
            _ => return None,
        })
    }

    /// Draws an immediate mode UI using egui.
    ///
    /// Call this method from your render loop to create and display UI elements.
    /// The UI is drawn on top of the 3D scene.
    ///
    /// # Arguments
    /// * `ui_fn` - A closure that receives the egui Context and can create UI elements
    ///
    /// # Example
    /// ```no_run
    /// # use kiss3d::window::Window;
    /// # use kiss3d::camera::OrbitCamera3d;
    /// # use kiss3d::scene::SceneNode3d;
    /// # #[cfg(feature = "egui")]
    /// # #[kiss3d::main]
    /// # async fn main() {
    /// # let mut window = Window::new("Example").await;
    /// # let mut camera = OrbitCamera3d::default();
    /// # let mut scene = SceneNode3d::empty();
    /// while window.render_3d(&mut scene, &mut camera).await {
    ///     window.draw_ui(|ctx| {
    ///         egui::Window::new("My Window").show(ctx, |ui| {
    ///             ui.label("Hello, world!");
    ///             if ui.button("Click me").clicked() {
    ///                 println!("Button clicked!");
    ///             }
    ///         });
    ///     });
    /// }
    /// # }
    /// # #[cfg(not(feature = "egui"))]
    /// # fn main() {}
    /// ```
    ///
    /// # Note
    /// Only available when the `egui` feature is enabled.
    pub fn draw_ui<F>(&mut self, mut ui_fn: F)
    where
        F: FnMut(&egui::Context),
    {
        // A frame that calls this none of the times shows the last pass's
        // shapes again, which is what lets a host redraw its UI only when
        // something changed; the shapes are dropped by the next pass, not by
        // rendering them (`EguiRenderer::begin_frame`).
        //
        // Open the egui pass lazily so that several `draw_ui` (and
        // `draw_inspector`) calls in the same frame all run their widgets into
        // the *same* pass. The pass is closed at render time by
        // `finish_egui_pass`. Beginning a fresh pass per call would have the
        // second call's `end_frame` overwrite the first call's shapes (and the
        // `std::mem::take` below would starve it of input).
        let opened = !self.egui_context.pass_active;
        if opened {
            self.begin_egui_pass();
        }

        ui_fn(self.egui_context.renderer.context());

        // A pass that only learned a size — a new `Area`, which draws none of
        // itself while it is measured, a `Grid`, a `Resize` — asks to be
        // discarded and run again before the frame is shown. `Context::run`
        // does that for its callers; with the pass open here, this does, or
        // the frame shows the gap. Only the call that opened the pass reruns:
        // a rerun replays one closure, and an earlier call's shapes are gone.
        let max_passes = self
            .egui_context
            .renderer
            .context()
            .options(|options| options.max_passes.get());
        let mut passes = 1;
        while opened && passes < max_passes && self.egui_context.renderer.context().will_discard() {
            self.egui_context
                .renderer
                .rerun_frame(self.egui_context.rerun_input.clone());
            ui_fn(self.egui_context.renderer.context());
            passes += 1;
        }
    }

    /// Begins a new egui pass, feeding it the events accumulated since the last
    /// pass. Idempotent callers should guard on `pass_active`.
    fn begin_egui_pass(&mut self) {
        // Get time for animations - use egui context's own start time
        #[cfg(not(target_arch = "wasm32"))]
        let time = Some(self.egui_context.start_time.elapsed().as_secs_f64());
        #[cfg(target_arch = "wasm32")]
        let time = {
            use web_time::Instant;
            static START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
            let start = START.get_or_init(Instant::now);
            Some(start.elapsed().as_secs_f64())
        };

        let scale_factor = self.canvas.scale_factor() as f32;

        // Set pixels_per_point on the context to match our DPI scale
        self.egui_context
            .renderer
            .context()
            .set_pixels_per_point(scale_factor);

        // Build raw input with accumulated events
        let mut raw_input = std::mem::take(&mut self.egui_context.raw_input);
        raw_input.screen_rect = Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(
                self.width() as f32 / scale_factor,
                self.height() as f32 / scale_factor,
            ),
        ));
        raw_input.time = time;
        raw_input.predicted_dt = 1.0 / 60.0;

        // `take` leaves what `Context::run` hands a later pass of the same
        // frame: the time, and none of the events.
        let first = raw_input.take();
        self.egui_context.rerun_input = raw_input;
        self.egui_context.renderer.begin_frame(first);
        self.egui_context.pass_active = true;
    }

    /// Closes the egui pass opened by `draw_ui`/`draw_inspector`, if any, so the
    /// accumulated shapes are ready to be painted by the egui renderer. Called
    /// once per frame from the render path. No-op when no UI was drawn.
    pub(crate) fn finish_egui_pass(&mut self) {
        if self.egui_context.pass_active {
            self.egui_context.renderer.end_frame();
            self.egui_context.pass_active = false;
            // What the pass asked for, applied where the window is: egui says
            // which shape a widget wants and only the host can set one.
            let want = self.egui_context.renderer.cursor();
            if want != self.egui_context.cursor {
                self.egui_context.cursor = want;
                self.canvas.set_cursor_icon(winit_cursor(want));
            }
        }
        // Note: `raw_input` is *not* reset here. It is drained by
        // `begin_egui_pass` (via `std::mem::take`) when the next pass opens, and
        // events fed by `handle_events` between this point and that next pass
        // must be preserved — resetting here would discard them and the UI would
        // stop responding to input.
    }
}

/// egui's cursor names as winit's. The two lists are the same set under
/// different names, and neither crate knows about the other.
fn winit_cursor(icon: egui::CursorIcon) -> winit::window::CursorIcon {
    use egui::CursorIcon as E;
    use winit::window::CursorIcon as W;
    match icon {
        E::Default => W::Default,
        // Hiding the pointer is `hide_cursor`, not a shape; nothing in a
        // widget's own paint should take it off the screen.
        E::None => W::Default,
        E::ContextMenu => W::ContextMenu,
        E::Help => W::Help,
        E::PointingHand => W::Pointer,
        E::Progress => W::Progress,
        E::Wait => W::Wait,
        E::Cell => W::Cell,
        E::Crosshair => W::Crosshair,
        E::Text => W::Text,
        E::VerticalText => W::VerticalText,
        E::Alias => W::Alias,
        E::Copy => W::Copy,
        E::Move => W::Move,
        E::NoDrop => W::NoDrop,
        E::NotAllowed => W::NotAllowed,
        E::Grab => W::Grab,
        E::Grabbing => W::Grabbing,
        E::AllScroll => W::AllScroll,
        E::ResizeHorizontal => W::EwResize,
        E::ResizeNeSw => W::NeswResize,
        E::ResizeNwSe => W::NwseResize,
        E::ResizeVertical => W::NsResize,
        E::ResizeEast => W::EResize,
        E::ResizeSouthEast => W::SeResize,
        E::ResizeSouth => W::SResize,
        E::ResizeSouthWest => W::SwResize,
        E::ResizeWest => W::WResize,
        E::ResizeNorthWest => W::NwResize,
        E::ResizeNorth => W::NResize,
        E::ResizeNorthEast => W::NeResize,
        E::ResizeColumn => W::ColResize,
        E::ResizeRow => W::RowResize,
        E::ZoomIn => W::ZoomIn,
        E::ZoomOut => W::ZoomOut,
    }
}
