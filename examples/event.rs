extern mod kiss3d;
extern mod nalgebra;

use kiss3d::window;
use kiss3d::event;

#[start]
fn start(argc: int, argv: **u8, crate_map: *u8) -> int {
    std::rt::start_on_main_thread(argc, argv, crate_map, main)
}

fn main() {
    do window::Window::spawn("Kiss3d: events") |window| {
        do window.set_keyboard_callback |_, event| {
            match *event {
                event::KeyPressed(code) => {
                    println("You pressed the key with code: " + code.to_str());
                    println("Do not try to press escape: the callback returns `false` (does not propagate events)!");
                },
                event::KeyReleased(code) => {
                    println("You released the key with code: " + code.to_str());
                    println("Do not try to press escape: the callback returns `false` (does not propagate events)!");
                }
            }

            // Override the default keyboard handling: this will prevent the window from closing
            // when pressing `ESC`:
            false
        }

        do window.set_mouse_callback |_, event| {
            match *event {
                event::ButtonPressed(button, mods) => {
                    println("You pressed the mouse button with code: "      + button.to_str());
                    println("You pressed the mouse button with modifiers: " + mods.to_str());
                },
                event::ButtonReleased(button, mods) => {
                    println("You released the mouse button with code: "      + button.to_str());
                    println("You released the mouse button with modifiers: " + mods.to_str());
                },
                event::CursorPos(x, y) => {
                    println("Cursor pos: (" + x.to_str() + " , " + y.to_str() + ")");
                },
                event::Scroll(xshift, yshift) => {
                    println("Cursor pos: (" + xshift.to_str() + ", " + yshift.to_str() + ")");
                }
            }

            // Do not override the default mouse handling:
            true
        }

        do window.render_loop |_| {
        }
    }
}
