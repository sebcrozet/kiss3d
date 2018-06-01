use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::Receiver;

/// An event.
pub struct Event<'a> {
    /// The event timestamp.
    pub timestamp: f64,
    /// The exact glfw event value. This can be modified to fool the other event handlers.
    pub value: WindowEvent,
    // /// The platform-specific event.
    // pub platform_value: PlatformEvent,
    /// Set this to `true` to prevent the window or the camera from handling the event.
    pub inhibited: bool,
    inhibitor: &'a RefCell<Vec<WindowEvent>>,
}

impl<'a> Drop for Event<'a> {
    #[inline]
    fn drop(&mut self) {
        if !self.inhibited {
            self.inhibitor.borrow_mut().push(self.value.clone())
        }
    }
}

impl<'a> Event<'a> {
    #[inline]
    fn new(timestamp: f64, value: WindowEvent, inhibitor: &RefCell<Vec<WindowEvent>>) -> Event {
        Event {
            timestamp: timestamp,
            value: value,
            inhibited: false,
            inhibitor: inhibitor,
        }
    }
}

/// An iterator through events.
pub struct Events<'a> {
    stream: glfw::FlushedMessages<'a, (f64, WindowEvent)>,
    inhibitor: &'a RefCell<Vec<WindowEvent>>,
}

impl<'a> Events<'a> {
    #[inline]
    fn new(
        stream: glfw::FlushedMessages<'a, (f64, WindowEvent)>,
        inhibitor: &'a RefCell<Vec<WindowEvent>>,
    ) -> Events<'a> {
        Events {
            stream: stream,
            inhibitor: inhibitor,
        }
    }
}

impl<'a> Iterator for Events<'a> {
    type Item = Event<'a>;

    #[inline]
    fn next(&mut self) -> Option<Event<'a>> {
        match self.stream.next() {
            None => None,
            Some((t, e)) => Some(Event::new(t, e, self.inhibitor)),
        }
    }
}

/// A stand-alone object that provides an iterator though glfw events.
///
/// It is not lifetime-bound to the main window.
pub struct EventManager {
    events: Rc<Receiver<(f64, WindowEvent)>>,
    inhibitor: Rc<RefCell<Vec<WindowEvent>>>,
}

impl EventManager {
    /// Creates a new event manager.
    #[inline]
    pub fn new(
        events: Rc<Receiver<(f64, WindowEvent)>>,
        inhibitor: Rc<RefCell<Vec<WindowEvent>>>,
    ) -> EventManager {
        EventManager {
            events: events,
            inhibitor: inhibitor,
        }
    }

    /// Gets an iterator to the glfw events already collected.
    #[inline]
    pub fn iter(&mut self) -> Events {
        Events::new(glfw::flush_messages(&*self.events), &*self.inhibitor)
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum WindowEvent {
    Pos(i32, i32),
    Size(i32, i32),
    Close,
    Refresh,
    Focus(bool),
    Iconify(bool),
    FramebufferSize(i32, i32),
    MouseButton(MouseButton, Action, Modifiers),
    CursorPos(f64, f64),
    CursorEnter(bool),
    Scroll(f64, f64),
    Key(Key, Scancode, Action, Modifiers),
    Char(char),
    CharModifiers(char, Modifiers),
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Key {
    Space,
    Apostrophe,
    Comma,
    Minus,
    Period,
    Slash,
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Semicolon,
    Equal,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    LeftBracket,
    Backslash,
    RightBracket,
    GraveAccent,
    World1,
    World2,
    Escape,
    Enter,
    Tab,
    Backspace,
    Insert,
    Delete,
    Right,
    Left,
    Down,
    Up,
    PageUp,
    PageDown,
    Home,
    End,
    CapsLock,
    ScrollLock,
    NumLock,
    PrintScreen,
    Pause,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    F25,
    Kp0,
    Kp1,
    Kp2,
    Kp3,
    Kp4,
    Kp5,
    Kp6,
    Kp7,
    Kp8,
    Kp9,
    KpDecimal,
    KpDivide,
    KpMultiply,
    KpSubtract,
    KpAdd,
    KpEnter,
    KpEqual,
    LeftShift,
    LeftControl,
    LeftAlt,
    LeftSuper,
    RightShift,
    RightControl,
    RightAlt,
    RightSuper,
    Menu,
    Unknown,
}
pub type Scancode = u32;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum MouseButton {
    Button1,
    Button2,
    Button3,
    Button4,
    Button5,
    Button6,
    Button7,
    Button8,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Action {
    Release,
    Press,
    Repeat,
}

bitflags! {
    #[doc = "Key modifiers"]
    pub struct Modifiers: i32 {
        const Shift       = 0b0001;
        const Control     = 0b0010;
        const Alt         = 0b0100;
        const Super       = 0b1000;
    }
}
