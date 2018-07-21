//! A resource manager to load meshes.
use std::f32;

use na::{Point2, Point3};

use resource::PlanarMesh;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

thread_local!(static KEY_MESH_MANAGER: RefCell<PlanarMeshManager2> = RefCell::new(PlanarMeshManager2::new()));

/// The mesh manager.
///
/// Upon construction, it contains:
///
/// It keeps a cache of already-loaded meshes. Note that this is only a cache, nothing more.
/// Thus, its usage is not required to load meshes.
pub struct PlanarMeshManager2 {
    meshes: HashMap<String, Rc<RefCell<PlanarMesh>>>,
}

impl PlanarMeshManager2 {
    /// Creates a new mesh manager.
    pub fn new() -> PlanarMeshManager2 {
        let mut res = PlanarMeshManager2 {
            meshes: HashMap::new(),
        };

        let rect_vtx = vec![
            Point2::new(0.5, 0.5),
            Point2::new(-0.5, -0.5),
            Point2::new(-0.5, 0.5),
            Point2::new(0.5, -0.5),
        ];

        let rect_ids = vec![Point3::new(0, 1, 2), Point3::new(1, 0, 3)];
        let rect = PlanarMesh::new(rect_vtx, rect_ids, None, false);
        res.add(Rc::new(RefCell::new(rect)), "rectangle");

        let mut circle_vtx = vec![Point2::origin()];
        let mut circle_ids = Vec::new();
        let nsamples = 50;

        for i in 0..nsamples {
            let ang = (i as f32) / (nsamples as f32) * f32::consts::PI * 2.0;
            circle_vtx.push(Point2::new(ang.cos(), ang.sin()) * 0.5);
            circle_ids.push(Point3::new(
                0,
                circle_vtx.len() as u16 - 2,
                circle_vtx.len() as u16 - 1,
            ));
        }
        circle_ids.push(Point3::new(0, circle_vtx.len() as u16 - 1, 1));

        let circle = PlanarMesh::new(circle_vtx, circle_ids, None, false);
        res.add(Rc::new(RefCell::new(circle)), "circle");

        res
    }

    /// Mutably applies a function to the mesh manager.
    pub fn get_global_manager<T, F: FnMut(&mut PlanarMeshManager2) -> T>(mut f: F) -> T {
        KEY_MESH_MANAGER.with(|manager| f(&mut *manager.borrow_mut()))
    }

    /// Get a mesh with the specified name. Returns `None` if the mesh is not registered.
    pub fn get(&mut self, name: &str) -> Option<Rc<RefCell<PlanarMesh>>> {
        self.meshes.get(&name.to_string()).map(|t| t.clone())
    }

    /// Adds a mesh with the specified name to this cache.
    pub fn add(&mut self, mesh: Rc<RefCell<PlanarMesh>>, name: &str) {
        let _ = self.meshes.insert(name.to_string(), mesh);
    }

    /// Removes a mesh from this cache.
    pub fn remove(&mut self, name: &str) {
        let _ = self.meshes.remove(&name.to_string());
    }
}
