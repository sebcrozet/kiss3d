//! A resource manager to load meshes.

use resource::Mesh2;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Result as IoResult;
use std::path::Path;
use std::rc::Rc;

thread_local!(static KEY_MESH_MANAGER: RefCell<MeshManager2> = RefCell::new(MeshManager2::new()));

/// The mesh manager.
///
/// Upon construction, it contains:
///
/// It keeps a cache of already-loaded meshes. Note that this is only a cache, nothing more.
/// Thus, its usage is not required to load meshes.
pub struct MeshManager2 {
    meshes: HashMap<String, Rc<RefCell<Mesh2>>>,
}

impl MeshManager2 {
    /// Creates a new mesh manager.
    pub fn new() -> MeshManager2 {
        let mut res = MeshManager2 {
            meshes: HashMap::new(),
        };

        // let _ = res.add_trimesh(procedural::unit_sphere(50, 50, true), false, "sphere");
        // let _ = res.add_trimesh(procedural::unit_cuboid(), false, "cube");
        // let _ = res.add_trimesh(procedural::unit_cone(50), false, "cone");
        // let _ = res.add_trimesh(procedural::unit_cylinder(50), false, "cylinder");

        res
    }

    /// Mutably applies a function to the mesh manager.
    pub fn get_global_manager<T, F: FnMut(&mut MeshManager2) -> T>(mut f: F) -> T {
        KEY_MESH_MANAGER.with(|manager| f(&mut *manager.borrow_mut()))
    }

    /// Get a mesh with the specified name. Returns `None` if the mesh is not registered.
    pub fn get(&mut self, name: &str) -> Option<Rc<RefCell<Mesh2>>> {
        self.meshes.get(&name.to_string()).map(|t| t.clone())
    }

    /// Adds a mesh with the specified name to this cache.
    pub fn add(&mut self, mesh: Rc<RefCell<Mesh2>>, name: &str) {
        let _ = self.meshes.insert(name.to_string(), mesh);
    }

    /// Removes a mesh from this cache.
    pub fn remove(&mut self, name: &str) {
        let _ = self.meshes.remove(&name.to_string());
    }
}
