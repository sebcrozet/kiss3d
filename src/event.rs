use glfw;

// FIXME: should those be on glfw?

/// Wrapper around glfw events.
#[deriving(ToStr)]
pub enum Event {
    /// Event triggered when a keyboard key is pressed.
    KeyPressed(glfw::Key),
    /// Event triggered when a keyboard key is released.
    KeyReleased(glfw::Key),
    /// Event triggered when the window framebuffer is resized.
    FramebufferSize(f64, f64),
    /// Event triggered when a mouse button is pressed.
    ButtonPressed(glfw::MouseButton,  glfw::Modifiers),
    /// Event triggered when a mouse button is released.
    ButtonReleased(glfw::MouseButton, glfw::Modifiers),
    /// Event triggered when the cursor position changes.
    CursorPos(f64, f64),
    /// Event triggered when the mouse scrolls.
    Scroll(f64, f64)
}
