use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{Receiver, TryIter};

use event::WindowEvent;

/// An event.
pub struct Event<'a> {
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
    fn new(value: WindowEvent, inhibitor: &RefCell<Vec<WindowEvent>>) -> Event {
        Event {
            value: value,
            inhibited: false,
            inhibitor: inhibitor,
        }
    }
}

/// An iterator through events.
pub struct Events<'a> {
    stream: TryIter<'a, WindowEvent>,
    inhibitor: &'a RefCell<Vec<WindowEvent>>,
}

impl<'a> Events<'a> {
    #[inline]
    fn new(
        stream: TryIter<'a, WindowEvent>,
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
            Some(e) => Some(Event::new(e, self.inhibitor)),
        }
    }
}

/// A stand-alone object that provides an iterator though glfw events.
///
/// It is not lifetime-bound to the main window.
pub struct EventManager {
    events: Rc<Receiver<WindowEvent>>,
    inhibitor: Rc<RefCell<Vec<WindowEvent>>>,
}

impl EventManager {
    /// Creates a new event manager.
    #[inline]
    pub fn new(
        events: Rc<Receiver<WindowEvent>>,
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
        Events::new(self.events.try_iter(), &*self.inhibitor)
    }
}
