extern crate glfw;
extern crate kiss3d;
extern crate nalgebra as na;

use glfw::{Action, WindowEvent};
use kiss3d::window::Window;

fn main() {
    let mut window = Window::new("Kiss3d: events");

    while window.render() {
        for mut event in window.events().iter() {
            match event.value {
                WindowEvent::Key(code, _, Action::Press, _) => {
                    println!("You pressed the key with code: {:?}", code);
                    println!("Do not try to press escape: the event is inhibited!");
                    event.inhibited = true // override the default keyboard handler
                },
                WindowEvent::Key(code, _, Action::Release, _) => {
                    println!("You released the key with code: {:?}", code);
                    println!("Do not try to press escape: the event is inhibited!");
                    event.inhibited = true // override the default keyboard handler
                },
                WindowEvent::MouseButton(button, Action::Press, mods) => {
                    println!("You pressed the mouse button with code: {:?}", button);
                    println!("You pressed the mouse button with modifiers: {:?}", mods);
                    // dont override the default mouse handler
                },
                WindowEvent::MouseButton(button, Action::Release, mods) => {
                    println!("You released the mouse button with code: {:?}", button);
                    println!("You released the mouse button with modifiers: {:?}", mods);
                    // dont override the default mouse handler
                },
                WindowEvent::CursorPos(x, y) => {
                    println!("Cursor pos: ({} , {})", x, y);
                    // dont override the default mouse handler
                },
                WindowEvent::Scroll(xshift, yshift) => {
                    println!("Cursor pos: ({} , {})", xshift, yshift);
                    // dont override the default mouse handler
                },
                _ => { }
            }
        }
    }
}
