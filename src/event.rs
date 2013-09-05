use std::libc;
use glfw;

// FIXME: should those be on glfw?

pub type KeyCode = libc::c_int;

#[deriving(ToStr)]
pub enum Event {
    Keyboard(KeyboardEvent),
    Mouse(MouseEvent),
    FramebufferSize(f64, f64)
}

#[deriving(ToStr)]
pub enum KeyboardEvent {
    KeyPressed(KeyCode),
    KeyReleased(KeyCode)
}

pub type MouseButton = libc::c_int;
pub type MouseAction = libc::c_int;

#[deriving(ToStr)]
pub enum MouseEvent {
    ButtonPressed(MouseButton, glfw::KeyMods),
    ButtonReleased(MouseButton, glfw::KeyMods),
    CursorPos(float, float),
    Scroll(float, float)
}
