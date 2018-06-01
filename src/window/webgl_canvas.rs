use std::sync::mpsc::Sender;

use event::{Action, Key, MouseButton, WindowEvent};
use stdweb::web::event as webevent;
use stdweb::web::{
    self, html_element::CanvasElement, EventListenerHandle, IEventTarget, IParentNode, TypedArray,
};
use stdweb::{unstable::TryInto, Value};
use window::AbstractCanvas;

pub struct WebGLCanvas {
    canvas: CanvasElement,
    key_states: [Action; Key::Unknown as usize + 1],
    button_states: [Action; MouseButton::Button8 as usize + 1],
    out_events: Sender<WindowEvent>,
    listeners: Vec<EventListenerHandle>,
}

impl AbstractCanvas for WebGLCanvas {
    fn open(
        title: &str,
        hide: bool,
        width: u32,
        height: u32,
        out_events: Sender<WindowEvent>,
    ) -> Self {
        let canvas: CanvasElement = web::document()
            .query_selector("#canvas")
            .expect("No canvas found.")
            .unwrap()
            .try_into()
            .unwrap();
        canvas.set_width(width);
        canvas.set_height(height);

        let resize = canvas.add_event_listener(|_: webevent::ResizeEvent| {});
        let listeners = vec![resize];

        WebGLCanvas {
            canvas,
            key_states: [Action::Release; Key::Unknown as usize + 1],
            button_states: [Action::Release; MouseButton::Button8 as usize + 1],
            out_events,
            listeners,
        }
    }

    fn poll_events(&self) {
        // Nothing to do.
    }

    fn swap_buffers(&mut self) {
        // Nothing to do.
    }

    fn should_close(&self) -> bool {
        false
    }

    fn size(&self) -> (u32, u32) {
        (self.canvas.width(), self.canvas.height())
    }

    fn set_title(&mut self, title: &str) {
        // Not supported.
    }

    fn close(&mut self) {
        // Not supported.
    }

    fn hide(&mut self) {
        // Not supported.
    }

    fn show(&mut self) {
        // Not supported.
    }

    fn get_mouse_button(&self, button: MouseButton) -> Action {
        self.button_states[button as usize]
    }
    fn get_key(&self, key: Key) -> Action {
        self.key_states[key as usize]
    }
}
