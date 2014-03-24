//! Builtin mesh loader.

use std::cell::RefCell;
use std::rc::Rc;
use collections::HashMap;
use loader::obj;
use builtin::cube_obj;
use builtin::sphere_obj;
use builtin::cone_obj;
use builtin::cylinder_obj;
use resource::Mesh;

#[path = "../error.rs"]
mod error;

/// Loads the built-in meshes for the: cube, sphere, cone, cylinder and capsule.
pub fn load() -> HashMap<~str, Rc<RefCell<Mesh>>> {
    let emptypath = Path::new("");

    // load
    let m_cube     = obj::parse(cube_obj::CUBE_OBJ, &emptypath, "cube").move_iter().next().unwrap().val1();
    let m_sphere   = obj::parse(sphere_obj::SPHERE_OBJ, &emptypath, "sphere").move_iter().next().unwrap().val1();
    let m_cone     = obj::parse(cone_obj::CONE_OBJ, &emptypath, "cone").move_iter().next().unwrap().val1();
    let m_cylinder = obj::parse(cylinder_obj::CYLINDER_OBJ, &emptypath, "cylinder").move_iter().next().unwrap().val1();

    // register draw informations
    let mut hmap = HashMap::new();

    hmap.insert(~"cube", Rc::new(RefCell::new(m_cube)));
    hmap.insert(~"sphere", Rc::new(RefCell::new(m_sphere)));
    hmap.insert(~"cone", Rc::new(RefCell::new(m_cone)));
    hmap.insert(~"cylinder", Rc::new(RefCell::new(m_cylinder)));

    hmap
}
