use glfw;

// FIXME: should those be on glfw?

#[deriving(ToStr)]
pub enum Event {
    KeyPressed(glfw::Key),
    KeyReleased(glfw::Key),
    FramebufferSize(f64, f64),
    ButtonPressed(glfw::MouseButton,  glfw::Modifiers),
    ButtonReleased(glfw::MouseButton, glfw::Modifiers),
    CursorPos(float, float),
    Scroll(float, float)
}
