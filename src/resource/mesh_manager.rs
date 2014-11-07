//! A resource manager to load meshes.

use std::io::IoResult;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use na::{Pnt3, Vec3};
use ncollide::procedural::TriMesh;
use ncollide::procedural;
use resource::Mesh;
use loader::obj;
use loader::mtl::MtlMaterial;

local_data_key!(KEY_MESH_MANAGER: RefCell<MeshManager>)

/// The mesh manager.
///
/// Upon construction, it contains:
///
/// It keeps a cache of already-loaded meshes. Note that this is only a cache, nothing more.
/// Thus, its usage is not required to load meshes.
pub struct MeshManager {
    meshes:       HashMap<String, Rc<RefCell<Mesh>>>
}

impl MeshManager {
    /// Creates a new mesh manager.
    pub fn new() -> MeshManager {
        let mut res = MeshManager {
            meshes: HashMap::new()
        };

        let _ = res.add_trimesh(procedural::unit_sphere(50, 50, true), false, "sphere");
        let _ = res.add_trimesh(procedural::unit_cuboid(), false, "cube");
        let _ = res.add_trimesh(procedural::unit_cone(50), false, "cone");
        let _ = res.add_trimesh(procedural::unit_cylinder(50), false, "cylinder");

        res
    }

    /// Mutably applies a function to the mesh manager.
    pub fn get_global_manager<T>(f: |&mut MeshManager| -> T) -> T {
        if KEY_MESH_MANAGER.get().is_none() {
            let _ = KEY_MESH_MANAGER.replace(Some(RefCell::new(MeshManager::new())));
        }

        f(KEY_MESH_MANAGER.get().unwrap().borrow_mut().deref_mut())
    }

    /// Get a mesh with the specified name. Returns `None` if the mesh is not registered.
    pub fn get(&mut self, name: &str) -> Option<Rc<RefCell<Mesh>>> {
        self.meshes.get(&name.to_string()).map(|t| t.clone())
    }

    /// Adds a mesh with the specified name to this cache.
    pub fn add(&mut self, mesh: Rc<RefCell<Mesh>>, name: &str) {
        let _ = self.meshes.insert(name.to_string(), mesh);
    }

    /// Adds a mesh with the specified mesh descriptor and name.
    pub fn add_trimesh(&mut self, descr: TriMesh<f32, Pnt3<f32>, Vec3<f32>>, dynamic_draw: bool, name: &str) -> Rc<RefCell<Mesh>> {
        let mesh = Mesh::from_trimesh(descr, dynamic_draw);
        let mesh = Rc::new(RefCell::new(mesh));

        self.add(mesh.clone(), name);

        mesh
    }

    /// Removes a mesh from this cache.
    pub fn remove(&mut self, name: &str) {
        let _ = self.meshes.remove(&name.to_string());
    }

    // FIXME: is this the right place to put this?
    /// Loads the meshes described by an obj file.
    pub fn load_obj(path: &Path, mtl_dir: &Path, geometry_name: &str)
                    -> IoResult<Vec<(String, Rc<RefCell<Mesh>>, Option<MtlMaterial>)>> {
        obj::parse_file(path, mtl_dir, geometry_name).map(|ms| {
            let mut res = Vec::new();

            for (n, m, mat) in ms.into_iter() {
                let m = Rc::new(RefCell::new(m));

                res.push((n, m, mat));
            }

            res
        })
    }
}
