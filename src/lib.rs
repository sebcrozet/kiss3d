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

**kiss3d** is not designed to be feature-complete or fast.
It is designed to be able to draw simple geometric figures and play with them
with one-liners.

An on-line version of this documentation is available [here](http://kiss3d.org).

## Features
Most features are one-liners.

* WASM compatibility.
* open a window with a default arc-ball camera and a point light.
* a first-person camera is available too and user-defined cameras are possible.
* display boxes, spheres, cones, cylinders, quads and lines.
* change an object color or texture.
* change an object transform (we use the [nalgebra](http://nalgebra.org) library
  to do that).
* create basic post-processing effects.

As an example, having a red, rotating cube with the light attached to the camera is as simple as (NOTE: this will **not** compile when targeting WASM):

```no_run
extern crate kiss3d;
extern crate nalgebra as na;

use na::{Vector3, UnitQuaternion};
use kiss3d::window::Window;
use kiss3d::light::Light;

fn main() {
    let mut window = Window::new("Kiss3d: cube");
    let mut c      = window.add_cube(1.0, 1.0, 1.0);

    c.set_color(1.0, 0.0, 0.0);

    window.set_light(Light::StickToCamera);

    let rot = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.014);

    while window.render() {
        c.prepend_to_local_rotation(&rot);
    }
}
```

The same example, but that will compile for both WASM and native platforms is slightly more complicated because **kiss3d** must control the render loop:

```no_run
extern crate kiss3d;
extern crate nalgebra as na;

use kiss3d::light::Light;
use kiss3d::scene::SceneNode;
use kiss3d::window::{State, Window};
use na::{UnitQuaternion, Vector3};

struct AppState {
    c: SceneNode,
    rot: UnitQuaternion<f32>,
}

impl State for AppState {
    fn step(&mut self, _: &mut Window) {
        self.c.prepend_to_local_rotation(&self.rot)
    }
}

fn main() {
    let mut window = Window::new("Kiss3d: wasm example");
    let mut c = window.add_cube(1.0, 1.0, 1.0);

    c.set_color(1.0, 0.0, 0.0);

    window.set_light(Light::StickToCamera);

    let rot = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.014);
    let state = AppState { c, rot };

    window.render_loop(state)
}
```

Some controls are handled by default by the engine (they can be overridden by the user):

* `scroll`: zoom in / zoom out.
* `left click + drag`: look around.
* `right click + drag`: translate the view point.
* `enter`: look at the origin (0.0, 0.0, 0.0).

## Compilation
You will need the last stable build of the [rust compiler](http://www.rust-lang.org)
and the official package manager: [cargo](https://github.com/rust-lang/cargo).

Simply add the following to your `Cargo.toml` file:

```text
[dependencies]
kiss3d = "0.16"
```

## Contributions
I’d love to see people improving this library for their own needs. However, keep in mind that
**kiss3d** is KISS. One-liner features (from the user point of view) are preferred.

## Acknowledgements

Thanks to all the Rustaceans for their help, and their OpenGL bindings.
*/

#![deny(non_camel_case_types)]
#![deny(unused_parens)]
#![deny(non_upper_case_globals)]
#![deny(unused_qualifications)]
#![warn(missing_docs)] // FIXME: should be denied.
#![warn(unused_results)]
#![allow(unused_unsafe)] // FIXME: should be denied
#![allow(missing_copy_implementations)]
#![doc(html_root_url = "http://kiss3d.org/doc")]

#[macro_use]
extern crate bitflags;
extern crate rusttype;
// extern crate glfw;
extern crate image;
// extern crate libc;
extern crate nalgebra as na;
extern crate ncollide3d;
extern crate num_traits as num;
#[macro_use]
extern crate serde_derive;
extern crate serde;

#[cfg(not(any(target_arch = "wasm32", target_arch = "asmjs")))]
extern crate gl;
#[cfg(not(any(target_arch = "wasm32", target_arch = "asmjs")))]
extern crate glutin;
#[cfg(any(target_arch = "wasm32", target_arch = "asmjs"))]
#[macro_use]
extern crate stdweb;
#[cfg(any(target_arch = "wasm32", target_arch = "asmjs"))]
#[macro_use]
extern crate stdweb_derive;

pub mod builtin;
pub mod camera;
pub mod context;
mod error;
pub mod event;
pub mod light;
pub mod line_renderer;
pub mod loader;
pub mod planar_camera;
pub mod planar_line_renderer;
pub mod point_renderer;
pub mod post_processing;
pub mod resource;
pub mod scene;
pub mod text;
pub mod window;
