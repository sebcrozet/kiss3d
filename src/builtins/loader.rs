use std::sys;
use std::cast;
use std::ptr;
use std::hashmap::HashMap;
use extra::rc::Rc;
use gl;
use gl::types::*;
use resources::shaders_manager::ObjectShaderContext;
use obj;
use builtins::cube_obj;
use builtins::sphere_obj;
use builtins::cone_obj;
use builtins::cylinder_obj;
use builtins::capsule_obj;
use resources::textures_manager::TexturesManager;
use mesh::Mesh;

#[path = "../error.rs"]
mod error;

pub fn load(ctxt: &ObjectShaderContext, textures_manager: &mut TexturesManager) -> HashMap<~str, Rc<Mesh>> {
    unsafe {
        // create white texture
        // Black/white checkerboard
        let default_tex = textures_manager.add_empty("default");
        let default_tex_pixels: [ GLfloat, ..3 ] = [
            1.0, 1.0, 1.0
            ];

        verify!(gl::ActiveTexture(gl::TEXTURE0));
        verify!(gl::BindTexture(gl::TEXTURE_2D, default_tex.borrow().id()));
        verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_BASE_LEVEL, 0));
        verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, 0));
        verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32));
        verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32));
        verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32));
        verify!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32));
        verify!(gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as i32, 1, 1, 0, gl::RGB, gl::FLOAT,
                cast::transmute(&default_tex_pixels[0])));

        verify!(gl::Uniform1i(ctxt.tex, 0));

        parse_builtins()
    }
}

fn parse_builtins() -> HashMap<~str, Rc<Mesh>> {
    // load
    let m_cube     = obj::parse(cube_obj::CUBE_OBJ);
    let m_sphere   = obj::parse(sphere_obj::SPHERE_OBJ);
    let m_cone     = obj::parse(cone_obj::CONE_OBJ);
    let m_cylinder = obj::parse(cylinder_obj::CYLINDER_OBJ);
    let m_capsule  = obj::parse(capsule_obj::CAPSULE_OBJ);

    // register draw informations
    let mut hmap = HashMap::new();

    hmap.insert(~"cube", Rc::from_send(m_cube));
    hmap.insert(~"sphere", Rc::from_send(m_sphere));
    hmap.insert(~"cone", Rc::from_send(m_cone));
    hmap.insert(~"cylinder", Rc::from_send(m_cylinder));
    hmap.insert(~"capsule", Rc::from_send(m_capsule));

    hmap
}
