use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::Sender;

use event::{Action, Key, Modifiers, MouseButton, WindowEvent};
use stdweb::web::event as webevent;
use stdweb::web::event::IMouseEvent;
use stdweb::web::{
    self, html_element::CanvasElement, EventListenerHandle, IEventTarget, IHtmlElement,
    IParentNode, TypedArray,
};
use stdweb::{unstable::TryInto, Value};
use window::AbstractCanvas;

struct WebGLCanvasData {
    canvas: CanvasElement,
    key_states: [Action; Key::Unknown as usize + 1],
    button_states: [Action; MouseButton::Button8 as usize + 1],
    out_events: Sender<WindowEvent>,
}

pub struct WebGLCanvas {
    data: Rc<RefCell<WebGLCanvasData>>,
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

        let data = Rc::new(RefCell::new(WebGLCanvasData {
            canvas,
            key_states: [Action::Release; Key::Unknown as usize + 1],
            button_states: [Action::Release; MouseButton::Button8 as usize + 1],
            out_events,
        }));

        let edata = data.clone();
        let resize = web::window().add_event_listener(move |_: webevent::ResizeEvent| {
            let edata = edata.borrow();
            let (w, h) = (edata.canvas.offset_width(), edata.canvas.offset_height());
            edata.canvas.set_width(w as u32);
            edata.canvas.set_height(h as u32);
            let _ = edata
                .out_events
                .send(WindowEvent::FramebufferSize(w as u32, h as u32));
            let _ = edata.out_events.send(WindowEvent::Size(w as u32, h as u32));
        });

        let edata = data.clone();
        let mouse_down = web::window().add_event_listener(move |e: webevent::MouseDownEvent| {
            let mut edata = edata.borrow_mut();
            let button = translate_mouse_button(&e);
            let _ = edata.out_events.send(WindowEvent::MouseButton(
                button,
                Action::Press,
                translate_modifiers(&e),
            ));
            edata.button_states[button as usize] = Action::Press;
        });

        let edata = data.clone();
        let mouse_up = web::window().add_event_listener(move |e: webevent::MouseUpEvent| {
            let mut edata = edata.borrow_mut();
            let button = translate_mouse_button(&e);
            let _ = edata.out_events.send(WindowEvent::MouseButton(
                button,
                Action::Release,
                translate_modifiers(&e),
            ));
            edata.button_states[button as usize] = Action::Release;
        });

        let edata = data.clone();
        let mouse_move = web::window().add_event_listener(move |e: webevent::MouseMoveEvent| {
            let edata = edata.borrow();
            let _ = edata.out_events.send(WindowEvent::CursorPos(
                e.client_x() as f64,
                e.client_y() as f64,
            ));
        });

        let listeners = vec![resize, mouse_down, mouse_move, mouse_up];

        WebGLCanvas { data, listeners }
    }

    fn render_loop(mut callback: impl FnMut(f64) + 'static) {
        let _ = web::window().request_animation_frame(move |t| {
            callback(t);
            Self::render_loop(callback);
        });
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
        (
            self.data.borrow().canvas.offset_width() as u32,
            self.data.borrow().canvas.offset_height() as u32,
        )
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
        self.data.borrow().button_states[button as usize]
    }
    fn get_key(&self, key: Key) -> Action {
        self.data.borrow().key_states[key as usize]
    }
}

fn translate_modifiers<E: IMouseEvent>(event: &E) -> Modifiers {
    let mut res = Modifiers::empty();
    if event.shift_key() {
        res.insert(Modifiers::Shift)
    }
    if event.ctrl_key() {
        res.insert(Modifiers::Control)
    }
    if event.alt_key() {
        res.insert(Modifiers::Alt)
    }
    if event.meta_key() {
        res.insert(Modifiers::Super)
    }

    res
}

fn translate_mouse_button<E: IMouseEvent>(event: &E) -> MouseButton {
    match event.button() {
        webevent::MouseButton::Left => MouseButton::Button1,
        webevent::MouseButton::Right => MouseButton::Button2,
        webevent::MouseButton::Wheel => MouseButton::Button3,
        webevent::MouseButton::Button4 => MouseButton::Button4,
        webevent::MouseButton::Button5 => MouseButton::Button5,
    }
}
