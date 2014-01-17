use std::hashmap::HashMap;
use std::cell::RefCell;
use std::rc::Rc;
use gl;
use resources::shader_manager::ObjectShaderContext;
use obj;
use builtins::cube_obj;
use builtins::sphere_obj;
use builtins::cone_obj;
use builtins::cylinder_obj;
use builtins::capsule_obj;
use mesh::Mesh;

#[path = "../error.rs"]
mod error;

pub fn load(ctxt: &ObjectShaderContext) -> HashMap<~str, Rc<RefCell<Mesh>>> {
    unsafe {
        // create white texture
        // Black/white checkerboard
        verify!(gl::Uniform1i(ctxt.tex, 0));

        parse_builtins()
    }
}

fn parse_builtins() -> HashMap<~str, Rc<RefCell<Mesh>>> {
    let emptypath = Path::new("");

    // load
    let m_cube     = obj::parse(cube_obj::CUBE_OBJ, &emptypath, "cube")[0].n1();
    let m_sphere   = obj::parse(sphere_obj::SPHERE_OBJ, &emptypath, "sphere")[0].n1();
    let m_cone     = obj::parse(cone_obj::CONE_OBJ, &emptypath, "cone")[0].n1();
    let m_cylinder = obj::parse(cylinder_obj::CYLINDER_OBJ, &emptypath, "cylinder")[0].n1();
    let m_capsule  = obj::parse(capsule_obj::CAPSULE_OBJ, &emptypath, "capsule")[0].n1();

    // register draw informations
    let mut hmap = HashMap::new();

    hmap.insert(~"cube", Rc::new(RefCell::new(m_cube)));
    hmap.insert(~"sphere", Rc::new(RefCell::new(m_sphere)));
    hmap.insert(~"cone", Rc::new(RefCell::new(m_cone)));
    hmap.insert(~"cylinder", Rc::new(RefCell::new(m_cylinder)));
    hmap.insert(~"capsule", Rc::new(RefCell::new(m_capsule)));

    hmap
}
