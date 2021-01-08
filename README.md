# Kiss3d

Keep It Simple, Stupid 3d graphics engine.

This library is born from the frustration that today’s 3D
graphics library are either:

* **Too low level**: you have to write your own shaders and opening a
  window takes 8 hours, 300 lines of code and 10L of coffee.
* High level, but **too hard to understand/use**: these libraries are made to
  create beautiful photoreal (or close to it) animations or games.
  They have many features; too many, in fact, if you just want to draw a few objects
  on the screen with as little friction as possible.

**kiss3d** is not designed to be feature-complete or fast.
It is designed to let you draw simple geometric figures and play with them
with as little friction as possible.

An online version of this documentation is available [here](http://kiss3d.org).

## Features

* WASM compatible.
* Out of the box, open a window with a default arc-ball camera and a point light.
* First-person camera available as well, and user-defined cameras are possible.
* Render boxes, spheres, cones, cylinders, quads and lines simply
* Change an object's color or texture.
* Change an object's transform (we use [nalgebra](http://nalgebra.org) to do that).
* Create basic post-processing effects.

As an example, creating a scene with a red, rotating cube with a light attached
to the camera is as simple as (NOTE: this will **not** compile when targeting WASM):

```rust
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

Making the same example compatible with both WASM and native platforms is slightly more complicated because **kiss3d** must control the render loop:

```rust
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

```
[dependencies]
kiss3d = "0.29"
```


## Contributions
I’d love to see people improving this library for their own needs. However, keep in mind that
**kiss3d** is KISS. One-liner features (from the user point of view) are preferred.

## Acknowledgements

Thanks to all the Rustaceans for their help, and their OpenGL bindings.
