//! The window, and things to handle the rendering loop and events.

pub use window::window::Window;
pub use window::event::{Event, Events, EventManager};
// pub use window::render_frame::{RenderFrames, RenderFrame};

mod event;
mod window;
// mod render_frame;
