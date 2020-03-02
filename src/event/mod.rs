//! Window event handling.

pub use self::event_manager::{Event, EventManager, Events};
pub use self::window_event::{Action, Key, Modifiers, MouseButton, TouchAction, WindowEvent};

mod event_manager;
mod window_event;
