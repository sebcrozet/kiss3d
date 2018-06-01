use std::sync::mpsc::Sender;

use event::{Action, Key, MouseButton, WindowEvent};
#[cfg(not(target_arch = "wasm32"))]
use window::GLCanvas as CanvasImpl;
#[cfg(target_arch = "wasm32")]
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

    pub fn poll_events(&self) {
        self.canvas.poll_events()
    }

    pub fn swap_buffers(&mut self) {
        self.canvas.swap_buffers()
    }

    pub fn should_close(&self) -> bool {
        self.canvas.should_close()
    }

    pub fn size(&self) -> (u32, u32) {
        self.canvas.size()
    }

    pub fn set_title(&mut self, title: &str) {
        self.canvas.set_title(title)
    }

    pub fn close(&mut self) {
        self.canvas.close()
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
    fn poll_events(&self);
    fn swap_buffers(&mut self);
    fn should_close(&self) -> bool;
    fn size(&self) -> (u32, u32);

    fn set_title(&mut self, title: &str);
    fn close(&mut self);
    fn hide(&mut self);
    fn show(&mut self);

    fn get_mouse_button(&self, button: MouseButton) -> Action;
    fn get_key(&self, key: Key) -> Action;
}

/*
let glfw = Self::context();
let (mut window, events) = glfw
    .create_window(width, height, title, WindowMode::Windowed)
    .expect("Unable to open a glfw window.");

window.make_current();

verify!(gl::load_with(
    |name| window.get_proc_address(name) as *const _
));

// setup callbacks
window.set_framebuffer_size_polling(true);
window.set_key_polling(true);
window.set_mouse_button_polling(true);
window.set_cursor_pos_polling(true);
window.set_scroll_polling(true);
*/
