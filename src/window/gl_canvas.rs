use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::Sender;

use context::Context;
use event::{Action, Key, Modifiers, MouseButton, WindowEvent};
use gl;
use glutin::{self, ContextBuilder, EventsLoop, GlContext, GlRequest, GlWindow, WindowBuilder};
use window::AbstractCanvas;

struct GLCanvasData {}

pub struct GLCanvas {
    window: GlWindow,
    events: EventsLoop,
    key_states: [Action; Key::Unknown as usize + 1],
    button_states: [Action; MouseButton::Button8 as usize + 1],
    out_events: Sender<WindowEvent>,
    // listeners: Vec<EventListenerHandle>,
}

impl AbstractCanvas for GLCanvas {
    fn open(
        title: &str,
        hide: bool,
        width: u32,
        height: u32,
        out_events: Sender<WindowEvent>,
    ) -> Self {
        let events = EventsLoop::new();
        let window = WindowBuilder::new()
            .with_title(title)
            .with_dimensions(width, height)
            .with_visibility(!hide);
        let context = ContextBuilder::new()
            .with_vsync(true)
            .with_gl(GlRequest::GlThenGles {
                opengl_version: (3, 2),
                opengles_version: (2, 0),
            });
        let window = GlWindow::new(window, context, &events).unwrap();
        let _ = unsafe { window.make_current().unwrap() };
        verify!(gl::load_with(
            |name| window.context().get_proc_address(name) as *const _
        ));

        unsafe {
            // Setup a single VAO.
            let mut vao = 0;
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
        }

        GLCanvas {
            window,
            events,
            key_states: [Action::Release; Key::Unknown as usize + 1],
            button_states: [Action::Release; MouseButton::Button8 as usize + 1],
            out_events,
        }
    }

    fn render_loop(mut callback: impl FnMut(f64) + 'static) {
        loop {
            callback(0.0) // XXX: timestamp
        }
    }

    fn poll_events(&mut self) {
        let out_events = &mut self.out_events;
        let mut window = &mut self.window;
        let mut button_states = &mut self.button_states;
        let mut key_states = &mut self.key_states;

        self.events.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::Resized(w, h) => {
                    if w != 0 && h != 0 {
                        window.context().resize(w, h);
                        window.set_inner_size(w, h);
                        let _ = out_events.send(WindowEvent::FramebufferSize(w, h));
                    }
                }
                glutin::WindowEvent::CursorMoved { position, .. } => {
                    let _ = out_events.send(WindowEvent::CursorPos(position.0, position.1));
                }
                glutin::WindowEvent::MouseInput {
                    state,
                    button,
                    modifiers,
                    ..
                } => {
                    let action = translate_action(state);
                    let button = translate_mouse_button(button);
                    let modifiers = translate_modifiers(modifiers);
                    button_states[button as usize] = action;
                    let _ = out_events.send(WindowEvent::MouseButton(button, action, modifiers));
                }
                glutin::WindowEvent::MouseWheel { delta, .. } => {
                    let (x, y) = match delta {
                        glutin::MouseScrollDelta::LineDelta(dx, dy)
                        | glutin::MouseScrollDelta::PixelDelta(dx, dy) => (dx, dy),
                    };
                    let _ = out_events.send(WindowEvent::Scroll(x as f64, y as f64));
                }
                _ => {}
            },
            _ => {}
        })
    }

    fn swap_buffers(&mut self) {
        let _ = self.window.swap_buffers();
    }

    fn should_close(&self) -> bool {
        false
    }

    fn size(&self) -> (u32, u32) {
        self.window
            .get_inner_size()
            .expect("The window was closed.")
    }

    fn set_title(&mut self, title: &str) {
        self.window.set_title(title)
    }

    fn close(&mut self) {
        // Not supported.
    }

    fn hide(&mut self) {
        self.window.hide()
    }

    fn show(&mut self) {
        self.window.show()
    }

    fn get_mouse_button(&self, button: MouseButton) -> Action {
        self.button_states[button as usize]
    }
    fn get_key(&self, key: Key) -> Action {
        self.key_states[key as usize]
    }
}

fn translate_action(action: glutin::ElementState) -> Action {
    match action {
        glutin::ElementState::Pressed => Action::Press,
        glutin::ElementState::Released => Action::Release,
    }
}

fn translate_modifiers(modifiers: glutin::ModifiersState) -> Modifiers {
    let mut res = Modifiers::empty();
    if modifiers.shift {
        res.insert(Modifiers::Shift)
    }
    if modifiers.ctrl {
        res.insert(Modifiers::Control)
    }
    if modifiers.alt {
        res.insert(Modifiers::Alt)
    }
    if modifiers.logo {
        res.insert(Modifiers::Super)
    }

    res
}

fn translate_mouse_button(button: glutin::MouseButton) -> MouseButton {
    match button {
        glutin::MouseButton::Left => MouseButton::Button1,
        glutin::MouseButton::Right => MouseButton::Button2,
        glutin::MouseButton::Middle => MouseButton::Button3,
        _ => MouseButton::Button4, // FIXME: default is not good.
    }
}
