extern crate kiss3d;
extern crate nalgebra as na;

use kiss3d::resource::{CubemapDirection, TextureManager};
use kiss3d::builtin::CubemapMaterial;
use kiss3d::resource::{Material};
use kiss3d::window::{State, Window};

use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

struct AppState {}
impl State for AppState {
    fn step(&mut self, _: &mut Window) {}
}

fn main() {
    let mut window = Window::new("Kiss3d: cube_map");
    let mut c = window.add_cube(400.0, 400.0, 400.0);

    let cubemap = TextureManager::get_global_manager(|tm| {
        tm.add_cubemap(
            [
                &Path::new("./examples/media/cubemap_positivex.png"),
                &Path::new("./examples/media/cubemap_negativex.png"),
                &Path::new("./examples/media/cubemap_positivey.png"),
                &Path::new("./examples/media/cubemap_negativey.png"),
                &Path::new("./examples/media/cubemap_positivez.png"),
                &Path::new("./examples/media/cubemap_negativez.png"),
            ],
            [
                CubemapDirection::PositiveX, // right
                CubemapDirection::NegativeX, // left
                CubemapDirection::PositiveY, // up
                CubemapDirection::NegativeY, // down
                CubemapDirection::PositiveZ, // front
                CubemapDirection::NegativeZ, // back
            ],
            "skybox",
        )
    });

    let material = Rc::new(RefCell::new(
        Box::new(CubemapMaterial::new(cubemap)) as Box<dyn Material + 'static>
    ));

    c.set_material(material);

    let state = AppState {};
    window.render_loop(state);
}
