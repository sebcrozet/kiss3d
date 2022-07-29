extern crate kiss3d;
extern crate nalgebra as na;

use kiss3d::window::Window;
use kiss3d::{light::Light, resource::TextureManager};
use na::{Translation3, UnitQuaternion, Vector3};
use std::path::Path;

fn main() {
    let mut window = Window::new("Kiss3d: texturing-mipmaps");
    let tex_path = Path::new("./examples/media/checkerboard.png");

    // Show two rotating quads without mipmaps, and the same quads with mipmaps below.
    let mut c1 = window.add_quad(0.2, 0.2, 1, 1);
    c1.set_texture_from_file(tex_path, "no-mipmaps");
    c1.set_local_translation(Translation3::new(-0.2, 0.2, 0.0));

    let mut c2 = window.add_quad(0.1, 0.1, 1, 1);
    c2.set_texture_from_file(tex_path, "no-mipmaps");
    c2.set_local_translation(Translation3::new(0.2, 0.2, 0.0));

    TextureManager::get_global_manager(|tm| tm.set_generate_mipmaps(true));

    let mut c3 = window.add_quad(0.2, 0.2, 1, 1);
    c3.set_texture_from_file(tex_path, "with-mipmaps");
    c3.set_local_translation(Translation3::new(-0.2, -0.2, 0.0));

    let mut c4 = window.add_quad(0.1, 0.1, 1, 1);
    c4.set_texture_from_file(tex_path, "with-mipmaps");
    c4.set_local_translation(Translation3::new(0.2, -0.2, 0.0));

    window.set_light(Light::StickToCamera);

    let rot3d = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.014);

    while window.render() {
        for c in [&mut c1, &mut c2, &mut c3, &mut c4] {
            c.append_rotation_wrt_center(&rot3d);
        }
    }
}
