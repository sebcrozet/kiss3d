extern mod kiss3d;
extern mod nalgebra;

use kiss3d::window::Window;
use kiss3d::event;

#[start]
fn start(argc: int, argv: **u8) -> int {
    std::rt::start_on_main_thread(argc, argv, main)
}

fn main() {
    do Window::spawn("Kiss3d: events") |window| {
        do window.render_loop |w| {
            w.poll_events(event_handler);
        }
    }
}

fn event_handler(_: &mut Window, event: &event::Event) -> bool {
    match *event {
        event::KeyPressed(code) => {
            println("You pressed the key with code: " + code.to_str());
            println("Do not try to press escape: the callback returns `false` (does not propagate events)!");
            false // override the default keyboard handler
        },
        event::KeyReleased(code) => {
            println("You released the key with code: " + code.to_str());
            println("Do not try to press escape: the callback returns `false` (does not propagate events)!");
            false // override the default keyboard handler

        },
        event::ButtonPressed(button, mods) => {
            println("You pressed the mouse button with code: "      + button.to_str());
            println("You pressed the mouse button with modifiers: " + mods.to_str());
            true // dont override the default mouse handler

        },
        event::ButtonReleased(button, mods) => {
            println("You released the mouse button with code: "      + button.to_str());
            println("You released the mouse button with modifiers: " + mods.to_str());
            true // dont override the default mouse handler
        },
        event::CursorPos(x, y) => {
            println("Cursor pos: (" + x.to_str() + " , " + y.to_str() + ")");
            true // dont override the default mouse handler
        },
        event::Scroll(xshift, yshift) => {
            println("Cursor pos: (" + xshift.to_str() + ", " + yshift.to_str() + ")");
            true // dont override the default mouse handler
        },
        _ => true
    }
}
