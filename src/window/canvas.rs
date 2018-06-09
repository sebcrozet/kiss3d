use std::sync::mpsc::Sender;

use event::{Action, Key, MouseButton, WindowEvent};
#[cfg(not(any(target_arch = "wasm32", target_arch = "asmjs")))]
use window::GLCanvas as CanvasImpl;
#[cfg(any(target_arch = "wasm32", target_arch = "asmjs"))]
use window::WebGLCanvas as CanvasImpl;

pub struct Canvas {
    canvas: CanvasImpl,
}

impl Canvas {
    pub fn open(
        title: &str,
        hide: bool,
        width: u32,
        height: u32,
        out_events: Sender<WindowEvent>,
    ) -> Self {
        Canvas {
            canvas: CanvasImpl::open(title, hide, width, height, out_events),
        }
    }

    pub fn render_loop(data: impl FnMut(f64) -> bool + 'static) {
        CanvasImpl::render_loop(data)
    }

    pub fn poll_events(&mut self) {
        self.canvas.poll_events()
    }

    pub fn swap_buffers(&mut self) {
        self.canvas.swap_buffers()
    }

    pub fn size(&self) -> (u32, u32) {
        self.canvas.size()
    }

    pub fn hidpi_factor(&self) -> f64 {
        self.canvas.hidpi_factor()
    }

    pub fn set_title(&mut self, title: &str) {
        self.canvas.set_title(title)
    }

    pub fn hide(&mut self) {
        self.canvas.hide()
    }

    pub fn show(&mut self) {
        self.canvas.show()
    }

    pub fn get_mouse_button(&self, button: MouseButton) -> Action {
        self.canvas.get_mouse_button(button)
    }

    pub fn get_key(&self, key: Key) -> Action {
        self.canvas.get_key(key)
    }
}

pub(crate) trait AbstractCanvas {
    fn open(
        title: &str,
        hide: bool,
        width: u32,
        height: u32,
        out_events: Sender<WindowEvent>,
    ) -> Self;
    fn render_loop(data: impl FnMut(f64) -> bool + 'static);
    fn poll_events(&mut self);
    fn swap_buffers(&mut self);
    fn size(&self) -> (u32, u32);
    fn hidpi_factor(&self) -> f64;

    fn set_title(&mut self, title: &str);
    fn hide(&mut self);
    fn show(&mut self);

    fn get_mouse_button(&self, button: MouseButton) -> Action;
    fn get_key(&self, key: Key) -> Action;
}
