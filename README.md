# kiss3d

Keep It Simple, Stupid 3d graphics engine.

This library is born from the frustration in front of the fact that today’s 3D
graphics library are:
  - either too low level: you have to write your own shaders and opening a
    window steals you 8 hours, 300 lines of code and 10L of coffee.
  - or high level but too hard to understand/use: those are libraries made to
    write beautiful animations or games. They have a lot of feature; too much
    feature if you only want to draw a few geometries on the screen.

**kiss3d** is not designed to be feature-complete or fast.
It is designed to be able to draw simple geometric figures and play with them
with one-liners.

## Features
All features are one-liners.
  - open a window with a default arc-ball camera and a point light.
  - display boxes, spheres, cones or cylinders.
  - change an object color or texture.
  - change an object transform (we use the **nalgebra** library to do that).
    An object cannot be scaled though.

As an exemple, having a red, rotating cube with the light attached to the camera is as simple as:
```rust
extern mod kiss3d;
extern mod nalgebra;

use nalgebra::traits::rotation::Rotation;
use nalgebra::vec::Vec3;
use kiss3d::window;

fn main()
{
  do window::Window::spawn(~"Kiss3d: cube") |window|
  {
    let c = window.add_cube(1.0, 1.0, 1.0).set_color(1.0, 0.0, 0.0);

    do window.set_loop_callback
    { c.transformation().rotate_by(&Vec3::new(0.0f64, 0.014, 0.0)) }

    window.set_light(window::StickToCamera);
  }
}
```
## Compilation
You will need the last rust compiler from the `master` branch.
If you encounter problems, make sure you have the last compiler version before creating an issue.

The simplest way to build **kiss3d** and all its dependencies is to do a
recursive clone:


    git clone --recursive git://github.com/sebcrozet/kiss3d.git
    cd kiss3d
    make deps
    make
    make test


The last command will compile demos on the `bin` folder.
Use `make doc` to compile the documentation on the `doc` folder.

## Contributions
I intend to work on this library to suit my needs only (to write demo for my
physics engine **nphysics**).  Therefore, I’d love to see people improving this
library for their own needs.

However, keep in mind that kiss3d is KISS.
Only one-liner features (from the user point of view) are accepted (there might
be exceptions).

## Acknowledgements

I am more a physics guy than a graphics guy. I did not want to spend too much
time to be able to display things on the screen. Thus I thank:
  - **bjz** for its glfw binding and its demos
    (git://github.com/bjz/open.gl-tutorials.git) from which I took a great
    bunch of initialization code.
