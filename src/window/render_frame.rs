use glfw;
use std::cell::RefCell;
use std::rc::Rc;
use nalgebra::na::{Vec2, Vec3};
use nalgebra::na;
use text::Font;
use camera::{Camera, ArcBall};
use post_processing::PostProcessingEffect;
use window::EventManager;
use window::Window;

/// A frame that is going to be rendered.
///
/// The scene is rendered when the render frame destructor is called.
/// Any modification made to a `RenderFrame` is not permanent.
/// For exemple, if the current `RenderFrame` camera is modified, the next render frame will use to
/// the default camera again.
pub struct RenderFrame<'a, 'b> {
    events:          Rc<Receiver<(f64, glfw::WindowEvent)>>,
    collector:       Rc<RefCell<Vec<glfw::WindowEvent>>>,
    window:          &'a mut Window<'b>,
    camera:          &'a mut Camera,
    post_processing: Option<&'a mut PostProcessingEffect>,
    skip_draw:       bool,
    events_handled:  bool
}

impl<'a, 'b> RenderFrame<'a, 'b> {
    #[doc(hidden)]
    #[inline]
    pub fn new(window:    &'a mut Window<'b>,
               camera:    &'a mut Camera,
               events:    Rc<Receiver<(f64, glfw::WindowEvent)>>,
               collector: Rc<RefCell<Vec<glfw::WindowEvent>>>)
               -> RenderFrame<'a, 'b> {
        let sz = window.size();

        camera.handle_event(window.glfw_window(), &glfw::FramebufferSizeEvent(sz.x as i32, sz.y as i32));

        RenderFrame {
            window:          window,
            skip_draw:       false,
            camera:          camera,
            post_processing: None,
            events:          events,
            collector:       collector,
            events_handled:  false
        }
    }

    /// Gets an event manager that can provide an iterator through the window events.
    #[inline]
    pub fn events(&self) -> EventManager {
        EventManager::new(self.events.clone(), self.collector.clone())
    }

    /// The window this render frame will be drawing in.
    #[inline]
    pub fn window<'c>(&'c mut self) -> &'c mut Window<'b> {
        &'c mut *self.window
    }

    /// Sets the camera to be used for the next render.
    #[inline]
    pub fn set_camera(&mut self, camera: &'a mut Camera) {
        // FIXME: too bad we have to do this every times…
        // NOTE: we could make the camera store the w and h so that it does not recompute
        // everything each time it gets a framebuffer event with the same w/h values.
        let sz = self.window.size();
        camera.handle_event(self.window.glfw_window(),
                            &glfw::FramebufferSizeEvent(sz.x as i32, sz.y as i32));

        self.camera = camera
    }

    /// Sets the current processing effect.
    #[inline]
    pub fn set_post_processing_effect(&mut self, effect: &'a mut PostProcessingEffect) {
        self.post_processing = Some(effect);
    }

    /// Skip the rendering for this render frame.
    #[inline]
    pub fn inhibit_draw(&mut self) {
        self.skip_draw = true
    }

    /// Whether or not this frame will be actually rendered.
    #[inline]
    pub fn draw_inhibited(&self) -> bool {
        self.skip_draw
    }

    /// Prevents, for this frame, any event from being interpreted by the render frame window and
    /// the camera.
    #[inline]
    pub fn inhibit_events(&mut self) {
        self.events_handled = true
    }

    /// Whether or not the events will be actually handled during this frame.
    #[inline]
    pub fn events_inhibited(&self) -> bool {
        self.events_handled
    }

    /// The current camera.
    #[inline]
    pub fn camera<'c>(&'c mut self) -> &'c mut Camera {
        let res: &'c mut Camera = self.camera;

        res
    }

    /// Adds a line to be drawn during the next frame.
    #[inline]
    pub fn draw_line(&mut self, a: &Vec3<f32>, b: &Vec3<f32>, color: &Vec3<f32>) {
        self.window.draw_line(a, b, color);
    }

    /// Adds a point to be drawn during the next frame.
    #[inline]
    pub fn draw_point(&mut self, pt: &Vec3<f32>, color: &Vec3<f32>) {
        self.window.draw_point(pt, color);
    }

    /// Adds a string to be drawn during the next frame.
    #[inline]
    pub fn draw_text(&mut self, text: &str, pos: &Vec2<f32>, font: &Rc<Font>, color: &Vec3<f32>) {
        self.window.draw_text(text, pos, font, color);
    }
}

#[unsafe_destructor]
impl<'a, 'b> Drop for RenderFrame<'a, 'b> {
    #[inline]
    fn drop(&mut self) {
        for event in self.collector.borrow().iter() {
            self.window.handle_event(event);
            self.camera.handle_event(self.window.glfw_window(), event);
        }

        if !self.events_handled {
            for event in glfw::flush_messages(self.events.deref()) {
                self.window.handle_event(event.ref1());
                self.camera.handle_event(self.window.glfw_window(), event.ref1());
            }
        }

        self.collector.borrow_mut().clear();

        if !self.skip_draw {
            self.window.draw(self.camera, &mut self.post_processing);
        }
    }
}

/// An iterator through render frames.
pub struct RenderFrames<'a, 'b> {
    events:    Rc<Receiver<(f64, glfw::WindowEvent)>>,
    collector: Rc<RefCell<Vec<glfw::WindowEvent>>>,
    window:    &'a mut Window<'b>,
    camera:    Box<Camera>
}

impl<'a, 'b> RenderFrames<'a, 'b> {
    #[doc(hidden)]
    pub fn new(events: Rc<Receiver<(f64, glfw::WindowEvent)>>,
               window: &'a mut Window<'b>)
               -> RenderFrames<'a, 'b> {
        RenderFrames::new_with_camera(events, window, box ArcBall::new(-Vec3::z(), na::zero()))
    }

    #[doc(hidden)]
    pub fn new_with_camera(events: Rc<Receiver<(f64, glfw::WindowEvent)>>,
                           window: &'a mut Window<'b>,
                           camera: Box<Camera>)
                           -> RenderFrames<'a, 'b> {
        RenderFrames {
            events:    events,
            collector: Rc::new(RefCell::new(Vec::new())),
            window:    window,
            camera:    camera
        }
    }

    /// Gets the next frame to be rendered.
    #[inline]
    pub fn next<'c>(&'c mut self) -> Option<RenderFrame<'c, 'b>> {
        if !self.window.glfw_window().should_close() {
            self.window.context().poll_events();
            self.collector.borrow_mut().clear();

            Some(RenderFrame::new(self.window, self.camera, self.events.clone(), self.collector.clone()))
        }
        else {
            None
        }
    }
}


