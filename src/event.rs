use std::libc;
use glfw;

// FIXME: should those be on glfw?

pub type KeyCode = libc::c_int;

#[deriving(ToStr)]
pub enum Event {
    KeyPressed(KeyCode),
    KeyReleased(KeyCode),
    FramebufferSize(f64, f64),
    ButtonPressed(MouseButton, glfw::KeyMods),
    ButtonReleased(MouseButton, glfw::KeyMods),
    CursorPos(float, float),
    Scroll(float, float)
}

pub type MouseButton = libc::c_int;
pub type MouseAction = libc::c_int;
