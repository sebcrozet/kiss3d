use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::Sender;

use crate::event::{Action, Key, MouseButton, WindowEvent};
use crate::window::{AbstractCanvas, CanvasSetup};
use image::{GenericImage, Pixel};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;

struct WgpuWasmCanvasData {
    canvas: HtmlCanvasElement,
    cursor_pos: Option<(f64, f64)>,
    key_states: [Action; Key::Unknown as usize + 1],
    button_states: [Action; MouseButton::Button8 as usize + 1],
    pending_events: Vec<WindowEvent>,
    out_events: Sender<WindowEvent>,
    scale_factor: f64,
}

/// A WASM canvas based on wgpu and winit.
pub struct WgpuWasmCanvas {
    data: Rc<RefCell<WgpuWasmCanvasData>>,
    #[allow(dead_code)]
    event_listeners: Vec<Box<dyn std::any::Any>>,
}

impl AbstractCanvas for WgpuWasmCanvas {
    fn open(
        _title: &str,
        _hide: bool,
        _width: u32,
        _height: u32,
        _setup: Option<CanvasSetup>,
        out_events: Sender<WindowEvent>,
    ) -> Self {
        let window = web_sys::window().expect("Failed to obtain window");
        let document = window.document().expect("Failed to obtain document");
        let scale_factor = window.device_pixel_ratio();

        let canvas: HtmlCanvasElement = match document.get_element_by_id("canvas") {
            Some(element) => element
                .dyn_into::<HtmlCanvasElement>()
                .expect("Canvas element is not an actual canvas."),
            None => {
                let canvas = document
                    .create_element("canvas")
                    .expect("Failed to create canvas element")
                    .dyn_into::<HtmlCanvasElement>()
                    .expect("Created element is not a canvas");

                canvas.set_id("canvas");
                canvas
                    .set_attribute("style", "width: 100vw; height: 100vh; display: block;")
                    .ok();

                document
                    .body()
                    .expect("Document has no body")
                    .append_child(&canvas)
                    .expect("Failed to append canvas to body");

                canvas
            }
        };

        let w = (canvas.offset_width() as f64 * scale_factor) as u32;
        let h = (canvas.offset_height() as f64 * scale_factor) as u32;
        canvas.set_width(w);
        canvas.set_height(h);

        // For WASM, we don't initialize wgpu here due to async requirements
        // In a full implementation, this would use proper async initialization
        // For now, we'll initialize it when needed

        let data = Rc::new(RefCell::new(WgpuWasmCanvasData {
            canvas,
            cursor_pos: None,
            key_states: [Action::Release; Key::Unknown as usize + 1],
            button_states: [Action::Release; MouseButton::Button8 as usize + 1],
            pending_events: vec![WindowEvent::FramebufferSize(w, h)],
            out_events,
            scale_factor,
        }));

        // Add event listeners (simplified version)
        let event_listeners = Vec::new();

        WgpuWasmCanvas {
            data,
            event_listeners,
        }
    }

    fn render_loop(mut callback: impl FnMut(f64) -> bool + 'static) {
        if let Some(window) = web_sys::window() {
            let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
            let g = f.clone();
            *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
                if callback(0.0) {
                    let _ = window.request_animation_frame(
                        f.borrow().as_ref().unwrap().as_ref().unchecked_ref(),
                    );
                } else {
                    f.borrow_mut().take();
                }
            }) as Box<dyn FnMut()>));

            let _ = web_sys::window()
                .unwrap()
                .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref());
        }
    }

    fn poll_events(&mut self) {
        let events: Vec<_> = {
            let data = self.data.borrow();
            data.pending_events.clone()
        };

        {
            let mut data = self.data.borrow_mut();
            data.pending_events.clear();
        }

        for e in events {
            let _ = self.data.borrow().out_events.send(e);
        }
    }

    fn swap_buffers(&mut self) {
        // No-op for wgpu
    }

    fn size(&self) -> (u32, u32) {
        let data = self.data.borrow();
        let scale_factor = data.scale_factor;
        (
            (data.canvas.offset_width() as f64 * scale_factor) as u32,
            (data.canvas.offset_height() as f64 * scale_factor) as u32,
        )
    }

    fn cursor_pos(&self) -> Option<(f64, f64)> {
        self.data.borrow().cursor_pos
    }

    fn scale_factor(&self) -> f64 {
        self.data.borrow().scale_factor
    }

    fn set_title(&mut self, _: &str) {
        // Not supported
    }

    fn set_icon(&mut self, _icon: impl GenericImage<Pixel = impl Pixel<Subpixel = u8>>) {
        // Not supported
    }

    fn set_cursor_grab(&self, _: bool) {
        // Not supported
    }

    fn set_cursor_position(&self, _: f64, _: f64) {
        // Not supported
    }

    fn hide_cursor(&self, _: bool) {
        // Not supported
    }

    fn hide(&mut self) {
        // Not supported
    }

    fn show(&mut self) {
        // Not supported
    }

    fn get_mouse_button(&self, button: MouseButton) -> Action {
        self.data.borrow().button_states[button as usize]
    }

    fn get_key(&self, key: Key) -> Action {
        self.data.borrow().key_states[key as usize]
    }

    fn begin_frame(&mut self) {
        // WASM rendering would need async surface acquisition
        // For now, this is a stub
    }

    fn end_frame(&mut self) {
        // WASM presentation handled by browser
    }
}
