use glfw;
use std::cell::RefCell;
use std::rc::Rc;
use nalgebra::na::{Vec2, Vec3};
use text::Font;
use camera::Camera;
use post_processing::PostProcessingEffect;
use window::EventManager;
use window::Window;

/// A frame that is going to be rendered.
///
/// The scene is rendered when the render frame destructor is called.
/// Any modification made to a `RenderFrame` is not permanent.
/// For exemple, if the current `RenderFrame` camera is modified, the next render frame will use to
/// the default camera again.
pub struct RenderFrame<'a, C> {
    camera:          Option<&'a mut Camera>,
    post_processing: Option<&'a mut PostProcessingEffect>,
    default_cam:     &'a mut C,
}

impl<'a, C: 'static + Camera> RenderFrame<'a, C> {
    #[doc(hidden)]
    #[inline]
    pub fn new(default_cam: &'a mut C) -> RenderFrame<'a, C> {
        let sz = window.size();

        default_cam.handle_event(window.glfw_window(), &glfw::FramebufferSizeEvent(sz.x as i32, sz.y as i32));

        RenderFrame {
            window:          window,
            skip_draw:       false,
            camera:          None,
            default_cam:     default_cam,
            post_processing: None,
            events:          events,
            collector:       collector,
            events_handled:  false
        }
    }

    /// Sets the current processing effect.
    #[inline]
    pub fn set_post_processing_effect(&mut self, effect: &'a mut PostProcessingEffect) {
        self.post_processing = Some(effect);
    }

    /// Sets the camera to be used for the next render.
    #[inline]
    pub fn set_camera(&mut self, camera: &'a mut Camera) {
        self.camera = Some(camera)
    }

    /// The camera that will be used for the next render.
    #[inline]
    pub fn camera<'c>(&'c mut self) -> &'c mut Camera { // FIXME: add an immutable version?
        let cam_mut: Option<&'c mut &'a mut Camera> = self.camera.as_mut(); // help the borrow-checker.
        match cam_mut {
            Some(cam) => {
                let res: &'c mut Camera = *cam;
                res
            },
            None      => &mut *self.default_cam as &'c mut Camera
        }
    }

    /// Gets the default camera − that is − the camera the window started its iterations with.
    ///
    /// This is the camera that will be used to render this frame if no call to `set_camera` is
    /// made during this rendering loop.
    pub fn default_camera<'c>(&'c mut self) -> &'c mut C { // FIXME: add an immutable version?
        let res: &'c mut C = self.default_cam;

        res
    }
}
