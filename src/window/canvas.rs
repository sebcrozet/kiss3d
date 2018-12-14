use std::sync::mpsc::Sender;

use event::{Action, Key, MouseButton, WindowEvent};
#[cfg(not(any(target_arch = "wasm32", target_arch = "asmjs")))]
use window::GLCanvas as CanvasImpl;
#[cfg(any(target_arch = "wasm32", target_arch = "asmjs"))]
use window::WebGLCanvas as CanvasImpl;
use image::{GenericImage, Pixel};

/// An abstract structure representing a window for native applications, and a canvas for web applications.
pub struct Canvas {
    canvas: CanvasImpl,
}

impl Canvas {
    /// Open a new window, and initialize the OpenGL/WebGL context.
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

    /// Run the platform-specific render loop.
    pub fn render_loop(data: impl FnMut(f64) -> bool + 'static) {
        CanvasImpl::render_loop(data)
    }

    /// Poll all events tha occurred since the last call to this method.
    pub fn poll_events(&mut self) {
        self.canvas.poll_events()
    }

    /// If double-buffering is supported, swap the buffers.
    pub fn swap_buffers(&mut self) {
        self.canvas.swap_buffers()
    }

    /// The size of the window.
    pub fn size(&self) -> (u32, u32) {
        self.canvas.size()
    }

    /// The current position of the cursor, if known.
    ///
    /// This position may not be known if, e.g., the cursor has not been moved since the
    /// window was open.
    pub fn cursor_pos(&self) -> Option<(f64, f64)> {
        self.canvas.cursor_pos()
    }

    /// The high-dpi factor.
    pub fn hidpi_factor(&self) -> f64 {
        self.canvas.hidpi_factor()
    }

    /// Set the window title.
    pub fn set_title(&mut self, title: &str) {
        self.canvas.set_title(title)
    }

    /// Set the window icon. See `Window::set_icon` for details.
    pub fn set_icon(&mut self, icon: impl GenericImage<Pixel = impl Pixel<Subpixel = u8>>) {
        self.canvas.set_icon(icon)
    }

    /// Hide the window.
    pub fn hide(&mut self) {
        self.canvas.hide()
    }

    /// Show the window.
    pub fn show(&mut self) {
        self.canvas.show()
    }

    /// The state of a mouse button.
    pub fn get_mouse_button(&self, button: MouseButton) -> Action {
        self.canvas.get_mouse_button(button)
    }

    /// The state of a key.
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
    fn cursor_pos(&self) -> Option<(f64, f64)>;
    fn hidpi_factor(&self) -> f64;

    fn set_title(&mut self, title: &str);
    fn set_icon(&mut self, icon: impl GenericImage<Pixel = impl Pixel<Subpixel = u8>>);
    fn hide(&mut self);
    fn show(&mut self);

    fn get_mouse_button(&self, button: MouseButton) -> Action;
    fn get_key(&self, key: Key) -> Action;
}
