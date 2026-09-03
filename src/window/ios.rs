//! iOS support: the inverted main loop.
//!
//! iOS forbids the pump model the other native platforms use. winit's
//! `run_app` calls `UIApplicationMain`, which owns the main thread and never
//! returns, and `pump_events` does not exist on the platform. So the roles
//! flip, the same way they do on the web: winit drives, and the user's
//! `async fn main` becomes a future polled once per loop turn. The renderer's
//! per-frame [`next_frame`]`().await` is the seam that makes the user's
//! `while window.render().await` loop yield back to UIKit between frames —
//! the iOS analogue of the wasm path awaiting `requestAnimationFrame`.

use std::cell::Cell;
use std::future::Future;
use std::pin::Pin;
use std::ptr;
use std::task::{Context as TaskContext, Poll, Waker};

use objc2::rc::Retained;
use objc2::{define_class, msg_send, MainThreadMarker, MainThreadOnly};
use objc2_foundation::{NSObjectProtocol, NSString};
use objc2_ui_kit::{UIKeyInput, UITextInputTraits, UIView};
use winit::application::ApplicationHandler;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

thread_local! {
    // The ActiveEventLoop of the winit callback currently on the stack. Only
    // valid inside that callback: IosApp sets it before polling the app
    // future and clears it after, and `create_window` reads it when the
    // future opens a window mid-poll.
    static ACTIVE: Cell<*const ActiveEventLoop> = const { Cell::new(ptr::null()) };
    // Turn counter that resumes stalled `next_frame` futures: a frame that
    // yielded in turn N completes its await in turn N+1.
    static FRAME: Cell<u64> = const { Cell::new(0) };
}

/// Creates a winit window from inside the running app future.
///
/// # Panics
/// Outside a `run_ios` poll — on iOS a window can only be created while
/// winit's loop is on the stack, which `#[kiss3d::main]` guarantees.
pub(crate) fn create_window(attrs: WindowAttributes) -> Window {
    ACTIVE.with(|active| {
        let ptr = active.get();
        assert!(
            !ptr.is_null(),
            "on iOS, windows can only be created inside #[kiss3d::main]'s event loop"
        );
        // Valid for the duration of the callback that set it; this is only
        // reachable from the app future, which is polled inside one.
        let event_loop = unsafe { &*ptr };
        event_loop
            .create_window(attrs)
            .expect("Failed to create window")
    })
}

/// Completes on the loop turn after its first poll. One await per rendered
/// frame is what paces the app future to the display.
pub(crate) async fn next_frame() {
    struct NextFrame(u64);

    impl Future for NextFrame {
        type Output = ();

        fn poll(self: Pin<&mut Self>, _: &mut TaskContext<'_>) -> Poll<()> {
            FRAME.with(|frame| {
                if frame.get() == self.0 {
                    Poll::Pending
                } else {
                    Poll::Ready(())
                }
            })
        }
    }

    NextFrame(FRAME.with(Cell::get)).await;
}

struct IosApp {
    // None once the future completes; the loop is asked to exit then, but
    // UIApplicationMain does not oblige, so this also guards re-polling.
    fut: Option<Pin<Box<dyn Future<Output = ()>>>>,
    // The future first runs in `resumed` — UIKit forbids UI work before the
    // application is active — and `about_to_wait` fires earlier than that.
    started: bool,
}

impl IosApp {
    fn poll(&mut self, event_loop: &ActiveEventLoop) {
        let Some(fut) = self.fut.as_mut() else {
            return;
        };
        ACTIVE.with(|active| active.set(ptr::from_ref(event_loop)));
        // A no-op waker: the loop re-polls every turn regardless, which is
        // exactly the pacing `next_frame` encodes.
        let mut cx = TaskContext::from_waker(Waker::noop());
        let done = fut.as_mut().poll(&mut cx).is_ready();
        ACTIVE.with(|active| active.set(ptr::null()));
        if done {
            self.fut = None;
            event_loop.exit();
        }
    }
}

impl ApplicationHandler for IosApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::Poll);
        self.started = true;
        self.poll(event_loop);
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: winit::event::WindowEvent,
    ) {
        super::wgpu_canvas::collect_window_event(window_id, event);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if !self.started {
            return;
        }
        FRAME.with(|frame| frame.set(frame.get() + 1));
        self.poll(event_loop);
    }
}

/// Runs a kiss3d app future under winit's iOS main loop. `#[kiss3d::main]`
/// expands to a call to this. In practice it never returns: once started,
/// `UIApplicationMain` owns the process.
pub fn run_ios(fut: impl Future<Output = ()> + 'static) {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut app = IosApp {
        fut: Some(Box::pin(fut)),
        started: false,
    };
    let _ = event_loop.run_app(&mut app);
}

/// Text produced by the system keyboard, drained by `poll_events` into the
/// regular event stream — typed text reaches the app the same way desktop
/// typing does.
pub(crate) enum TextEvent {
    Char(char),
    Backspace,
}

thread_local! {
    static TEXT_EVENTS: std::cell::RefCell<Vec<TextEvent>> =
        const { std::cell::RefCell::new(Vec::new()) };
    // The hidden UIKeyInput view, created on the first keyboard request and
    // kept attached: becoming/resigning first responder is what shows and
    // hides the keyboard.
    static KEY_VIEW: std::cell::RefCell<Option<Retained<KeyInputView>>> =
        const { std::cell::RefCell::new(None) };
}

pub(crate) fn take_text_events() -> Vec<TextEvent> {
    TEXT_EVENTS.with(|events| events.borrow_mut().drain(..).collect())
}

// winit has no IME support on iOS, so the keyboard is summoned the way UIKit
// intends: only a first responder that adopts UIKeyInput gets one, so a
// zero-sized subview adopts it and the system delivers typed text to
// `insertText:`. The view must not be `hidden` — a hidden view is refused
// first-responder status — but at zero size it draws nothing.
define_class!(
    #[unsafe(super(UIView))]
    #[thread_kind = MainThreadOnly]
    #[name = "Kiss3dKeyInputView"]
    pub(crate) struct KeyInputView;

    /// UIResponder override: without it the keyboard can never appear.
    impl KeyInputView {
        #[unsafe(method(canBecomeFirstResponder))]
        fn can_become_first_responder(&self) -> bool {
            true
        }
    }

    unsafe impl NSObjectProtocol for KeyInputView {}

    unsafe impl UITextInputTraits for KeyInputView {}

    unsafe impl UIKeyInput for KeyInputView {
        #[unsafe(method(hasText))]
        fn has_text(&self) -> bool {
            true
        }

        #[unsafe(method(insertText:))]
        fn insert_text(&self, text: &NSString) {
            TEXT_EVENTS.with(|events| {
                let mut events = events.borrow_mut();
                for c in text.to_string().chars() {
                    events.push(TextEvent::Char(c));
                }
            });
        }

        #[unsafe(method(deleteBackward))]
        fn delete_backward(&self) {
            TEXT_EVENTS.with(|events| events.borrow_mut().push(TextEvent::Backspace));
        }
    }
);

/// Show or hide the system keyboard for `window`.
pub(crate) fn set_keyboard_visible(window: &Window, visible: bool) {
    use wgpu::rwh::{HasWindowHandle, RawWindowHandle};

    // Everything here is UIKit: reachable only from the main thread, which
    // is the only thread winit callbacks (and therefore the app future) run
    // on. The guard is belt and braces, not a code path.
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let Ok(handle) = window.window_handle() else {
        return;
    };
    let RawWindowHandle::UiKit(ui_kit) = handle.as_raw() else {
        return;
    };
    // Valid while `window` is alive, which the borrow guarantees.
    let parent: &UIView = unsafe { ui_kit.ui_view.cast().as_ref() };

    KEY_VIEW.with(|cell| {
        let mut cell = cell.borrow_mut();
        if visible {
            let view = cell.get_or_insert_with(|| {
                let view: Retained<KeyInputView> =
                    unsafe { msg_send![KeyInputView::alloc(mtm), init] };
                parent.addSubview(&view);
                view
            });
            unsafe { view.becomeFirstResponder() };
        } else if let Some(view) = cell.as_ref() {
            unsafe { view.resignFirstResponder() };
        }
    });
}
