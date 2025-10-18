use std::num::NonZeroU32;
use std::sync::mpsc::Sender;

use crate::context::Context;
use crate::event::{Action, Key, Modifiers, MouseButton, TouchAction, WindowEvent};
use crate::window::canvas::{CanvasSetup, NumSamples};
use crate::window::AbstractCanvas;

use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextApi, ContextAttributesBuilder, PossiblyCurrentContext, Version};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{Surface, SwapInterval, WindowSurface};
use glutin_winit::{DisplayBuilder, GlWindow};

use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, MouseScrollDelta, TouchPhase, WindowEvent as WinitWindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};
use winit::raw_window_handle::HasWindowHandle;
use winit::window::{Icon, Window, WindowAttributes};

use image::{GenericImage, Pixel};

/// A canvas based on glutin and OpenGL.
pub struct GLCanvas {
    window: Window,
    gl_context: PossiblyCurrentContext,
    gl_surface: Surface<WindowSurface>,
    events: EventLoop<()>,
    cursor_pos: Option<(f64, f64)>,
    key_states: [Action; Key::Unknown as usize + 1],
    button_states: [Action; MouseButton::Button8 as usize + 1],
    out_events: Sender<WindowEvent>,
    modifiers_state: ModifiersState,
}

impl AbstractCanvas for GLCanvas {
    fn open(
        title: &str,
        hide: bool,
        width: u32,
        height: u32,
        canvas_setup: Option<CanvasSetup>,
        out_events: Sender<WindowEvent>,
    ) -> Self {
        let events = EventLoop::new().unwrap();

        let window_attrs = WindowAttributes::default()
            .with_title(title)
            .with_inner_size(LogicalSize::new(width as f64, height as f64))
            .with_visible(!hide);

        let canvas_setup = canvas_setup.unwrap_or(CanvasSetup {
            vsync: true,
            samples: NumSamples::Zero,
        });

        // Build the display and window
        let mut template = ConfigTemplateBuilder::new().with_alpha_size(8);

        // Only set multisampling if samples > 0
        if canvas_setup.samples as u8 > 0 {
            template = template.with_multisampling(canvas_setup.samples as u8);
        }

        let display_builder = DisplayBuilder::new().with_window_attributes(Some(window_attrs));

        let (window, gl_config) = display_builder
            .build(&events, template, |configs| {
                // Pick the config with the most samples
                configs
                    .reduce(|accum, config| {
                        if config.num_samples() > accum.num_samples() {
                            config
                        } else {
                            accum
                        }
                    })
                    .unwrap()
            })
            .unwrap();

        let window = window.unwrap();

        // Create OpenGL context
        let raw_window_handle = window.window_handle().ok().map(|wh| wh.as_raw());

        let context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(Version::new(3, 2))))
            .build(raw_window_handle);

        let fallback_context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::Gles(Some(Version::new(2, 0))))
            .build(raw_window_handle);

        let gl_display = gl_config.display();

        let not_current_gl_context = unsafe {
            gl_display
                .create_context(&gl_config, &context_attributes)
                .unwrap_or_else(|_| {
                    gl_display
                        .create_context(&gl_config, &fallback_context_attributes)
                        .expect("failed to create context")
                })
        };

        // Create surface
        let attrs = window
            .build_surface_attributes(Default::default())
            .expect("Failed to build surface attributes");
        let gl_surface = unsafe {
            gl_config
                .display()
                .create_window_surface(&gl_config, &attrs)
                .unwrap()
        };

        // Make context current
        let gl_context = not_current_gl_context.make_current(&gl_surface).unwrap();

        // Initialize glow context
        Context::init(|| unsafe {
            glow::Context::from_loader_function(|name| {
                gl_display.get_proc_address(std::ffi::CString::new(name).unwrap().as_c_str())
            })
        });

        // Set vsync
        if canvas_setup.vsync {
            let _ = gl_surface
                .set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()));
        }

        let ctxt = Context::get();
        let vao = ctxt.create_vertex_array();
        ctxt.bind_vertex_array(vao.as_ref());

        GLCanvas {
            window,
            gl_context,
            gl_surface,
            events,
            cursor_pos: None,
            key_states: [Action::Release; Key::Unknown as usize + 1],
            button_states: [Action::Release; MouseButton::Button8 as usize + 1],
            out_events,
            modifiers_state: ModifiersState::default(),
        }
    }

    fn poll_events(&mut self) {
        use winit::platform::pump_events::EventLoopExtPumpEvents;

        let out_events = &self.out_events;
        let button_states = &mut self.button_states;
        let key_states = &mut self.key_states;
        let cursor_pos = &mut self.cursor_pos;
        let modifiers_state = &mut self.modifiers_state;
        let gl_context = &self.gl_context;
        let gl_surface = &self.gl_surface;

        // Create a temporary event handler
        struct EventHandler<'a> {
            out_events: &'a Sender<WindowEvent>,
            button_states: &'a mut [Action],
            key_states: &'a mut [Action],
            cursor_pos: &'a mut Option<(f64, f64)>,
            modifiers_state: &'a mut ModifiersState,
            gl_context: &'a PossiblyCurrentContext,
            gl_surface: &'a Surface<WindowSurface>,
        }

        impl ApplicationHandler for EventHandler<'_> {
            fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

            fn window_event(
                &mut self,
                _event_loop: &ActiveEventLoop,
                _window_id: winit::window::WindowId,
                event: WinitWindowEvent,
            ) {
                match event {
                    WinitWindowEvent::CloseRequested => {
                        let _ = self.out_events.send(WindowEvent::Close);
                    }
                    WinitWindowEvent::Resized(physical_size) => {
                        if physical_size.width > 0 && physical_size.height > 0 {
                            self.gl_surface.resize(
                                self.gl_context,
                                NonZeroU32::new(physical_size.width).unwrap(),
                                NonZeroU32::new(physical_size.height).unwrap(),
                            );
                            let _ = self.out_events.send(WindowEvent::FramebufferSize(
                                physical_size.width,
                                physical_size.height,
                            ));
                        }
                    }
                    WinitWindowEvent::CursorMoved { position, .. } => {
                        let modifiers = translate_modifiers(*self.modifiers_state);
                        *self.cursor_pos = Some((position.x, position.y));
                        let _ = self
                            .out_events
                            .send(WindowEvent::CursorPos(position.x, position.y, modifiers));
                    }
                    WinitWindowEvent::MouseInput { state, button, .. } => {
                        let action = translate_action(state);
                        let button = translate_mouse_button(button);
                        let modifiers = translate_modifiers(*self.modifiers_state);
                        self.button_states[button as usize] = action;
                        let _ = self
                            .out_events
                            .send(WindowEvent::MouseButton(button, action, modifiers));
                    }
                    WinitWindowEvent::Touch(touch) => {
                        let action = match touch.phase {
                            TouchPhase::Started => TouchAction::Start,
                            TouchPhase::Ended => TouchAction::End,
                            TouchPhase::Moved => TouchAction::Move,
                            TouchPhase::Cancelled => TouchAction::Cancel,
                        };

                        let _ = self.out_events.send(WindowEvent::Touch(
                            touch.id,
                            touch.location.x,
                            touch.location.y,
                            action,
                            Modifiers::empty(),
                        ));
                    }
                    WinitWindowEvent::MouseWheel { delta, .. } => {
                        let (x, y) = match delta {
                            MouseScrollDelta::LineDelta(dx, dy) => {
                                (dx as f64 * 10.0, dy as f64 * 10.0)
                            }
                            MouseScrollDelta::PixelDelta(delta) => (delta.x, delta.y),
                        };
                        let modifiers = translate_modifiers(*self.modifiers_state);
                        let _ = self.out_events.send(WindowEvent::Scroll(x, y, modifiers));
                    }
                    WinitWindowEvent::KeyboardInput { event, .. } => {
                        let action = translate_action(event.state);
                        let key = translate_key(event.physical_key);
                        let modifiers = translate_modifiers(*self.modifiers_state);
                        self.key_states[key as usize] = action;
                        let _ = self
                            .out_events
                            .send(WindowEvent::Key(key, action, modifiers));

                        // Also send character event if available
                        if let winit::keyboard::Key::Character(ref c) = event.logical_key {
                            for ch in c.chars() {
                                let _ = self.out_events.send(WindowEvent::Char(ch));
                            }
                        }
                    }
                    WinitWindowEvent::ModifiersChanged(new_modifiers) => {
                        *self.modifiers_state = new_modifiers.state();
                    }
                    _ => {}
                }
            }
        }

        let mut handler = EventHandler {
            out_events,
            button_states,
            key_states,
            cursor_pos,
            modifiers_state,
            gl_context,
            gl_surface,
        };

        let timeout = Some(std::time::Duration::ZERO);
        let _ = self.events.pump_app_events(timeout, &mut handler);
    }

    fn swap_buffers(&mut self) {
        let _ = self.gl_surface.swap_buffers(&self.gl_context);
    }

    fn size(&self) -> (u32, u32) {
        self.window.inner_size().into()
    }

    fn cursor_pos(&self) -> Option<(f64, f64)> {
        self.cursor_pos
    }

    fn scale_factor(&self) -> f64 {
        self.window.scale_factor()
    }

    fn set_title(&mut self, title: &str) {
        self.window.set_title(title)
    }

    fn set_icon(&mut self, icon: impl GenericImage<Pixel = impl Pixel<Subpixel = u8>>) {
        let (width, height) = icon.dimensions();
        let mut rgba = Vec::with_capacity((width * height) as usize * 4);
        for (_, _, pixel) in icon.pixels() {
            rgba.extend_from_slice(&pixel.to_rgba().0);
        }
        let icon = Icon::from_rgba(rgba, width, height).unwrap();
        self.window.set_window_icon(Some(icon))
    }

    fn set_cursor_grab(&self, grab: bool) {
        use winit::window::CursorGrabMode;
        let mode = if grab {
            CursorGrabMode::Confined
        } else {
            CursorGrabMode::None
        };
        let _ = self.window.set_cursor_grab(mode);
    }

    fn set_cursor_position(&self, x: f64, y: f64) {
        let _ = self
            .window
            .set_cursor_position(winit::dpi::PhysicalPosition::new(x, y));
    }

    fn hide_cursor(&self, hide: bool) {
        self.window.set_cursor_visible(!hide)
    }

    fn hide(&mut self) {
        self.window.set_visible(false)
    }

    fn show(&mut self) {
        self.window.set_visible(true)
    }

    fn get_mouse_button(&self, button: MouseButton) -> Action {
        self.button_states[button as usize]
    }
    fn get_key(&self, key: Key) -> Action {
        self.key_states[key as usize]
    }
}

fn translate_action(action: ElementState) -> Action {
    match action {
        ElementState::Pressed => Action::Press,
        ElementState::Released => Action::Release,
    }
}

fn translate_modifiers(modifiers: ModifiersState) -> Modifiers {
    let mut res = Modifiers::empty();
    if modifiers.shift_key() {
        res.insert(Modifiers::Shift)
    }
    if modifiers.control_key() {
        res.insert(Modifiers::Control)
    }
    if modifiers.alt_key() {
        res.insert(Modifiers::Alt)
    }
    if modifiers.super_key() {
        res.insert(Modifiers::Super)
    }

    res
}

fn translate_mouse_button(button: winit::event::MouseButton) -> MouseButton {
    match button {
        winit::event::MouseButton::Left => MouseButton::Button1,
        winit::event::MouseButton::Right => MouseButton::Button2,
        winit::event::MouseButton::Middle => MouseButton::Button3,
        _ => MouseButton::Button4,
    }
}

fn translate_key(physical_key: PhysicalKey) -> Key {
    if let PhysicalKey::Code(key_code) = physical_key {
        match key_code {
            KeyCode::Digit1 => Key::Key1,
            KeyCode::Digit2 => Key::Key2,
            KeyCode::Digit3 => Key::Key3,
            KeyCode::Digit4 => Key::Key4,
            KeyCode::Digit5 => Key::Key5,
            KeyCode::Digit6 => Key::Key6,
            KeyCode::Digit7 => Key::Key7,
            KeyCode::Digit8 => Key::Key8,
            KeyCode::Digit9 => Key::Key9,
            KeyCode::Digit0 => Key::Key0,
            KeyCode::KeyA => Key::A,
            KeyCode::KeyB => Key::B,
            KeyCode::KeyC => Key::C,
            KeyCode::KeyD => Key::D,
            KeyCode::KeyE => Key::E,
            KeyCode::KeyF => Key::F,
            KeyCode::KeyG => Key::G,
            KeyCode::KeyH => Key::H,
            KeyCode::KeyI => Key::I,
            KeyCode::KeyJ => Key::J,
            KeyCode::KeyK => Key::K,
            KeyCode::KeyL => Key::L,
            KeyCode::KeyM => Key::M,
            KeyCode::KeyN => Key::N,
            KeyCode::KeyO => Key::O,
            KeyCode::KeyP => Key::P,
            KeyCode::KeyQ => Key::Q,
            KeyCode::KeyR => Key::R,
            KeyCode::KeyS => Key::S,
            KeyCode::KeyT => Key::T,
            KeyCode::KeyU => Key::U,
            KeyCode::KeyV => Key::V,
            KeyCode::KeyW => Key::W,
            KeyCode::KeyX => Key::X,
            KeyCode::KeyY => Key::Y,
            KeyCode::KeyZ => Key::Z,
            KeyCode::Escape => Key::Escape,
            KeyCode::F1 => Key::F1,
            KeyCode::F2 => Key::F2,
            KeyCode::F3 => Key::F3,
            KeyCode::F4 => Key::F4,
            KeyCode::F5 => Key::F5,
            KeyCode::F6 => Key::F6,
            KeyCode::F7 => Key::F7,
            KeyCode::F8 => Key::F8,
            KeyCode::F9 => Key::F9,
            KeyCode::F10 => Key::F10,
            KeyCode::F11 => Key::F11,
            KeyCode::F12 => Key::F12,
            KeyCode::F13 => Key::F13,
            KeyCode::F14 => Key::F14,
            KeyCode::F15 => Key::F15,
            KeyCode::F16 => Key::F16,
            KeyCode::F17 => Key::F17,
            KeyCode::F18 => Key::F18,
            KeyCode::F19 => Key::F19,
            KeyCode::F20 => Key::F20,
            KeyCode::F21 => Key::F21,
            KeyCode::F22 => Key::F22,
            KeyCode::F23 => Key::F23,
            KeyCode::F24 => Key::F24,
            KeyCode::PrintScreen => Key::Snapshot,
            KeyCode::ScrollLock => Key::Scroll,
            KeyCode::Pause => Key::Pause,
            KeyCode::Insert => Key::Insert,
            KeyCode::Home => Key::Home,
            KeyCode::Delete => Key::Delete,
            KeyCode::End => Key::End,
            KeyCode::PageDown => Key::PageDown,
            KeyCode::PageUp => Key::PageUp,
            KeyCode::ArrowLeft => Key::Left,
            KeyCode::ArrowUp => Key::Up,
            KeyCode::ArrowRight => Key::Right,
            KeyCode::ArrowDown => Key::Down,
            KeyCode::Backspace => Key::Back,
            KeyCode::Enter => Key::Return,
            KeyCode::Space => Key::Space,
            // KeyCode::Compose doesn't exist in winit 0.30
            KeyCode::NumLock => Key::Numlock,
            KeyCode::Numpad0 => Key::Numpad0,
            KeyCode::Numpad1 => Key::Numpad1,
            KeyCode::Numpad2 => Key::Numpad2,
            KeyCode::Numpad3 => Key::Numpad3,
            KeyCode::Numpad4 => Key::Numpad4,
            KeyCode::Numpad5 => Key::Numpad5,
            KeyCode::Numpad6 => Key::Numpad6,
            KeyCode::Numpad7 => Key::Numpad7,
            KeyCode::Numpad8 => Key::Numpad8,
            KeyCode::Numpad9 => Key::Numpad9,
            KeyCode::NumpadAdd => Key::Add,
            KeyCode::Quote => Key::Apostrophe,
            // KeyCode::Apps doesn't exist in winit 0.30
            KeyCode::Backslash => Key::Backslash,
            KeyCode::NumpadClear => Key::NumpadEquals,
            KeyCode::Comma => Key::Comma,
            KeyCode::Convert => Key::Convert,
            KeyCode::NumpadDecimal => Key::Decimal,
            KeyCode::NumpadDivide => Key::Divide,
            KeyCode::NumpadMultiply => Key::Multiply,
            KeyCode::Equal => Key::Equals,
            KeyCode::Backquote => Key::Grave,
            KeyCode::KanaMode => Key::Kana,
            // KeyCode::KanjiMode doesn't exist in winit 0.30, using KanaMode instead
            KeyCode::AltLeft => Key::LAlt,
            KeyCode::BracketLeft => Key::LBracket,
            KeyCode::ControlLeft => Key::LControl,
            KeyCode::ShiftLeft => Key::LShift,
            KeyCode::SuperLeft => Key::LWin,
            KeyCode::LaunchMail => Key::Mail,
            KeyCode::MediaSelect => Key::MediaSelect,
            KeyCode::MediaStop => Key::MediaStop,
            KeyCode::Minus => Key::Minus,
            KeyCode::AudioVolumeMute => Key::Mute,
            KeyCode::BrowserForward => Key::NavigateForward,
            KeyCode::BrowserBack => Key::NavigateBackward,
            KeyCode::MediaTrackNext => Key::NextTrack,
            KeyCode::NonConvert => Key::NoConvert,
            KeyCode::NumpadComma => Key::NumpadComma,
            KeyCode::NumpadEnter => Key::NumpadEnter,
            KeyCode::IntlBackslash => Key::OEM102,
            KeyCode::Period => Key::Period,
            KeyCode::MediaPlayPause => Key::PlayPause,
            KeyCode::Power => Key::Power,
            KeyCode::MediaTrackPrevious => Key::PrevTrack,
            KeyCode::AltRight => Key::RAlt,
            KeyCode::BracketRight => Key::RBracket,
            KeyCode::ControlRight => Key::RControl,
            KeyCode::ShiftRight => Key::RShift,
            KeyCode::SuperRight => Key::RWin,
            KeyCode::Semicolon => Key::Semicolon,
            KeyCode::Slash => Key::Slash,
            KeyCode::Sleep => Key::Sleep,
            KeyCode::NumpadSubtract => Key::Subtract,
            KeyCode::Tab => Key::Tab,
            KeyCode::AudioVolumeDown => Key::VolumeDown,
            KeyCode::AudioVolumeUp => Key::VolumeUp,
            KeyCode::WakeUp => Key::Wake,
            KeyCode::BrowserHome => Key::WebHome,
            KeyCode::BrowserRefresh => Key::WebRefresh,
            KeyCode::BrowserSearch => Key::WebSearch,
            KeyCode::IntlYen => Key::Yen,
            KeyCode::Copy => Key::Copy,
            KeyCode::Paste => Key::Paste,
            KeyCode::Cut => Key::Cut,
            _ => Key::Unknown,
        }
    } else {
        Key::Unknown
    }
}
