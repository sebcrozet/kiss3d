/*!
# Kiss3d

Keep It Simple, Stupid 3d graphics engine.

This library is born from the frustration in front of the fact that today’s 3D
graphics library are:

* either too low level: you have to write your own shaders and opening a
  window steals you 8 hours, 300 lines of code and 10L of coffee.
* or high level but too hard to understand/use: those are libraries made to
  write beautiful animations or games. They have a lot of feature; too much
  feature if you only want to draw a few geometries on the screen.

**Kiss3d** is not designed to be feature-complete or fast.
It is designed to be able to draw simple geometric figures and play with them
with one-liners.

## Features
Most features are one-liners.

* open a window with a default arc-ball camera and a point light.
* a first-person camera is available too and user-defined cameras are possible.
* display boxes, spheres, cones, cylinders, quads and lines.
* change an object color or texture.
* change an object transform (we use the [nalgebra](http://nalgebra.org) library
  to do that).  An object cannot be scaled though.
* create basic post-processing effects.

As an example, having a red, rotating cube with the light attached to the camera is as simple as:

```no_run
extern crate kiss3d;
extern crate nalgebra as na;

use na::Vec3;
use kiss3d::window::Window;
use kiss3d::light::Light;

fn main() {
    let mut window = Window::new("Kiss3d: cube");
    let mut c      = window.add_cube(1.0, 1.0, 1.0);

    c.set_color(1.0, 0.0, 0.0);

    window.set_light(Light::StickToCamera);

    while window.render() {
        c.prepend_to_local_rotation(&Vec3::new(0.0f32, 0.014, 0.0));
    }
}
```

Some controls are handled by default by the engine (they can be overridden by the user):

* `scroll`: zoom in / zoom out.
* `left click + drag`: look around.
* `right click + drag`: translate the view point.
* `enter`: look at the origin (0.0, 0.0, 0.0).

## Compilation
You will need the last nightly build of the [rust compiler](http://www.rust-lang.org)
and the official package manager: [cargo](https://github.com/rust-lang/cargo).

Simply add the following to your `Cargo.toml` file:

```text
[dependencies.kiss3d]
git = "https://github.com/sebcrozet/kiss3d"
```

## Contributions
I’d love to see people improving this library for their own needs. However, keep in mind that
**Kiss3d** is KISS. One-liner features (from the user point of view) are preferred.
*/

#![deny(non_camel_case_types)]
#![deny(unused_parens)]
#![deny(non_upper_case_globals)]
#![deny(unused_qualifications)]
#![warn(missing_docs)] // FIXME: should be denied.
#![deny(unused_results)]
#![allow(unused_unsafe)] // FIXME: should be denied
#![allow(missing_copy_implementations)]
#![doc(html_root_url = "http://kiss3d.org/doc")]

extern crate libc;
extern crate time;
extern crate gl;
extern crate num;
extern crate nalgebra as na;
extern crate ncollide_procedural;
extern crate image;
extern crate freetype;
extern crate glfw;

mod error;
pub mod window;
pub mod scene;
pub mod camera;
pub mod light;
pub mod loader;
pub mod line_renderer;
pub mod point_renderer;
pub mod builtin;
pub mod post_processing;
pub mod resource;
pub mod text;
