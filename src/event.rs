use std::libc;

// FIXME: should those be on glfw?

pub type KeyCode = libc::c_int;

pub enum KeyboardEvent
{
  KeyPressed(KeyCode),
  KeyReleased(KeyCode)
}

pub type MouseButton = libc::c_int;
pub type MouseAction = libc::c_int;
pub type MouseMods   = libc::c_int;

pub enum MouseEvent
{
  ButtonPressed(MouseButton, MouseMods),
  ButtonReleased(MouseButton, MouseMods),
  CursorPos(float, float),
  Scroll(float, float)
}
