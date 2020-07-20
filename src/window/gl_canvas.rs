use std::sync::mpsc::Sender;

use crate::event::{Action, Key, Modifiers, MouseButton, TouchAction, WindowEvent};
use gl;
use glutin::{
    self, dpi::LogicalSize, ContextBuilder, EventsLoop, GlContext, GlRequest, GlWindow, TouchPhase,
    WindowBuilder,
};
use image::{GenericImage, Pixel};
use crate::window::canvas::{CanvasSetup, NumSamples};
use crate::window::AbstractCanvas;

/// A canvas based on glutin and OpenGL.
pub struct GLCanvas {
    window: GlWindow,
    events: EventsLoop,
    cursor_pos: Option<(f64, f64)>,
    key_states: [Action; Key::Unknown as usize + 1],
    button_states: [Action; MouseButton::Button8 as usize + 1],
    out_events: Sender<WindowEvent>,
    // listeners: Vec<EventListenerHandle>,
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
        let events = EventsLoop::new();
        let window = WindowBuilder::new()
            .with_title(title)
            .with_dimensions(LogicalSize::new(width as f64, height as f64))
            .with_visibility(!hide);
        let canvas_setup = canvas_setup.unwrap_or(CanvasSetup {
            vsync: true,
            samples: NumSamples::Zero,
        });
        let context = ContextBuilder::new()
            .with_vsync(canvas_setup.vsync)
            .with_multisampling(canvas_setup.samples as u16)
            .with_gl(GlRequest::GlThenGles {
                opengl_version: (3, 2),
                opengles_version: (2, 0),
            });
        let window = GlWindow::new(window, context, &events).unwrap();
        let _ = unsafe { window.make_current().unwrap() };
        crate::context::Context::init(|| {
            glow::Context::from_loader_function(|name| {
                window.context().get_proc_address(name) as *const _
            })
        });
        verify!(gl::load_with(
            |name| window.context().get_proc_address(name) as *const _
        ));

        crate::context::Context::with_mut(|ctx| ctx.init_vao());

        GLCanvas {
            window,
            events,
            cursor_pos: None,
            key_states: [Action::Release; Key::Unknown as usize + 1],
            button_states: [Action::Release; MouseButton::Button8 as usize + 1],
            out_events,
        }
    }

    fn render_loop(mut callback: impl FnMut(f64) -> bool + 'static) {
        loop {
            if !callback(0.0) {
                break;
            } // XXX: timestamp
        }
    }

    fn poll_events(&mut self) {
        let out_events = &mut self.out_events;
        let window = &mut self.window;
        let button_states = &mut self.button_states;
        let key_states = &mut self.key_states;
        let cursor_pos = &mut self.cursor_pos;

        self.events.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::CloseRequested => {
                    let _ = out_events.send(WindowEvent::Close);
                }
                glutin::WindowEvent::Resized(logical_size) => {
                    let dpi_factor = window.get_hidpi_factor();
                    let physical_size = logical_size.to_physical(dpi_factor);
                    window.resize(physical_size);
                    let fb_size: (u32, u32) = physical_size.into();
                    let _ = out_events.send(WindowEvent::FramebufferSize(fb_size.0, fb_size.1));
                }
                glutin::WindowEvent::CursorMoved {
                    position,
                    modifiers,
                    ..
                } => {
                    let modifiers = translate_modifiers(modifiers);
                    let dpi_factor = window.get_hidpi_factor();
                    let physical_pos = position.to_physical(dpi_factor);
                    *cursor_pos = Some(physical_pos.into());
                    let _ = out_events.send(WindowEvent::CursorPos(
                        physical_pos.x,
                        physical_pos.y,
                        modifiers,
                    ));
                }
                glutin::WindowEvent::MouseInput {
                    state,
                    button,
                    modifiers,
                    ..
                } => {
                    let action = translate_action(state);
                    let button = translate_mouse_button(button);
                    let modifiers = translate_modifiers(modifiers);
                    button_states[button as usize] = action;
                    let _ = out_events.send(WindowEvent::MouseButton(button, action, modifiers));
                }
                glutin::WindowEvent::Touch(touch) => {
                    let action = match touch.phase {
                        TouchPhase::Started => TouchAction::Start,
                        TouchPhase::Ended => TouchAction::End,
                        TouchPhase::Moved => TouchAction::Move,
                        TouchPhase::Cancelled => TouchAction::Cancel,
                    };

                    let _ = out_events.send(WindowEvent::Touch(
                        touch.id,
                        touch.location.x,
                        touch.location.y,
                        action,
                        Modifiers::empty(),
                    ));
                }
                glutin::WindowEvent::MouseWheel {
                    delta, modifiers, ..
                } => {
                    let (x, y) = match delta {
                        glutin::MouseScrollDelta::LineDelta(dx, dy) => {
                            (dx as f64 * 10.0, dy as f64 * 10.0)
                        }
                        glutin::MouseScrollDelta::PixelDelta(delta) => delta.into(),
                    };
                    let modifiers = translate_modifiers(modifiers);
                    let _ = out_events.send(WindowEvent::Scroll(x, y, modifiers));
                }
                glutin::WindowEvent::KeyboardInput { input, .. } => {
                    let action = translate_action(input.state);
                    let key = translate_key(input.virtual_keycode);
                    let modifiers = translate_modifiers(input.modifiers);
                    key_states[key as usize] = action;
                    let _ = out_events.send(WindowEvent::Key(key, action, modifiers));
                }
                glutin::WindowEvent::ReceivedCharacter(c) => {
                    let _ = out_events.send(WindowEvent::Char(c));
                }
                _ => {}
            },
            _ => {}
        })
    }

    fn swap_buffers(&mut self) {
        let _ = self.window.swap_buffers();
    }

    fn size(&self) -> (u32, u32) {
        let hidpi = self.window.get_hidpi_factor();
        let logical_size = self
            .window
            .get_inner_size()
            .expect("The window was closed.");
        logical_size.to_physical(hidpi).into()
    }

    fn cursor_pos(&self) -> Option<(f64, f64)> {
        self.cursor_pos
    }

    fn hidpi_factor(&self) -> f64 {
        self.window.get_hidpi_factor() as f64
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
        let icon = glutin::Icon::from_rgba(rgba, width, height).unwrap();
        self.window.set_window_icon(Some(icon))
    }

    fn set_cursor_grab(&self, grab: bool) {
        let _ = self.window.window().grab_cursor(grab);
    }

    fn hide(&mut self) {
        self.window.hide()
    }

    fn show(&mut self) {
        self.window.show()
    }

    fn get_mouse_button(&self, button: MouseButton) -> Action {
        self.button_states[button as usize]
    }
    fn get_key(&self, key: Key) -> Action {
        self.key_states[key as usize]
    }
}

fn translate_action(action: glutin::ElementState) -> Action {
    match action {
        glutin::ElementState::Pressed => Action::Press,
        glutin::ElementState::Released => Action::Release,
    }
}

fn translate_modifiers(modifiers: glutin::ModifiersState) -> Modifiers {
    let mut res = Modifiers::empty();
    if modifiers.shift {
        res.insert(Modifiers::Shift)
    }
    if modifiers.ctrl {
        res.insert(Modifiers::Control)
    }
    if modifiers.alt {
        res.insert(Modifiers::Alt)
    }
    if modifiers.logo {
        res.insert(Modifiers::Super)
    }

    res
}

fn translate_mouse_button(button: glutin::MouseButton) -> MouseButton {
    match button {
        glutin::MouseButton::Left => MouseButton::Button1,
        glutin::MouseButton::Right => MouseButton::Button2,
        glutin::MouseButton::Middle => MouseButton::Button3,
        glutin::MouseButton::Other(_) => MouseButton::Button4, // XXX: the default is not good.
    }
}

fn translate_key(button: Option<glutin::VirtualKeyCode>) -> Key {
    if let Some(button) = button {
        match button {
            glutin::VirtualKeyCode::Key1 => Key::Key1,
            glutin::VirtualKeyCode::Key2 => Key::Key2,
            glutin::VirtualKeyCode::Key3 => Key::Key3,
            glutin::VirtualKeyCode::Key4 => Key::Key4,
            glutin::VirtualKeyCode::Key5 => Key::Key5,
            glutin::VirtualKeyCode::Key6 => Key::Key6,
            glutin::VirtualKeyCode::Key7 => Key::Key7,
            glutin::VirtualKeyCode::Key8 => Key::Key8,
            glutin::VirtualKeyCode::Key9 => Key::Key9,
            glutin::VirtualKeyCode::Key0 => Key::Key0,
            glutin::VirtualKeyCode::A => Key::A,
            glutin::VirtualKeyCode::B => Key::B,
            glutin::VirtualKeyCode::C => Key::C,
            glutin::VirtualKeyCode::D => Key::D,
            glutin::VirtualKeyCode::E => Key::E,
            glutin::VirtualKeyCode::F => Key::F,
            glutin::VirtualKeyCode::G => Key::G,
            glutin::VirtualKeyCode::H => Key::H,
            glutin::VirtualKeyCode::I => Key::I,
            glutin::VirtualKeyCode::J => Key::J,
            glutin::VirtualKeyCode::K => Key::K,
            glutin::VirtualKeyCode::L => Key::L,
            glutin::VirtualKeyCode::M => Key::M,
            glutin::VirtualKeyCode::N => Key::N,
            glutin::VirtualKeyCode::O => Key::O,
            glutin::VirtualKeyCode::P => Key::P,
            glutin::VirtualKeyCode::Q => Key::Q,
            glutin::VirtualKeyCode::R => Key::R,
            glutin::VirtualKeyCode::S => Key::S,
            glutin::VirtualKeyCode::T => Key::T,
            glutin::VirtualKeyCode::U => Key::U,
            glutin::VirtualKeyCode::V => Key::V,
            glutin::VirtualKeyCode::W => Key::W,
            glutin::VirtualKeyCode::X => Key::X,
            glutin::VirtualKeyCode::Y => Key::Y,
            glutin::VirtualKeyCode::Z => Key::Z,
            glutin::VirtualKeyCode::Escape => Key::Escape,
            glutin::VirtualKeyCode::F1 => Key::F1,
            glutin::VirtualKeyCode::F2 => Key::F2,
            glutin::VirtualKeyCode::F3 => Key::F3,
            glutin::VirtualKeyCode::F4 => Key::F4,
            glutin::VirtualKeyCode::F5 => Key::F5,
            glutin::VirtualKeyCode::F6 => Key::F6,
            glutin::VirtualKeyCode::F7 => Key::F7,
            glutin::VirtualKeyCode::F8 => Key::F8,
            glutin::VirtualKeyCode::F9 => Key::F9,
            glutin::VirtualKeyCode::F10 => Key::F10,
            glutin::VirtualKeyCode::F11 => Key::F11,
            glutin::VirtualKeyCode::F12 => Key::F12,
            glutin::VirtualKeyCode::F13 => Key::F13,
            glutin::VirtualKeyCode::F14 => Key::F14,
            glutin::VirtualKeyCode::F15 => Key::F15,
            glutin::VirtualKeyCode::F16 => Key::F16,
            glutin::VirtualKeyCode::F17 => Key::F17,
            glutin::VirtualKeyCode::F18 => Key::F18,
            glutin::VirtualKeyCode::F19 => Key::F19,
            glutin::VirtualKeyCode::F20 => Key::F20,
            glutin::VirtualKeyCode::F21 => Key::F21,
            glutin::VirtualKeyCode::F22 => Key::F22,
            glutin::VirtualKeyCode::F23 => Key::F23,
            glutin::VirtualKeyCode::F24 => Key::F24,
            glutin::VirtualKeyCode::Snapshot => Key::Snapshot,
            glutin::VirtualKeyCode::Scroll => Key::Scroll,
            glutin::VirtualKeyCode::Pause => Key::Pause,
            glutin::VirtualKeyCode::Insert => Key::Insert,
            glutin::VirtualKeyCode::Home => Key::Home,
            glutin::VirtualKeyCode::Delete => Key::Delete,
            glutin::VirtualKeyCode::End => Key::End,
            glutin::VirtualKeyCode::PageDown => Key::PageDown,
            glutin::VirtualKeyCode::PageUp => Key::PageUp,
            glutin::VirtualKeyCode::Left => Key::Left,
            glutin::VirtualKeyCode::Up => Key::Up,
            glutin::VirtualKeyCode::Right => Key::Right,
            glutin::VirtualKeyCode::Down => Key::Down,
            glutin::VirtualKeyCode::Back => Key::Back,
            glutin::VirtualKeyCode::Return => Key::Return,
            glutin::VirtualKeyCode::Space => Key::Space,
            glutin::VirtualKeyCode::Compose => Key::Compose,
            glutin::VirtualKeyCode::Caret => Key::Caret,
            glutin::VirtualKeyCode::Numlock => Key::Numlock,
            glutin::VirtualKeyCode::Numpad0 => Key::Numpad0,
            glutin::VirtualKeyCode::Numpad1 => Key::Numpad1,
            glutin::VirtualKeyCode::Numpad2 => Key::Numpad2,
            glutin::VirtualKeyCode::Numpad3 => Key::Numpad3,
            glutin::VirtualKeyCode::Numpad4 => Key::Numpad4,
            glutin::VirtualKeyCode::Numpad5 => Key::Numpad5,
            glutin::VirtualKeyCode::Numpad6 => Key::Numpad6,
            glutin::VirtualKeyCode::Numpad7 => Key::Numpad7,
            glutin::VirtualKeyCode::Numpad8 => Key::Numpad8,
            glutin::VirtualKeyCode::Numpad9 => Key::Numpad9,
            glutin::VirtualKeyCode::AbntC1 => Key::AbntC1,
            glutin::VirtualKeyCode::AbntC2 => Key::AbntC2,
            glutin::VirtualKeyCode::Add => Key::Add,
            glutin::VirtualKeyCode::Apostrophe => Key::Apostrophe,
            glutin::VirtualKeyCode::Apps => Key::Apps,
            glutin::VirtualKeyCode::At => Key::At,
            glutin::VirtualKeyCode::Ax => Key::Ax,
            glutin::VirtualKeyCode::Backslash => Key::Backslash,
            glutin::VirtualKeyCode::Calculator => Key::Calculator,
            glutin::VirtualKeyCode::Capital => Key::Capital,
            glutin::VirtualKeyCode::Colon => Key::Colon,
            glutin::VirtualKeyCode::Comma => Key::Comma,
            glutin::VirtualKeyCode::Convert => Key::Convert,
            glutin::VirtualKeyCode::Decimal => Key::Decimal,
            glutin::VirtualKeyCode::Divide => Key::Divide,
            glutin::VirtualKeyCode::Equals => Key::Equals,
            glutin::VirtualKeyCode::Grave => Key::Grave,
            glutin::VirtualKeyCode::Kana => Key::Kana,
            glutin::VirtualKeyCode::Kanji => Key::Kanji,
            glutin::VirtualKeyCode::LAlt => Key::LAlt,
            glutin::VirtualKeyCode::LBracket => Key::LBracket,
            glutin::VirtualKeyCode::LControl => Key::LControl,
            glutin::VirtualKeyCode::LShift => Key::LShift,
            glutin::VirtualKeyCode::LWin => Key::LWin,
            glutin::VirtualKeyCode::Mail => Key::Mail,
            glutin::VirtualKeyCode::MediaSelect => Key::MediaSelect,
            glutin::VirtualKeyCode::MediaStop => Key::MediaStop,
            glutin::VirtualKeyCode::Minus => Key::Minus,
            glutin::VirtualKeyCode::Multiply => Key::Multiply,
            glutin::VirtualKeyCode::Mute => Key::Mute,
            glutin::VirtualKeyCode::MyComputer => Key::MyComputer,
            glutin::VirtualKeyCode::NavigateForward => Key::NavigateForward,
            glutin::VirtualKeyCode::NavigateBackward => Key::NavigateBackward,
            glutin::VirtualKeyCode::NextTrack => Key::NextTrack,
            glutin::VirtualKeyCode::NoConvert => Key::NoConvert,
            glutin::VirtualKeyCode::NumpadComma => Key::NumpadComma,
            glutin::VirtualKeyCode::NumpadEnter => Key::NumpadEnter,
            glutin::VirtualKeyCode::NumpadEquals => Key::NumpadEquals,
            glutin::VirtualKeyCode::OEM102 => Key::OEM102,
            glutin::VirtualKeyCode::Period => Key::Period,
            glutin::VirtualKeyCode::PlayPause => Key::PlayPause,
            glutin::VirtualKeyCode::Power => Key::Power,
            glutin::VirtualKeyCode::PrevTrack => Key::PrevTrack,
            glutin::VirtualKeyCode::RAlt => Key::RAlt,
            glutin::VirtualKeyCode::RBracket => Key::RBracket,
            glutin::VirtualKeyCode::RControl => Key::RControl,
            glutin::VirtualKeyCode::RShift => Key::RShift,
            glutin::VirtualKeyCode::RWin => Key::RWin,
            glutin::VirtualKeyCode::Semicolon => Key::Semicolon,
            glutin::VirtualKeyCode::Slash => Key::Slash,
            glutin::VirtualKeyCode::Sleep => Key::Sleep,
            glutin::VirtualKeyCode::Stop => Key::Stop,
            glutin::VirtualKeyCode::Subtract => Key::Subtract,
            glutin::VirtualKeyCode::Sysrq => Key::Sysrq,
            glutin::VirtualKeyCode::Tab => Key::Tab,
            glutin::VirtualKeyCode::Underline => Key::Underline,
            glutin::VirtualKeyCode::Unlabeled => Key::Unlabeled,
            glutin::VirtualKeyCode::VolumeDown => Key::VolumeDown,
            glutin::VirtualKeyCode::VolumeUp => Key::VolumeUp,
            glutin::VirtualKeyCode::Wake => Key::Wake,
            glutin::VirtualKeyCode::WebBack => Key::WebBack,
            glutin::VirtualKeyCode::WebFavorites => Key::WebFavorites,
            glutin::VirtualKeyCode::WebForward => Key::WebForward,
            glutin::VirtualKeyCode::WebHome => Key::WebHome,
            glutin::VirtualKeyCode::WebRefresh => Key::WebRefresh,
            glutin::VirtualKeyCode::WebSearch => Key::WebSearch,
            glutin::VirtualKeyCode::WebStop => Key::WebStop,
            glutin::VirtualKeyCode::Yen => Key::Yen,
            glutin::VirtualKeyCode::Copy => Key::Copy,
            glutin::VirtualKeyCode::Paste => Key::Paste,
            glutin::VirtualKeyCode::Cut => Key::Cut,
        }
    } else {
        Key::Unknown
    }
}
