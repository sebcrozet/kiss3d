use std::sync::mpsc::Sender;

use crate::context::Context;
use crate::event::{Action, Key, Modifiers, MouseButton, WindowEvent};
use crate::window::canvas::{CanvasSetup, NumSamples};
use crate::window::AbstractCanvas;
use image::{GenericImage, Pixel};
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, ModifiersState, MouseScrollDelta, WindowEvent as WinitWindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
    window::WindowBuilder,
};

/// A canvas based on winit and wgpu.
pub struct WgpuCanvas {
    window: winit::window::Window,
    surface: wgpu::Surface,
    surface_config: wgpu::SurfaceConfiguration,
    events: EventLoop<()>,
    cursor_pos: Option<(f64, f64)>,
    key_states: [Action; Key::Unknown as usize + 1],
    button_states: [Action; MouseButton::Button8 as usize + 1],
    modifiers_state: ModifiersState,
    out_events: Sender<WindowEvent>,
}

impl WgpuCanvas {
    /// Get the current surface texture for rendering
    pub fn get_current_texture(&self) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        self.surface.get_current_texture()
    }

    /// Get a reference to the surface
    pub fn surface(&self) -> &wgpu::Surface {
        &self.surface
    }

    /// Reconfigure the surface (e.g., after resize)
    pub fn reconfigure_surface(&mut self, width: u32, height: u32) {
        self.surface_config.width = width;
        self.surface_config.height = height;
        let device = Context::get().device();
        self.surface.configure(&*device, &self.surface_config);
    }
}

impl AbstractCanvas for WgpuCanvas {
    fn open(
        title: &str,
        hide: bool,
        width: u32,
        height: u32,
        canvas_setup: Option<CanvasSetup>,
        out_events: Sender<WindowEvent>,
    ) -> Self {
        let events = EventLoop::new();

        let window = WindowBuilder::new()
            .with_title(title)
            .with_inner_size(LogicalSize::new(width as f64, height as f64))
            .with_visible(!hide)
            .build(&events)
            .unwrap();

        // Create wgpu instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Create surface
        let surface = unsafe { instance.create_surface(&window).unwrap() };

        // Request adapter
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .unwrap();

        // Request device and queue
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        ))
        .unwrap();

        // Configure surface
        let size = window.inner_size();
        let canvas_setup = canvas_setup.unwrap_or(CanvasSetup {
            vsync: true,
            samples: NumSamples::Zero,
        });

        // Use a sensible default format
        let surface_format = wgpu::TextureFormat::Bgra8UnormSrgb;

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: if canvas_setup.vsync {
                wgpu::PresentMode::Fifo
            } else {
                wgpu::PresentMode::Immediate
            },
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };

        surface.configure(&device, &surface_config);

        // Initialize context
        Context::init(device, queue);

        WgpuCanvas {
            window,
            surface,
            surface_config,
            events,
            cursor_pos: None,
            key_states: [Action::Release; Key::Unknown as usize + 1],
            button_states: [Action::Release; MouseButton::Button8 as usize + 1],
            modifiers_state: ModifiersState::empty(),
            out_events,
        }
    }

    fn render_loop(mut callback: impl FnMut(f64) -> bool + 'static) {
        loop {
            if !callback(0.0) {
                break;
            }
        }
    }

    fn poll_events(&mut self) {
        let out_events = &mut self.out_events;
        let button_states = &mut self.button_states;
        let key_states = &mut self.key_states;
        let cursor_pos = &mut self.cursor_pos;
        let modifiers = &mut self.modifiers_state;

        self.events.run_return(|event, _, control_flow| {
            *control_flow = ControlFlow::Exit;

            match event {
                Event::WindowEvent { event, .. } => match event {
                    WinitWindowEvent::CloseRequested => {
                        let _ = out_events.send(WindowEvent::Close);
                    }
                    WinitWindowEvent::Resized(physical_size) => {
                        let _ = out_events.send(WindowEvent::FramebufferSize(
                            physical_size.width,
                            physical_size.height,
                        ));
                    }
                    WinitWindowEvent::CursorMoved { position, modifiers: mods, .. } => {
                        *modifiers = mods;
                        *cursor_pos = Some((position.x, position.y));
                        let _ = out_events.send(WindowEvent::CursorPos(
                            position.x,
                            position.y,
                            translate_modifiers(mods),
                        ));
                    }
                    WinitWindowEvent::MouseInput { state, button, modifiers: mods, .. } => {
                        *modifiers = mods;
                        let action = translate_action(state);
                        let button = translate_mouse_button(button);
                        button_states[button as usize] = action;
                        let _ = out_events.send(WindowEvent::MouseButton(
                            button,
                            action,
                            translate_modifiers(mods),
                        ));
                    }
                    WinitWindowEvent::MouseWheel { delta, modifiers: mods, .. } => {
                        *modifiers = mods;
                        let (x, y) = match delta {
                            MouseScrollDelta::LineDelta(dx, dy) => {
                                (dx as f64 * 10.0, dy as f64 * 10.0)
                            }
                            MouseScrollDelta::PixelDelta(delta) => (delta.x, delta.y),
                        };
                        let _ = out_events.send(WindowEvent::Scroll(x, y, translate_modifiers(mods)));
                    }
                    WinitWindowEvent::KeyboardInput { input, .. } => {
                        if let Some(vk) = input.virtual_keycode {
                            let action = translate_action(input.state);
                            let key = translate_key(vk);
                            key_states[key as usize] = action;
                            let _ = out_events.send(WindowEvent::Key(key, action, translate_modifiers(input.modifiers)));
                        }
                    }
                    WinitWindowEvent::ReceivedCharacter(ch) => {
                        let _ = out_events.send(WindowEvent::Char(ch));
                    }
                    WinitWindowEvent::ModifiersChanged(mods) => {
                        *modifiers = mods;
                    }
                    _ => {}
                },
                _ => {}
            }
        });
    }

    fn swap_buffers(&mut self) {
        // No-op - wgpu handles presentation automatically
    }

    fn size(&self) -> (u32, u32) {
        let size = self.window.inner_size();
        (size.width, size.height)
    }

    fn cursor_pos(&self) -> Option<(f64, f64)> {
        self.cursor_pos
    }

    fn scale_factor(&self) -> f64 {
        self.window.scale_factor()
    }

    fn set_title(&mut self, title: &str) {
        self.window.set_title(title)
    }

    fn set_icon(&mut self, icon: impl GenericImage<Pixel = impl Pixel<Subpixel = u8>>) {
        let (width, height) = icon.dimensions();
        let mut rgba = Vec::with_capacity((width * height) as usize * 4);
        for (_, _, pixel) in icon.pixels() {
            rgba.extend_from_slice(&pixel.to_rgba().0);
        }
        let icon = winit::window::Icon::from_rgba(rgba, width, height).unwrap();
        self.window.set_window_icon(Some(icon))
    }

    fn set_cursor_grab(&self, _grab: bool) {
        // TODO: Implement with winit 0.29 API
    }

    fn set_cursor_position(&self, x: f64, y: f64) {
        let _ = self
            .window
            .set_cursor_position(winit::dpi::PhysicalPosition::new(x, y));
    }

    fn hide_cursor(&self, hide: bool) {
        self.window.set_cursor_visible(!hide)
    }

    fn hide(&mut self) {
        self.window.set_visible(false)
    }

    fn show(&mut self) {
        self.window.set_visible(true)
    }

    fn get_mouse_button(&self, button: MouseButton) -> Action {
        self.button_states[button as usize]
    }

    fn get_key(&self, key: Key) -> Action {
        self.key_states[key as usize]
    }

    fn begin_frame(&mut self) {
        if let Ok(surface_texture) = self.surface.get_current_texture() {
            Context::get().begin_frame(surface_texture);
        }
    }

    fn end_frame(&mut self) {
        Context::get().end_frame();
    }
}

fn translate_action(action: ElementState) -> Action {
    match action {
        ElementState::Pressed => Action::Press,
        ElementState::Released => Action::Release,
    }
}

fn translate_modifiers(mods: ModifiersState) -> Modifiers {
    let mut res = Modifiers::empty();
    if mods.shift() {
        res.insert(Modifiers::Shift);
    }
    if mods.ctrl() {
        res.insert(Modifiers::Control);
    }
    if mods.alt() {
        res.insert(Modifiers::Alt);
    }
    if mods.logo() {
        res.insert(Modifiers::Super);
    }
    res
}

fn translate_mouse_button(button: winit::event::MouseButton) -> MouseButton {
    match button {
        winit::event::MouseButton::Left => MouseButton::Button1,
        winit::event::MouseButton::Right => MouseButton::Button2,
        winit::event::MouseButton::Middle => MouseButton::Button3,
        winit::event::MouseButton::Other(4) => MouseButton::Button4,
        winit::event::MouseButton::Other(5) => MouseButton::Button5,
        winit::event::MouseButton::Other(6) => MouseButton::Button6,
        winit::event::MouseButton::Other(7) => MouseButton::Button7,
        winit::event::MouseButton::Other(_) => MouseButton::Button8,
    }
}

fn translate_key(keycode: winit::event::VirtualKeyCode) -> Key {
    use winit::event::VirtualKeyCode;

    match keycode {
        VirtualKeyCode::Key1 => Key::Key1,
        VirtualKeyCode::Key2 => Key::Key2,
        VirtualKeyCode::Key3 => Key::Key3,
        VirtualKeyCode::Key4 => Key::Key4,
        VirtualKeyCode::Key5 => Key::Key5,
        VirtualKeyCode::Key6 => Key::Key6,
        VirtualKeyCode::Key7 => Key::Key7,
        VirtualKeyCode::Key8 => Key::Key8,
        VirtualKeyCode::Key9 => Key::Key9,
        VirtualKeyCode::Key0 => Key::Key0,
        VirtualKeyCode::A => Key::A,
        VirtualKeyCode::B => Key::B,
        VirtualKeyCode::C => Key::C,
        VirtualKeyCode::D => Key::D,
        VirtualKeyCode::E => Key::E,
        VirtualKeyCode::F => Key::F,
        VirtualKeyCode::G => Key::G,
        VirtualKeyCode::H => Key::H,
        VirtualKeyCode::I => Key::I,
        VirtualKeyCode::J => Key::J,
        VirtualKeyCode::K => Key::K,
        VirtualKeyCode::L => Key::L,
        VirtualKeyCode::M => Key::M,
        VirtualKeyCode::N => Key::N,
        VirtualKeyCode::O => Key::O,
        VirtualKeyCode::P => Key::P,
        VirtualKeyCode::Q => Key::Q,
        VirtualKeyCode::R => Key::R,
        VirtualKeyCode::S => Key::S,
        VirtualKeyCode::T => Key::T,
        VirtualKeyCode::U => Key::U,
        VirtualKeyCode::V => Key::V,
        VirtualKeyCode::W => Key::W,
        VirtualKeyCode::X => Key::X,
        VirtualKeyCode::Y => Key::Y,
        VirtualKeyCode::Z => Key::Z,
        VirtualKeyCode::Escape => Key::Escape,
        VirtualKeyCode::F1 => Key::F1,
        VirtualKeyCode::F2 => Key::F2,
        VirtualKeyCode::F3 => Key::F3,
        VirtualKeyCode::F4 => Key::F4,
        VirtualKeyCode::F5 => Key::F5,
        VirtualKeyCode::F6 => Key::F6,
        VirtualKeyCode::F7 => Key::F7,
        VirtualKeyCode::F8 => Key::F8,
        VirtualKeyCode::F9 => Key::F9,
        VirtualKeyCode::F10 => Key::F10,
        VirtualKeyCode::F11 => Key::F11,
        VirtualKeyCode::F12 => Key::F12,
        VirtualKeyCode::F13 => Key::F13,
        VirtualKeyCode::F14 => Key::F14,
        VirtualKeyCode::F15 => Key::F15,
        VirtualKeyCode::Insert => Key::Insert,
        VirtualKeyCode::Home => Key::Home,
        VirtualKeyCode::Delete => Key::Delete,
        VirtualKeyCode::End => Key::End,
        VirtualKeyCode::PageDown => Key::PageDown,
        VirtualKeyCode::PageUp => Key::PageUp,
        VirtualKeyCode::Left => Key::Left,
        VirtualKeyCode::Up => Key::Up,
        VirtualKeyCode::Right => Key::Right,
        VirtualKeyCode::Down => Key::Down,
        VirtualKeyCode::Back => Key::Back,
        VirtualKeyCode::Return => Key::Return,
        VirtualKeyCode::Space => Key::Space,
        VirtualKeyCode::Tab => Key::Tab,
        VirtualKeyCode::Numpad0 => Key::Numpad0,
        VirtualKeyCode::Numpad1 => Key::Numpad1,
        VirtualKeyCode::Numpad2 => Key::Numpad2,
        VirtualKeyCode::Numpad3 => Key::Numpad3,
        VirtualKeyCode::Numpad4 => Key::Numpad4,
        VirtualKeyCode::Numpad5 => Key::Numpad5,
        VirtualKeyCode::Numpad6 => Key::Numpad6,
        VirtualKeyCode::Numpad7 => Key::Numpad7,
        VirtualKeyCode::Numpad8 => Key::Numpad8,
        VirtualKeyCode::Numpad9 => Key::Numpad9,
        _ => Key::Unknown,
    }
}
