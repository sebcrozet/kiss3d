use std::sync::mpsc::Sender;

use crate::context::Context;
use crate::event::{Action, Key, Modifiers, MouseButton, TouchAction, WindowEvent};
use crate::window::canvas::{CanvasSetup, NumSamples};
use crate::window::AbstractCanvas;
use glutin::{
    self,
    dpi::LogicalSize,
    event::TouchPhase,
    event_loop::{ControlFlow, EventLoop},
    platform::desktop::EventLoopExtDesktop,
    window::WindowBuilder,
    ContextBuilder, GlRequest, PossiblyCurrent, WindowedContext,
};
use image::{GenericImage, Pixel};

/// A canvas based on glutin and OpenGL.
pub struct GLCanvas {
    window: WindowedContext<PossiblyCurrent>,
    events: EventLoop<()>,
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
        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        let events = {
            use glutin::platform::unix::EventLoopExtUnix;
            EventLoop::new_any_thread()
        };
        #[cfg(windows)]
        let events = {
            use glutin::platform::windows::EventLoopExtWindows;
            EventLoop::new_any_thread()
        };
        #[cfg(not(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd",
            windows
        )))]
        let events = EventLoop::new();

        let window = WindowBuilder::new()
            .with_title(title)
            .with_inner_size(LogicalSize::new(width as f64, height as f64))
            .with_visible(!hide);
        let canvas_setup = canvas_setup.unwrap_or(CanvasSetup {
            vsync: true,
            samples: NumSamples::Zero,
        });
        let window = ContextBuilder::new()
            .with_vsync(canvas_setup.vsync)
            .with_multisampling(canvas_setup.samples as u16)
            .with_gl(GlRequest::GlThenGles {
                opengl_version: (3, 2),
                opengles_version: (2, 0),
            })
            .build_windowed(window, &events)
            .unwrap();
        let window = unsafe { window.make_current().unwrap() };
        Context::init(|| unsafe {
            glow::Context::from_loader_function(|name| window.get_proc_address(name) as *const _)
        });

        let ctxt = Context::get();
        let vao = ctxt.create_vertex_array();
        ctxt.bind_vertex_array(vao.as_ref());

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

        self.events.run_return(|event, _, control_flow| {
            use glutin::event::Event;

            match event {
                Event::WindowEvent { event, .. } => match event {
                    glutin::event::WindowEvent::CloseRequested => {
                        let _ = out_events.send(WindowEvent::Close);
                    }
                    glutin::event::WindowEvent::Resized(physical_size) => {
                        window.resize(physical_size);
                        let fb_size: (u32, u32) = physical_size.into();
                        let _ = out_events.send(WindowEvent::FramebufferSize(fb_size.0, fb_size.1));
                    }
                    glutin::event::WindowEvent::CursorMoved {
                        position,
                        modifiers,
                        ..
                    } => {
                        let modifiers = translate_modifiers(modifiers);
                        *cursor_pos = Some(position.into());
                        let _ = out_events
                            .send(WindowEvent::CursorPos(position.x, position.y, modifiers));
                    }
                    glutin::event::WindowEvent::MouseInput {
                        state,
                        button,
                        modifiers,
                        ..
                    } => {
                        let action = translate_action(state);
                        let button = translate_mouse_button(button);
                        let modifiers = translate_modifiers(modifiers);
                        button_states[button as usize] = action;
                        let _ =
                            out_events.send(WindowEvent::MouseButton(button, action, modifiers));
                    }
                    glutin::event::WindowEvent::Touch(touch) => {
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
                    glutin::event::WindowEvent::MouseWheel {
                        delta, modifiers, ..
                    } => {
                        let (x, y) = match delta {
                            glutin::event::MouseScrollDelta::LineDelta(dx, dy) => {
                                (dx as f64 * 10.0, dy as f64 * 10.0)
                            }
                            glutin::event::MouseScrollDelta::PixelDelta(delta) => delta.into(),
                        };
                        let modifiers = translate_modifiers(modifiers);
                        let _ = out_events.send(WindowEvent::Scroll(x, y, modifiers));
                    }
                    glutin::event::WindowEvent::KeyboardInput { input, .. } => {
                        let action = translate_action(input.state);
                        let key = translate_key(input.virtual_keycode);
                        let modifiers = translate_modifiers(input.modifiers);
                        key_states[key as usize] = action;
                        let _ = out_events.send(WindowEvent::Key(key, action, modifiers));
                    }
                    glutin::event::WindowEvent::ReceivedCharacter(c) => {
                        let _ = out_events.send(WindowEvent::Char(c));
                    }
                    _ => {}
                },
                Event::RedrawEventsCleared => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            };
        })
    }

    fn swap_buffers(&mut self) {
        let _ = self.window.swap_buffers();
    }

    fn size(&self) -> (u32, u32) {
        self.window.window().inner_size().into()
    }

    fn cursor_pos(&self) -> Option<(f64, f64)> {
        self.cursor_pos
    }

    fn scale_factor(&self) -> f64 {
        self.window.window().scale_factor() as f64
    }

    fn set_title(&mut self, title: &str) {
        self.window.window().set_title(title)
    }

    fn set_icon(&mut self, icon: impl GenericImage<Pixel = impl Pixel<Subpixel = u8>>) {
        let (width, height) = icon.dimensions();
        let mut rgba = Vec::with_capacity((width * height) as usize * 4);
        for (_, _, pixel) in icon.pixels() {
            rgba.extend_from_slice(&pixel.to_rgba().0);
        }
        let icon = glutin::window::Icon::from_rgba(rgba, width, height).unwrap();
        self.window.window().set_window_icon(Some(icon))
    }

    fn set_cursor_grab(&self, grab: bool) {
        let _ = self.window.window().set_cursor_grab(grab);
    }

    fn set_cursor_position(&self, x: f64, y: f64) {
        self.window
            .window()
            .set_cursor_position(glutin::dpi::PhysicalPosition::new(x, y))
            .unwrap();
    }

    fn hide_cursor(&self, hide: bool) {
        self.window.window().set_cursor_visible(!hide)
    }

    fn hide(&mut self) {
        self.window.window().set_visible(false)
    }

    fn show(&mut self) {
        self.window.window().set_visible(true)
    }

    fn get_mouse_button(&self, button: MouseButton) -> Action {
        self.button_states[button as usize]
    }
    fn get_key(&self, key: Key) -> Action {
        self.key_states[key as usize]
    }
}

fn translate_action(action: glutin::event::ElementState) -> Action {
    match action {
        glutin::event::ElementState::Pressed => Action::Press,
        glutin::event::ElementState::Released => Action::Release,
    }
}

fn translate_modifiers(modifiers: glutin::event::ModifiersState) -> Modifiers {
    let mut res = Modifiers::empty();
    if modifiers.shift() {
        res.insert(Modifiers::Shift)
    }
    if modifiers.ctrl() {
        res.insert(Modifiers::Control)
    }
    if modifiers.alt() {
        res.insert(Modifiers::Alt)
    }
    if modifiers.logo() {
        res.insert(Modifiers::Super)
    }

    res
}

fn translate_mouse_button(button: glutin::event::MouseButton) -> MouseButton {
    match button {
        glutin::event::MouseButton::Left => MouseButton::Button1,
        glutin::event::MouseButton::Right => MouseButton::Button2,
        glutin::event::MouseButton::Middle => MouseButton::Button3,
        glutin::event::MouseButton::Other(_) => MouseButton::Button4, // XXX: the default is not good.
    }
}

fn translate_key(button: Option<glutin::event::VirtualKeyCode>) -> Key {
    if let Some(button) = button {
        match button {
            glutin::event::VirtualKeyCode::Key1 => Key::Key1,
            glutin::event::VirtualKeyCode::Key2 => Key::Key2,
            glutin::event::VirtualKeyCode::Key3 => Key::Key3,
            glutin::event::VirtualKeyCode::Key4 => Key::Key4,
            glutin::event::VirtualKeyCode::Key5 => Key::Key5,
            glutin::event::VirtualKeyCode::Key6 => Key::Key6,
            glutin::event::VirtualKeyCode::Key7 => Key::Key7,
            glutin::event::VirtualKeyCode::Key8 => Key::Key8,
            glutin::event::VirtualKeyCode::Key9 => Key::Key9,
            glutin::event::VirtualKeyCode::Key0 => Key::Key0,
            glutin::event::VirtualKeyCode::A => Key::A,
            glutin::event::VirtualKeyCode::B => Key::B,
            glutin::event::VirtualKeyCode::C => Key::C,
            glutin::event::VirtualKeyCode::D => Key::D,
            glutin::event::VirtualKeyCode::E => Key::E,
            glutin::event::VirtualKeyCode::F => Key::F,
            glutin::event::VirtualKeyCode::G => Key::G,
            glutin::event::VirtualKeyCode::H => Key::H,
            glutin::event::VirtualKeyCode::I => Key::I,
            glutin::event::VirtualKeyCode::J => Key::J,
            glutin::event::VirtualKeyCode::K => Key::K,
            glutin::event::VirtualKeyCode::L => Key::L,
            glutin::event::VirtualKeyCode::M => Key::M,
            glutin::event::VirtualKeyCode::N => Key::N,
            glutin::event::VirtualKeyCode::O => Key::O,
            glutin::event::VirtualKeyCode::P => Key::P,
            glutin::event::VirtualKeyCode::Q => Key::Q,
            glutin::event::VirtualKeyCode::R => Key::R,
            glutin::event::VirtualKeyCode::S => Key::S,
            glutin::event::VirtualKeyCode::T => Key::T,
            glutin::event::VirtualKeyCode::U => Key::U,
            glutin::event::VirtualKeyCode::V => Key::V,
            glutin::event::VirtualKeyCode::W => Key::W,
            glutin::event::VirtualKeyCode::X => Key::X,
            glutin::event::VirtualKeyCode::Y => Key::Y,
            glutin::event::VirtualKeyCode::Z => Key::Z,
            glutin::event::VirtualKeyCode::Escape => Key::Escape,
            glutin::event::VirtualKeyCode::F1 => Key::F1,
            glutin::event::VirtualKeyCode::F2 => Key::F2,
            glutin::event::VirtualKeyCode::F3 => Key::F3,
            glutin::event::VirtualKeyCode::F4 => Key::F4,
            glutin::event::VirtualKeyCode::F5 => Key::F5,
            glutin::event::VirtualKeyCode::F6 => Key::F6,
            glutin::event::VirtualKeyCode::F7 => Key::F7,
            glutin::event::VirtualKeyCode::F8 => Key::F8,
            glutin::event::VirtualKeyCode::F9 => Key::F9,
            glutin::event::VirtualKeyCode::F10 => Key::F10,
            glutin::event::VirtualKeyCode::F11 => Key::F11,
            glutin::event::VirtualKeyCode::F12 => Key::F12,
            glutin::event::VirtualKeyCode::F13 => Key::F13,
            glutin::event::VirtualKeyCode::F14 => Key::F14,
            glutin::event::VirtualKeyCode::F15 => Key::F15,
            glutin::event::VirtualKeyCode::F16 => Key::F16,
            glutin::event::VirtualKeyCode::F17 => Key::F17,
            glutin::event::VirtualKeyCode::F18 => Key::F18,
            glutin::event::VirtualKeyCode::F19 => Key::F19,
            glutin::event::VirtualKeyCode::F20 => Key::F20,
            glutin::event::VirtualKeyCode::F21 => Key::F21,
            glutin::event::VirtualKeyCode::F22 => Key::F22,
            glutin::event::VirtualKeyCode::F23 => Key::F23,
            glutin::event::VirtualKeyCode::F24 => Key::F24,
            glutin::event::VirtualKeyCode::Snapshot => Key::Snapshot,
            glutin::event::VirtualKeyCode::Scroll => Key::Scroll,
            glutin::event::VirtualKeyCode::Pause => Key::Pause,
            glutin::event::VirtualKeyCode::Insert => Key::Insert,
            glutin::event::VirtualKeyCode::Home => Key::Home,
            glutin::event::VirtualKeyCode::Delete => Key::Delete,
            glutin::event::VirtualKeyCode::End => Key::End,
            glutin::event::VirtualKeyCode::PageDown => Key::PageDown,
            glutin::event::VirtualKeyCode::PageUp => Key::PageUp,
            glutin::event::VirtualKeyCode::Left => Key::Left,
            glutin::event::VirtualKeyCode::Up => Key::Up,
            glutin::event::VirtualKeyCode::Right => Key::Right,
            glutin::event::VirtualKeyCode::Down => Key::Down,
            glutin::event::VirtualKeyCode::Back => Key::Back,
            glutin::event::VirtualKeyCode::Return => Key::Return,
            glutin::event::VirtualKeyCode::Space => Key::Space,
            glutin::event::VirtualKeyCode::Compose => Key::Compose,
            glutin::event::VirtualKeyCode::Caret => Key::Caret,
            glutin::event::VirtualKeyCode::Numlock => Key::Numlock,
            glutin::event::VirtualKeyCode::Numpad0 => Key::Numpad0,
            glutin::event::VirtualKeyCode::Numpad1 => Key::Numpad1,
            glutin::event::VirtualKeyCode::Numpad2 => Key::Numpad2,
            glutin::event::VirtualKeyCode::Numpad3 => Key::Numpad3,
            glutin::event::VirtualKeyCode::Numpad4 => Key::Numpad4,
            glutin::event::VirtualKeyCode::Numpad5 => Key::Numpad5,
            glutin::event::VirtualKeyCode::Numpad6 => Key::Numpad6,
            glutin::event::VirtualKeyCode::Numpad7 => Key::Numpad7,
            glutin::event::VirtualKeyCode::Numpad8 => Key::Numpad8,
            glutin::event::VirtualKeyCode::Numpad9 => Key::Numpad9,
            glutin::event::VirtualKeyCode::AbntC1 => Key::AbntC1,
            glutin::event::VirtualKeyCode::AbntC2 => Key::AbntC2,
            glutin::event::VirtualKeyCode::NumpadAdd => Key::Add,
            glutin::event::VirtualKeyCode::Apostrophe => Key::Apostrophe,
            glutin::event::VirtualKeyCode::Apps => Key::Apps,
            glutin::event::VirtualKeyCode::At => Key::At,
            glutin::event::VirtualKeyCode::Ax => Key::Ax,
            glutin::event::VirtualKeyCode::Backslash => Key::Backslash,
            glutin::event::VirtualKeyCode::Calculator => Key::Calculator,
            glutin::event::VirtualKeyCode::Capital => Key::Capital,
            glutin::event::VirtualKeyCode::Colon => Key::Colon,
            glutin::event::VirtualKeyCode::Comma => Key::Comma,
            glutin::event::VirtualKeyCode::Convert => Key::Convert,
            glutin::event::VirtualKeyCode::NumpadDecimal => Key::Decimal,
            glutin::event::VirtualKeyCode::NumpadDivide => Key::Divide,
            glutin::event::VirtualKeyCode::Asterisk => Key::Multiply,
            glutin::event::VirtualKeyCode::Plus => Key::Add,
            glutin::event::VirtualKeyCode::Equals => Key::Equals,
            glutin::event::VirtualKeyCode::Grave => Key::Grave,
            glutin::event::VirtualKeyCode::Kana => Key::Kana,
            glutin::event::VirtualKeyCode::Kanji => Key::Kanji,
            glutin::event::VirtualKeyCode::LAlt => Key::LAlt,
            glutin::event::VirtualKeyCode::LBracket => Key::LBracket,
            glutin::event::VirtualKeyCode::LControl => Key::LControl,
            glutin::event::VirtualKeyCode::LShift => Key::LShift,
            glutin::event::VirtualKeyCode::LWin => Key::LWin,
            glutin::event::VirtualKeyCode::Mail => Key::Mail,
            glutin::event::VirtualKeyCode::MediaSelect => Key::MediaSelect,
            glutin::event::VirtualKeyCode::MediaStop => Key::MediaStop,
            glutin::event::VirtualKeyCode::Minus => Key::Minus,
            glutin::event::VirtualKeyCode::NumpadMultiply => Key::Multiply,
            glutin::event::VirtualKeyCode::Mute => Key::Mute,
            glutin::event::VirtualKeyCode::MyComputer => Key::MyComputer,
            glutin::event::VirtualKeyCode::NavigateForward => Key::NavigateForward,
            glutin::event::VirtualKeyCode::NavigateBackward => Key::NavigateBackward,
            glutin::event::VirtualKeyCode::NextTrack => Key::NextTrack,
            glutin::event::VirtualKeyCode::NoConvert => Key::NoConvert,
            glutin::event::VirtualKeyCode::NumpadComma => Key::NumpadComma,
            glutin::event::VirtualKeyCode::NumpadEnter => Key::NumpadEnter,
            glutin::event::VirtualKeyCode::NumpadEquals => Key::NumpadEquals,
            glutin::event::VirtualKeyCode::OEM102 => Key::OEM102,
            glutin::event::VirtualKeyCode::Period => Key::Period,
            glutin::event::VirtualKeyCode::PlayPause => Key::PlayPause,
            glutin::event::VirtualKeyCode::Power => Key::Power,
            glutin::event::VirtualKeyCode::PrevTrack => Key::PrevTrack,
            glutin::event::VirtualKeyCode::RAlt => Key::RAlt,
            glutin::event::VirtualKeyCode::RBracket => Key::RBracket,
            glutin::event::VirtualKeyCode::RControl => Key::RControl,
            glutin::event::VirtualKeyCode::RShift => Key::RShift,
            glutin::event::VirtualKeyCode::RWin => Key::RWin,
            glutin::event::VirtualKeyCode::Semicolon => Key::Semicolon,
            glutin::event::VirtualKeyCode::Slash => Key::Slash,
            glutin::event::VirtualKeyCode::Sleep => Key::Sleep,
            glutin::event::VirtualKeyCode::Stop => Key::Stop,
            glutin::event::VirtualKeyCode::NumpadSubtract => Key::Subtract,
            glutin::event::VirtualKeyCode::Sysrq => Key::Sysrq,
            glutin::event::VirtualKeyCode::Tab => Key::Tab,
            glutin::event::VirtualKeyCode::Underline => Key::Underline,
            glutin::event::VirtualKeyCode::Unlabeled => Key::Unlabeled,
            glutin::event::VirtualKeyCode::VolumeDown => Key::VolumeDown,
            glutin::event::VirtualKeyCode::VolumeUp => Key::VolumeUp,
            glutin::event::VirtualKeyCode::Wake => Key::Wake,
            glutin::event::VirtualKeyCode::WebBack => Key::WebBack,
            glutin::event::VirtualKeyCode::WebFavorites => Key::WebFavorites,
            glutin::event::VirtualKeyCode::WebForward => Key::WebForward,
            glutin::event::VirtualKeyCode::WebHome => Key::WebHome,
            glutin::event::VirtualKeyCode::WebRefresh => Key::WebRefresh,
            glutin::event::VirtualKeyCode::WebSearch => Key::WebSearch,
            glutin::event::VirtualKeyCode::WebStop => Key::WebStop,
            glutin::event::VirtualKeyCode::Yen => Key::Yen,
            glutin::event::VirtualKeyCode::Copy => Key::Copy,
            glutin::event::VirtualKeyCode::Paste => Key::Paste,
            glutin::event::VirtualKeyCode::Cut => Key::Cut,
        }
    } else {
        Key::Unknown
    }
}
