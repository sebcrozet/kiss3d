use glfw;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::Receiver;

// #[derive(Debug, Clone)]
// pub enum AppEvent {
//     MouseDown(MouseButton, Action, Modifiers),
//     MouseUp(MouseButtonEvent),
//     KeyDown(KeyDownEvent),
//     KeyUp(KeyUpEvent),
//     Resized((u32, u32)),
//     MousePos((f64, f64)),
//     Close,
//     Unknown,
// }

//     Close,
//     Refresh,
//     Focus(bool),
//     Iconify(bool),
//     FramebufferSize(i32, i32),
//     MouseButton(MouseButton, Action, Modifiers),
//     CursorPos(f64, f64),
//     CursorEnter(bool),
//     Scroll(f64, f64),
//     Key(Key, Scancode, Action, Modifiers),
//     Char(char),
//     CharModifiers(char, Modifiers),
//     FileDrop(Vec<PathBuf>),
// }

/// An event.
pub struct Event<'a> {
    /// The event timestamp.
    pub timestamp: f64,
    /// The exact glfw event value. This can be modified to fool the other event handlers.
    pub value: glfw::WindowEvent,
    // /// The platform-specific event.
    // pub platform_value: PlatformEvent,
    /// Set this to `true` to prevent the window or the camera from handling the event.
    pub inhibited: bool,
    inhibitor: &'a RefCell<Vec<glfw::WindowEvent>>,
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
    fn new(
        timestamp: f64,
        value: glfw::WindowEvent,
        inhibitor: &RefCell<Vec<glfw::WindowEvent>>,
    ) -> Event {
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
    stream: glfw::FlushedMessages<'a, (f64, glfw::WindowEvent)>,
    inhibitor: &'a RefCell<Vec<glfw::WindowEvent>>,
}

impl<'a> Events<'a> {
    #[inline]
    fn new(
        stream: glfw::FlushedMessages<'a, (f64, glfw::WindowEvent)>,
        inhibitor: &'a RefCell<Vec<glfw::WindowEvent>>,
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
    events: Rc<Receiver<(f64, glfw::WindowEvent)>>,
    inhibitor: Rc<RefCell<Vec<glfw::WindowEvent>>>,
}

impl EventManager {
    /// Creates a new event manager.
    #[inline]
    pub fn new(
        events: Rc<Receiver<(f64, glfw::WindowEvent)>>,
        inhibitor: Rc<RefCell<Vec<glfw::WindowEvent>>>,
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
