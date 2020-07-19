use conrod::{
    event::Input,
    input::{Button, Key as CKey, Motion, MouseButton},
};
use conrod_renderer::ConrodRenderer;
use kiss3d::{
    event::{Action, Key, WindowEvent},
    resource::{Texture, TextureManager},
    window::UiContext,
};
use nalgebra::Vector2;
use std::{collections::HashMap, rc::Rc};

pub use conrod_core as conrod;

mod conrod_renderer;

pub struct ConrodContext {
    renderer: ConrodRenderer,
    textures: conrod::image::Map<(Rc<Texture>, (u32, u32))>,
    texture_ids: HashMap<String, conrod::image::Id>,
}

impl ConrodContext {
    pub fn conrod_ui(&self) -> &conrod::Ui {
        self.renderer.ui()
    }

    pub fn conrod_ui_mut(&mut self) -> &mut conrod::Ui {
        self.renderer.ui_mut()
    }

    /// Attributes a conrod ID to the given texture and returns it if it exists.
    pub fn conrod_texture_id(&mut self, name: &str) -> Option<conrod::image::Id> {
        let tex = TextureManager::get_global_manager(|tm| tm.get_with_size(name))?;
        let textures = &mut self.textures;
        Some(
            *self
                .texture_ids
                .entry(name.to_string())
                .or_insert_with(|| textures.insert(tex)),
        )
    }

    /// Returns `true` if the keyboard is currently interacting with a Conrod widget.
    pub fn is_conrod_ui_capturing_keyboard(&self) -> bool {
        let ui = self.renderer.ui();
        let state = &ui.global_input().current;
        let window_id = Some(ui.window);

        state.widget_capturing_keyboard.is_some() && state.widget_capturing_keyboard != window_id
    }
}

impl UiContext for ConrodContext {
    type Init = ();

    fn new(width: u32, height: u32, _ui_init: Self::Init) -> Self {
        Self {
            renderer: ConrodRenderer::new(width as f64, height as f64),
            textures: conrod::image::Map::new(),
            texture_ids: HashMap::new(),
        }
    }

    fn handle_event(&mut self, event: &WindowEvent, size: Vector2<u32>, hidpi: f64) -> bool {
        let conrod_ui = self.renderer.ui_mut();
        if let Some(input) = window_event_to_conrod_input(*event, size, hidpi) {
            conrod_ui.handle_event(input);
        }

        let state = &conrod_ui.global_input().current;
        let window_id = Some(conrod_ui.window);

        if event.is_keyboard_event()
            && state.widget_capturing_keyboard.is_some()
            && state.widget_capturing_keyboard != window_id
        {
            return true;
        }

        if event.is_mouse_event()
            && state.widget_capturing_mouse.is_some()
            && state.widget_capturing_mouse != window_id
        {
            return true;
        }

        false
    }

    fn render(&mut self, width: u32, height: u32, hidpi_factor: f64) {
        self.renderer.render(
            width as f32,
            height as f32,
            hidpi_factor as f32,
            &self.textures,
        )
    }
}

fn window_event_to_conrod_input(
    event: WindowEvent,
    size: Vector2<u32>,
    hidpi: f64,
) -> Option<Input> {
    let transform_coords = |x: f64, y: f64| {
        (
            (x - size.x as f64 / 2.0) / hidpi,
            -(y - size.y as f64 / 2.0) / hidpi,
        )
    };

    match event {
        WindowEvent::FramebufferSize(w, h) => {
            Some(Input::Resize(w as f64 / hidpi, h as f64 / hidpi))
        }
        WindowEvent::Focus(focus) => Some(Input::Focus(focus)),
        WindowEvent::CursorPos(x, y, _) => {
            let (x, y) = transform_coords(x, y);
            Some(Input::Motion(Motion::MouseCursor { x, y }))
        }
        WindowEvent::Scroll(x, y, _) => Some(Input::Motion(Motion::Scroll { x, y: -y })),
        WindowEvent::MouseButton(button, action, _) => {
            let button = match button {
                kiss3d::event::MouseButton::Button1 => MouseButton::Left,
                kiss3d::event::MouseButton::Button2 => MouseButton::Right,
                kiss3d::event::MouseButton::Button3 => MouseButton::Middle,
                kiss3d::event::MouseButton::Button4 => MouseButton::X1,
                kiss3d::event::MouseButton::Button5 => MouseButton::X2,
                kiss3d::event::MouseButton::Button6 => MouseButton::Button6,
                kiss3d::event::MouseButton::Button7 => MouseButton::Button7,
                kiss3d::event::MouseButton::Button8 => MouseButton::Button8,
            };

            match action {
                Action::Press => Some(Input::Press(Button::Mouse(button))),
                Action::Release => Some(Input::Release(Button::Mouse(button))),
            }
        }
        WindowEvent::Key(key, action, _) => {
            let key = match key {
                Key::Key1 => CKey::D1,
                Key::Key2 => CKey::D2,
                Key::Key3 => CKey::D3,
                Key::Key4 => CKey::D4,
                Key::Key5 => CKey::D5,
                Key::Key6 => CKey::D6,
                Key::Key7 => CKey::D7,
                Key::Key8 => CKey::D8,
                Key::Key9 => CKey::D9,
                Key::Key0 => CKey::D0,
                Key::A => CKey::A,
                Key::B => CKey::B,
                Key::C => CKey::C,
                Key::D => CKey::D,
                Key::E => CKey::E,
                Key::F => CKey::F,
                Key::G => CKey::G,
                Key::H => CKey::H,
                Key::I => CKey::I,
                Key::J => CKey::J,
                Key::K => CKey::K,
                Key::L => CKey::L,
                Key::M => CKey::M,
                Key::N => CKey::N,
                Key::O => CKey::O,
                Key::P => CKey::P,
                Key::Q => CKey::Q,
                Key::R => CKey::R,
                Key::S => CKey::S,
                Key::T => CKey::T,
                Key::U => CKey::U,
                Key::V => CKey::V,
                Key::W => CKey::W,
                Key::X => CKey::X,
                Key::Y => CKey::Y,
                Key::Z => CKey::Z,
                Key::Escape => CKey::Escape,
                Key::F1 => CKey::F1,
                Key::F2 => CKey::F2,
                Key::F3 => CKey::F3,
                Key::F4 => CKey::F4,
                Key::F5 => CKey::F5,
                Key::F6 => CKey::F6,
                Key::F7 => CKey::F7,
                Key::F8 => CKey::F8,
                Key::F9 => CKey::F9,
                Key::F10 => CKey::F10,
                Key::F11 => CKey::F11,
                Key::F12 => CKey::F12,
                Key::F13 => CKey::F13,
                Key::F14 => CKey::F14,
                Key::F15 => CKey::F15,
                Key::F16 => CKey::F16,
                Key::F17 => CKey::F17,
                Key::F18 => CKey::F18,
                Key::F19 => CKey::F19,
                Key::F20 => CKey::F20,
                Key::F21 => CKey::F21,
                Key::F22 => CKey::F22,
                Key::F23 => CKey::F23,
                Key::F24 => CKey::F24,
                Key::Pause => CKey::Pause,
                Key::Insert => CKey::Insert,
                Key::Home => CKey::Home,
                Key::Delete => CKey::Delete,
                Key::End => CKey::End,
                Key::PageDown => CKey::PageDown,
                Key::PageUp => CKey::PageUp,
                Key::Left => CKey::Left,
                Key::Up => CKey::Up,
                Key::Right => CKey::Right,
                Key::Down => CKey::Down,
                Key::Return => CKey::Return,
                Key::Space => CKey::Space,
                Key::Caret => CKey::Caret,
                Key::Numpad0 => CKey::NumPad0,
                Key::Numpad1 => CKey::NumPad1,
                Key::Numpad2 => CKey::NumPad2,
                Key::Numpad3 => CKey::NumPad3,
                Key::Numpad4 => CKey::NumPad4,
                Key::Numpad5 => CKey::NumPad5,
                Key::Numpad6 => CKey::NumPad6,
                Key::Numpad7 => CKey::NumPad7,
                Key::Numpad8 => CKey::NumPad8,
                Key::Numpad9 => CKey::NumPad9,
                Key::Add => CKey::Plus,
                Key::At => CKey::At,
                Key::Backslash => CKey::Backslash,
                Key::Calculator => CKey::Calculator,
                Key::Colon => CKey::Colon,
                Key::Comma => CKey::Comma,
                Key::Equals => CKey::Equals,
                Key::LBracket => CKey::LeftBracket,
                Key::LControl => CKey::LCtrl,
                Key::LShift => CKey::LShift,
                Key::Mail => CKey::Mail,
                Key::MediaSelect => CKey::MediaSelect,
                Key::Minus => CKey::Minus,
                Key::Mute => CKey::Mute,
                Key::NumpadComma => CKey::NumPadComma,
                Key::NumpadEnter => CKey::NumPadEnter,
                Key::NumpadEquals => CKey::NumPadEquals,
                Key::Period => CKey::Period,
                Key::Power => CKey::Power,
                Key::RAlt => CKey::RAlt,
                Key::RBracket => CKey::RightBracket,
                Key::RControl => CKey::RCtrl,
                Key::RShift => CKey::RShift,
                Key::Semicolon => CKey::Semicolon,
                Key::Slash => CKey::Slash,
                Key::Sleep => CKey::Sleep,
                Key::Stop => CKey::Stop,
                Key::Tab => CKey::Tab,
                Key::VolumeDown => CKey::VolumeDown,
                Key::VolumeUp => CKey::VolumeUp,
                Key::Copy => CKey::Copy,
                Key::Paste => CKey::Paste,
                Key::Cut => CKey::Cut,
                _ => CKey::Unknown,
            };

            match action {
                Action::Press => Some(Input::Press(Button::Keyboard(key))),
                Action::Release => Some(Input::Release(Button::Keyboard(key))),
            }
        }
        _ => None,
    }
}
