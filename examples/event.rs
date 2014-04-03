extern crate native;
extern crate glfw;
extern crate kiss3d;
extern crate nalgebra;

use kiss3d::window::Window;

#[start]
fn start(argc: int, argv: **u8) -> int {
    native::start(argc, argv, main)
}

fn main() {
    Window::spawn("Kiss3d: events", |window| {
        window.render_loop(|w| {
            w.poll_events(event_handler);
        })
    })
}

fn event_handler(_: &mut Window, event: &glfw::WindowEvent) -> bool {
    match *event {
        glfw::KeyEvent(code, _, glfw::Press, _) => {
            println!("You pressed the key with code: {:?}", code);
            println!("Do not try to press escape: the callback returns `false` (does not propagate events)!");
            false // override the default keyboard handler
        },
        glfw::KeyEvent(code, _, glfw::Release, _) => {
            println!("You released the key with code: {:?}", code);
            println!("Do not try to press escape: the callback returns `false` (does not propagate events)!");
            false // override the default keyboard handler
        },
        glfw::MouseButtonEvent(button, glfw::Press, mods) => {
            println!("You pressed the mouse button with code: {:?}", button);
            println!("You pressed the mouse button with modifiers: {:?}", mods);
            true // dont override the default mouse handler
        },
        glfw::MouseButtonEvent(button, glfw::Release, mods) => {
            println!("You released the mouse button with code: {:?}", button);
            println!("You released the mouse button with modifiers: {:?}", mods);
            true // dont override the default mouse handler
        },
        glfw::CursorPosEvent(x, y) => {
            println!("Cursor pos: ({} , {})", x, y);
            true // dont override the default mouse handler
        },
        glfw::ScrollEvent(xshift, yshift) => {
            println!("Cursor pos: ({} , {})", xshift, yshift);
            true // dont override the default mouse handler
        },
        _ => true
    }
}
