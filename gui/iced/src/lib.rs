use glow::HasContext;
use iced_native::{keyboard, mouse, window, Debug, Event, Program};
use kiss3d::{context::Context, event::WindowEvent, window::UiContext};
use nalgebra::Vector2;

mod backend;
mod program;
mod quad;
mod text;
mod triangle;

pub mod settings;
pub mod widget;

pub use backend::Backend;
pub use iced_graphics::Viewport;
pub use settings::Settings;

pub(crate) use iced_graphics::{Point, Size, Transformation};

pub type Renderer = iced_graphics::Renderer<Backend>;

pub struct IcedContext<P>
where
    P: Program<Renderer = Renderer> + 'static,
{
    // pending_program: Option<P>,
    state: iced_native::program::State<P>,
    debug: Debug,
    renderer: Renderer,
    viewport: Viewport,
    cursor_position: (f64, f64),
    // TODO
}

impl<P> IcedContext<P>
where
    P: Program<Renderer = Renderer> + 'static,
{
    pub fn program(&self) -> &P {
        self.state.program()
    }
}

impl<P> UiContext for IcedContext<P>
where
    P: Program<Renderer = Renderer> + 'static,
{
    type Init = P;

    fn new(width: u32, height: u32, ui_init: Self::Init) -> Self {
        let mut debug = Debug::new();
        let mut renderer = Renderer::new(Backend::new(Settings::default()));
        let viewport = Viewport::with_physical_size(Size::new(width, height), 1.0);
        let state = iced_native::program::State::new(
            ui_init,
            viewport.logical_size(),
            Point::new(-1.0, -1.0),
            &mut renderer,
            &mut debug,
        );
        Self {
            // pending_program: None,
            state,
            debug,
            renderer,
            viewport,
            cursor_position: (0.0, 0.0),
        }
    }

    fn handle_event(&mut self, event: &WindowEvent, size: Vector2<u32>, hidpi_factor: f64) -> bool {
        // if let Some(viewport) = &self.cached_viewport {
        if self.viewport.physical_width() != size.x
            || self.viewport.physical_height() != size.y
            || self.viewport.scale_factor() != hidpi_factor
        {
            self.viewport = Viewport::with_physical_size(Size::new(size.x, size.y), hidpi_factor)
        }
        // }
        // let viewport = self
        //     .cached_viewport
        //     .get_or_insert_with(|| Viewport::with_physical_size(Size::new(size.x, size.y), hidpi));

        // if let Some(program) = self.pending_program.take() {
        //     self.state = Some(program::State::new(
        //         program,
        //         self.viewport.logical_size(),
        //         // conversion::cursor_position(cursor_position, viewport.scale_factor()),
        //         Point::new(-1.0, -1.0), // TODO
        //         &mut self.renderer,
        //         &mut self.debug,
        //     ))
        // }

        match event {
            WindowEvent::CursorPos(x, y, mods) => {
                self.cursor_position = (x / hidpi_factor, y / hidpi_factor);
            }
            _ => {}
        }

        if let Some(event) = window_event_to_iced_event(*event, size, hidpi_factor) {
            self.state.queue_event(event);
        }
        // TODO
        // todo!()
        false
    }

    fn render(&mut self, width: u32, height: u32, hidpi_factor: f64) {
        // let viewport = match &self.cached_viewport {
        //     Some(x) => x,
        //     None => return,
        // };
        if self.viewport.physical_width() != width
            || self.viewport.physical_height() != height
            || self.viewport.scale_factor() != hidpi_factor
        {
            self.viewport = Viewport::with_physical_size(Size::new(width, height), hidpi_factor)
        }

        // We update iced
        let _ = self.state.update(
            self.viewport.logical_size(),
            Point::new(self.cursor_position.0 as f32, self.cursor_position.1 as f32),
            None,
            &mut self.renderer,
            &mut self.debug,
        );

        // Then draw iced on top
        let ctxt = Context::get();
        let gl = ctxt.get_glow();

        // Enable auto-conversion from/to sRGB
        unsafe { gl.enable(glow::FRAMEBUFFER_SRGB) };

        // Enable alpha blending
        unsafe { gl.enable(glow::BLEND) };
        unsafe { gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA) };

        // Disable multisampling by default
        unsafe { gl.disable(glow::MULTISAMPLE) };

        let mouse_interaction = self.renderer.backend_mut().draw(
            gl,
            &self.viewport,
            self.state.primitive(),
            &self.debug.overlay(),
        );

        unsafe { gl.enable(glow::MULTISAMPLE) };
        unsafe { gl.disable(glow::BLEND) };
        unsafe { gl.disable(glow::FRAMEBUFFER_SRGB) };

        // // And update the mouse cursor
        // window.set_cursor_icon(
        //     iced_winit::conversion::mouse_interaction(
        //         mouse_interaction,
        //     ),
        // );
    }
}

fn window_event_to_iced_event(event: WindowEvent, size: Vector2<u32>, hidpi: f64) -> Option<Event> {
    match event {
        WindowEvent::FramebufferSize(w, h) => Some(Event::Window(window::Event::Resized {
            width: (w as f64 / hidpi) as u32,
            height: (h as f64 / hidpi) as u32,
        })),
        // WindowEvent::Size(w, h) => {}
        // WindowEvent::CursorEnter(_) => {}
        WindowEvent::CursorPos(x, y, mods) => Some(Event::Mouse(mouse::Event::CursorMoved {
            x: (x / hidpi) as f32,
            y: (y / hidpi) as f32,
        })),
        WindowEvent::MouseButton(btn, act, mods) => {
            let button = match btn {
                kiss3d::event::MouseButton::Button1 => mouse::Button::Left,
                kiss3d::event::MouseButton::Button2 => mouse::Button::Right,
                kiss3d::event::MouseButton::Button3 => mouse::Button::Middle,
                kiss3d::event::MouseButton::Button4 => mouse::Button::Other(4),
                kiss3d::event::MouseButton::Button5 => mouse::Button::Other(5),
                kiss3d::event::MouseButton::Button6 => mouse::Button::Other(6),
                kiss3d::event::MouseButton::Button7 => mouse::Button::Other(7),
                kiss3d::event::MouseButton::Button8 => mouse::Button::Other(8),
            };
            Some(Event::Mouse(match act {
                kiss3d::event::Action::Press => mouse::Event::ButtonPressed(button),
                kiss3d::event::Action::Release => mouse::Event::ButtonReleased(button),
            }))
        }
        WindowEvent::Scroll(dx, dy, mods) => {
            // TODO: ScrollDelta::Lines?
            Some(Event::Mouse(mouse::Event::WheelScrolled {
                delta: mouse::ScrollDelta::Pixels {
                    x: dx as f32,
                    y: dy as f32,
                },
            }))
        }
        WindowEvent::Key(key, act, mods) => {
            let key_code = conv_key_code(key)?;
            let modifiers = conv_modifiers(mods);
            Some(Event::Keyboard(match act {
                kiss3d::event::Action::Press => keyboard::Event::KeyPressed {
                    key_code,
                    modifiers,
                },
                kiss3d::event::Action::Release => keyboard::Event::KeyReleased {
                    key_code,
                    modifiers,
                },
            }))
        }
        WindowEvent::Char(c) => Some(Event::Keyboard(keyboard::Event::CharacterReceived(c))),
        // WindowEvent::CharModifiers(_, _) => {}
        // WindowEvent::Pos(_, _) => {}
        // WindowEvent::Close => {}
        // WindowEvent::Refresh => {}
        // WindowEvent::Focus(_) => {}
        // WindowEvent::Iconify(_) => {}
        // WindowEvent::Touch(_, _, _, _, _) => {}
        _ => None,
    }
}

fn conv_modifiers(mods: kiss3d::event::Modifiers) -> iced_native::keyboard::ModifiersState {
    iced_native::keyboard::ModifiersState {
        shift: mods.contains(kiss3d::event::Modifiers::Shift),
        control: mods.contains(kiss3d::event::Modifiers::Control),
        alt: mods.contains(kiss3d::event::Modifiers::Alt),
        logo: mods.contains(kiss3d::event::Modifiers::Super),
    }
}

fn conv_key_code(key: kiss3d::event::Key) -> Option<iced_native::keyboard::KeyCode> {
    Some(match key {
        kiss3d::event::Key::Key1 => iced_native::keyboard::KeyCode::Key1,
        kiss3d::event::Key::Key2 => iced_native::keyboard::KeyCode::Key2,
        kiss3d::event::Key::Key3 => iced_native::keyboard::KeyCode::Key3,
        kiss3d::event::Key::Key4 => iced_native::keyboard::KeyCode::Key4,
        kiss3d::event::Key::Key5 => iced_native::keyboard::KeyCode::Key5,
        kiss3d::event::Key::Key6 => iced_native::keyboard::KeyCode::Key6,
        kiss3d::event::Key::Key7 => iced_native::keyboard::KeyCode::Key7,
        kiss3d::event::Key::Key8 => iced_native::keyboard::KeyCode::Key8,
        kiss3d::event::Key::Key9 => iced_native::keyboard::KeyCode::Key9,
        kiss3d::event::Key::Key0 => iced_native::keyboard::KeyCode::Key0,
        kiss3d::event::Key::A => iced_native::keyboard::KeyCode::A,
        kiss3d::event::Key::B => iced_native::keyboard::KeyCode::B,
        kiss3d::event::Key::C => iced_native::keyboard::KeyCode::C,
        kiss3d::event::Key::D => iced_native::keyboard::KeyCode::D,
        kiss3d::event::Key::E => iced_native::keyboard::KeyCode::E,
        kiss3d::event::Key::F => iced_native::keyboard::KeyCode::F,
        kiss3d::event::Key::G => iced_native::keyboard::KeyCode::G,
        kiss3d::event::Key::H => iced_native::keyboard::KeyCode::H,
        kiss3d::event::Key::I => iced_native::keyboard::KeyCode::I,
        kiss3d::event::Key::J => iced_native::keyboard::KeyCode::J,
        kiss3d::event::Key::K => iced_native::keyboard::KeyCode::K,
        kiss3d::event::Key::L => iced_native::keyboard::KeyCode::L,
        kiss3d::event::Key::M => iced_native::keyboard::KeyCode::M,
        kiss3d::event::Key::N => iced_native::keyboard::KeyCode::N,
        kiss3d::event::Key::O => iced_native::keyboard::KeyCode::O,
        kiss3d::event::Key::P => iced_native::keyboard::KeyCode::P,
        kiss3d::event::Key::Q => iced_native::keyboard::KeyCode::Q,
        kiss3d::event::Key::R => iced_native::keyboard::KeyCode::R,
        kiss3d::event::Key::S => iced_native::keyboard::KeyCode::S,
        kiss3d::event::Key::T => iced_native::keyboard::KeyCode::T,
        kiss3d::event::Key::U => iced_native::keyboard::KeyCode::U,
        kiss3d::event::Key::V => iced_native::keyboard::KeyCode::V,
        kiss3d::event::Key::W => iced_native::keyboard::KeyCode::W,
        kiss3d::event::Key::X => iced_native::keyboard::KeyCode::X,
        kiss3d::event::Key::Y => iced_native::keyboard::KeyCode::Y,
        kiss3d::event::Key::Z => iced_native::keyboard::KeyCode::Z,
        kiss3d::event::Key::Escape => iced_native::keyboard::KeyCode::Escape,
        kiss3d::event::Key::F1 => iced_native::keyboard::KeyCode::F1,
        kiss3d::event::Key::F2 => iced_native::keyboard::KeyCode::F2,
        kiss3d::event::Key::F3 => iced_native::keyboard::KeyCode::F3,
        kiss3d::event::Key::F4 => iced_native::keyboard::KeyCode::F4,
        kiss3d::event::Key::F5 => iced_native::keyboard::KeyCode::F5,
        kiss3d::event::Key::F6 => iced_native::keyboard::KeyCode::F6,
        kiss3d::event::Key::F7 => iced_native::keyboard::KeyCode::F7,
        kiss3d::event::Key::F8 => iced_native::keyboard::KeyCode::F8,
        kiss3d::event::Key::F9 => iced_native::keyboard::KeyCode::F9,
        kiss3d::event::Key::F10 => iced_native::keyboard::KeyCode::F10,
        kiss3d::event::Key::F11 => iced_native::keyboard::KeyCode::F11,
        kiss3d::event::Key::F12 => iced_native::keyboard::KeyCode::F12,
        kiss3d::event::Key::F13 => iced_native::keyboard::KeyCode::F13,
        kiss3d::event::Key::F14 => iced_native::keyboard::KeyCode::F14,
        kiss3d::event::Key::F15 => iced_native::keyboard::KeyCode::F15,
        kiss3d::event::Key::F16 => iced_native::keyboard::KeyCode::F16,
        kiss3d::event::Key::F17 => iced_native::keyboard::KeyCode::F17,
        kiss3d::event::Key::F18 => iced_native::keyboard::KeyCode::F18,
        kiss3d::event::Key::F19 => iced_native::keyboard::KeyCode::F19,
        kiss3d::event::Key::F20 => iced_native::keyboard::KeyCode::F20,
        kiss3d::event::Key::F21 => iced_native::keyboard::KeyCode::F21,
        kiss3d::event::Key::F22 => iced_native::keyboard::KeyCode::F22,
        kiss3d::event::Key::F23 => iced_native::keyboard::KeyCode::F23,
        kiss3d::event::Key::F24 => iced_native::keyboard::KeyCode::F24,
        kiss3d::event::Key::Snapshot => iced_native::keyboard::KeyCode::Snapshot,
        kiss3d::event::Key::Scroll => iced_native::keyboard::KeyCode::Scroll,
        kiss3d::event::Key::Pause => iced_native::keyboard::KeyCode::Pause,
        kiss3d::event::Key::Insert => iced_native::keyboard::KeyCode::Insert,
        kiss3d::event::Key::Home => iced_native::keyboard::KeyCode::Home,
        kiss3d::event::Key::Delete => iced_native::keyboard::KeyCode::Delete,
        kiss3d::event::Key::End => iced_native::keyboard::KeyCode::End,
        kiss3d::event::Key::PageDown => iced_native::keyboard::KeyCode::PageDown,
        kiss3d::event::Key::PageUp => iced_native::keyboard::KeyCode::PageUp,
        kiss3d::event::Key::Left => iced_native::keyboard::KeyCode::Left,
        kiss3d::event::Key::Up => iced_native::keyboard::KeyCode::Up,
        kiss3d::event::Key::Right => iced_native::keyboard::KeyCode::Right,
        kiss3d::event::Key::Down => iced_native::keyboard::KeyCode::Down,
        kiss3d::event::Key::Back => iced_native::keyboard::KeyCode::Backspace,
        kiss3d::event::Key::Return => iced_native::keyboard::KeyCode::Enter,
        kiss3d::event::Key::Space => iced_native::keyboard::KeyCode::Space,
        kiss3d::event::Key::Compose => iced_native::keyboard::KeyCode::Compose,
        kiss3d::event::Key::Caret => iced_native::keyboard::KeyCode::Caret,
        kiss3d::event::Key::Numlock => iced_native::keyboard::KeyCode::Numlock,
        kiss3d::event::Key::Numpad0 => iced_native::keyboard::KeyCode::Numpad0,
        kiss3d::event::Key::Numpad1 => iced_native::keyboard::KeyCode::Numpad1,
        kiss3d::event::Key::Numpad2 => iced_native::keyboard::KeyCode::Numpad2,
        kiss3d::event::Key::Numpad3 => iced_native::keyboard::KeyCode::Numpad3,
        kiss3d::event::Key::Numpad4 => iced_native::keyboard::KeyCode::Numpad4,
        kiss3d::event::Key::Numpad5 => iced_native::keyboard::KeyCode::Numpad5,
        kiss3d::event::Key::Numpad6 => iced_native::keyboard::KeyCode::Numpad6,
        kiss3d::event::Key::Numpad7 => iced_native::keyboard::KeyCode::Numpad7,
        kiss3d::event::Key::Numpad8 => iced_native::keyboard::KeyCode::Numpad8,
        kiss3d::event::Key::Numpad9 => iced_native::keyboard::KeyCode::Numpad9,
        kiss3d::event::Key::AbntC1 => iced_native::keyboard::KeyCode::AbntC1,
        kiss3d::event::Key::AbntC2 => iced_native::keyboard::KeyCode::AbntC2,
        kiss3d::event::Key::Add => iced_native::keyboard::KeyCode::Add,
        kiss3d::event::Key::Apostrophe => iced_native::keyboard::KeyCode::Apostrophe,
        kiss3d::event::Key::Apps => iced_native::keyboard::KeyCode::Apps,
        kiss3d::event::Key::At => iced_native::keyboard::KeyCode::At,
        kiss3d::event::Key::Ax => iced_native::keyboard::KeyCode::Ax,
        kiss3d::event::Key::Backslash => iced_native::keyboard::KeyCode::Backslash,
        kiss3d::event::Key::Calculator => iced_native::keyboard::KeyCode::Calculator,
        kiss3d::event::Key::Capital => iced_native::keyboard::KeyCode::Capital,
        kiss3d::event::Key::Colon => iced_native::keyboard::KeyCode::Colon,
        kiss3d::event::Key::Comma => iced_native::keyboard::KeyCode::Comma,
        kiss3d::event::Key::Convert => iced_native::keyboard::KeyCode::Convert,
        kiss3d::event::Key::Decimal => iced_native::keyboard::KeyCode::Decimal,
        kiss3d::event::Key::Divide => iced_native::keyboard::KeyCode::Divide,
        kiss3d::event::Key::Equals => iced_native::keyboard::KeyCode::Equals,
        kiss3d::event::Key::Grave => iced_native::keyboard::KeyCode::Grave,
        kiss3d::event::Key::Kana => iced_native::keyboard::KeyCode::Kana,
        kiss3d::event::Key::Kanji => iced_native::keyboard::KeyCode::Kanji,
        kiss3d::event::Key::LAlt => iced_native::keyboard::KeyCode::LAlt,
        kiss3d::event::Key::LBracket => iced_native::keyboard::KeyCode::LBracket,
        kiss3d::event::Key::LControl => iced_native::keyboard::KeyCode::LControl,
        kiss3d::event::Key::LShift => iced_native::keyboard::KeyCode::LShift,
        kiss3d::event::Key::LWin => iced_native::keyboard::KeyCode::LWin,
        kiss3d::event::Key::Mail => iced_native::keyboard::KeyCode::Mail,
        kiss3d::event::Key::MediaSelect => iced_native::keyboard::KeyCode::MediaSelect,
        kiss3d::event::Key::MediaStop => iced_native::keyboard::KeyCode::MediaStop,
        kiss3d::event::Key::Minus => iced_native::keyboard::KeyCode::Minus,
        kiss3d::event::Key::Multiply => iced_native::keyboard::KeyCode::Multiply,
        kiss3d::event::Key::Mute => iced_native::keyboard::KeyCode::Mute,
        kiss3d::event::Key::MyComputer => iced_native::keyboard::KeyCode::MyComputer,
        kiss3d::event::Key::NavigateForward => iced_native::keyboard::KeyCode::NavigateForward,
        kiss3d::event::Key::NavigateBackward => iced_native::keyboard::KeyCode::NavigateBackward,
        kiss3d::event::Key::NextTrack => iced_native::keyboard::KeyCode::NextTrack,
        kiss3d::event::Key::NoConvert => iced_native::keyboard::KeyCode::NoConvert,
        kiss3d::event::Key::NumpadComma => iced_native::keyboard::KeyCode::NumpadComma,
        kiss3d::event::Key::NumpadEnter => iced_native::keyboard::KeyCode::NumpadEnter,
        kiss3d::event::Key::NumpadEquals => iced_native::keyboard::KeyCode::NumpadEquals,
        kiss3d::event::Key::OEM102 => iced_native::keyboard::KeyCode::OEM102,
        kiss3d::event::Key::Period => iced_native::keyboard::KeyCode::Period,
        kiss3d::event::Key::PlayPause => iced_native::keyboard::KeyCode::PlayPause,
        kiss3d::event::Key::Power => iced_native::keyboard::KeyCode::Power,
        kiss3d::event::Key::PrevTrack => iced_native::keyboard::KeyCode::PrevTrack,
        kiss3d::event::Key::RAlt => iced_native::keyboard::KeyCode::RAlt,
        kiss3d::event::Key::RBracket => iced_native::keyboard::KeyCode::RBracket,
        kiss3d::event::Key::RControl => iced_native::keyboard::KeyCode::RControl,
        kiss3d::event::Key::RShift => iced_native::keyboard::KeyCode::RShift,
        kiss3d::event::Key::RWin => iced_native::keyboard::KeyCode::RWin,
        kiss3d::event::Key::Semicolon => iced_native::keyboard::KeyCode::Semicolon,
        kiss3d::event::Key::Slash => iced_native::keyboard::KeyCode::Slash,
        kiss3d::event::Key::Sleep => iced_native::keyboard::KeyCode::Sleep,
        kiss3d::event::Key::Stop => iced_native::keyboard::KeyCode::Stop,
        kiss3d::event::Key::Subtract => iced_native::keyboard::KeyCode::Subtract,
        kiss3d::event::Key::Sysrq => iced_native::keyboard::KeyCode::Sysrq,
        kiss3d::event::Key::Tab => iced_native::keyboard::KeyCode::Tab,
        kiss3d::event::Key::Underline => iced_native::keyboard::KeyCode::Underline,
        kiss3d::event::Key::Unlabeled => iced_native::keyboard::KeyCode::Unlabeled,
        kiss3d::event::Key::VolumeDown => iced_native::keyboard::KeyCode::VolumeDown,
        kiss3d::event::Key::VolumeUp => iced_native::keyboard::KeyCode::VolumeUp,
        kiss3d::event::Key::Wake => iced_native::keyboard::KeyCode::Wake,
        kiss3d::event::Key::WebBack => iced_native::keyboard::KeyCode::WebBack,
        kiss3d::event::Key::WebFavorites => iced_native::keyboard::KeyCode::WebFavorites,
        kiss3d::event::Key::WebForward => iced_native::keyboard::KeyCode::WebForward,
        kiss3d::event::Key::WebHome => iced_native::keyboard::KeyCode::WebHome,
        kiss3d::event::Key::WebRefresh => iced_native::keyboard::KeyCode::WebRefresh,
        kiss3d::event::Key::WebSearch => iced_native::keyboard::KeyCode::WebSearch,
        kiss3d::event::Key::WebStop => iced_native::keyboard::KeyCode::WebStop,
        kiss3d::event::Key::Yen => iced_native::keyboard::KeyCode::Yen,
        kiss3d::event::Key::Copy => iced_native::keyboard::KeyCode::Copy,
        kiss3d::event::Key::Paste => iced_native::keyboard::KeyCode::Paste,
        kiss3d::event::Key::Cut => iced_native::keyboard::KeyCode::Cut,
        kiss3d::event::Key::Unknown => return None,
    })
}
