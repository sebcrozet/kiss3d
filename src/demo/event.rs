extern mod kiss3d;
extern mod nalgebra;

use kiss3d::window;
use kiss3d::event;

fn main()
{
  do window::Window::spawn(~"Kiss3d: events") |w|
  {
    do w.set_keyboard_callback |event|
    {
      match *event
      {
        event::KeyPressed(code) => {
          println("You pressed the key with code: " + code.to_str());
          println("Do not try to press escape: the callback returns `false` (does not propagate events)!");
        },
        event::KeyReleased(code) => {
          println("You released the key with code: " + code.to_str());
          println("Do not try to press escape: the callback returns `false` (does not propagate events)!");
        }
      }

      false // override the default keyboard handling
    }

    do w.set_mouse_callback |event|
    {
      match *event
      {
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

      true // do not override the default keyboard handling
    }
  }
}
