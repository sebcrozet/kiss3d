//! A resource manager to load materials.

use builtin::PlanarObjectMaterial;
use resource::PlanarMaterial;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

thread_local!(static KEY_MATERIAL_MANAGER: RefCell<PlanarMaterialManager> = RefCell::new(PlanarMaterialManager::new()));

/// The material manager.
///
/// Upon construction, it contains:
/// * the `object` material, used as the default to render objects.
/// * the `normals` material, used do display an object normals.
///
/// It keeps a cache of already-loaded materials. Note that this is only a cache, nothing more.
/// Thus, its usage is not required to load materials.
pub struct PlanarMaterialManager {
    default_material: Rc<RefCell<Box<PlanarMaterial + 'static>>>,
    materials: HashMap<String, Rc<RefCell<Box<PlanarMaterial + 'static>>>>,
}

impl PlanarMaterialManager {
    /// Creates a new material manager.
    pub fn new() -> PlanarMaterialManager {
        // load the default ObjectMaterial and the LineMaterial
        let mut materials = HashMap::new();

        let om = Rc::new(RefCell::new(
            Box::new(PlanarObjectMaterial::new()) as Box<PlanarMaterial + 'static>
        ));
        let _ = materials.insert("object".to_string(), om.clone());

        PlanarMaterialManager {
            default_material: om,
            materials: materials,
        }
    }

    /// Mutably applies a function to the material manager.
    pub fn get_global_manager<T, F: FnMut(&mut PlanarMaterialManager) -> T>(mut f: F) -> T {
        KEY_MATERIAL_MANAGER.with(|manager| f(&mut *manager.borrow_mut()))
    }

    /// Gets the default material to draw objects.
    pub fn get_default(&self) -> Rc<RefCell<Box<PlanarMaterial + 'static>>> {
        self.default_material.clone()
    }

    /// Get a material with the specified name. Returns `None` if the material is not registered.
    pub fn get(&mut self, name: &str) -> Option<Rc<RefCell<Box<PlanarMaterial + 'static>>>> {
        self.materials.get(&name.to_string()).map(|t| t.clone())
    }

    /// Adds a material with the specified name to this cache.
    pub fn add(&mut self, material: Rc<RefCell<Box<PlanarMaterial + 'static>>>, name: &str) {
        let _ = self.materials.insert(name.to_string(), material);
    }

    /// Removes a mesh from this cache.
    pub fn remove(&mut self, name: &str) {
        let _ = self.materials.remove(&name.to_string());
    }
}
