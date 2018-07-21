//! A resource manager to load materials.

use builtin::{NormalsMaterial, ObjectMaterial, UvsMaterial};
use resource::Material;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

thread_local!(static KEY_MATERIAL_MANAGER: RefCell<MaterialManager> = RefCell::new(MaterialManager::new()));

/// The material manager.
///
/// Upon construction, it contains:
/// * the `object` material, used as the default to render objects.
/// * the `normals` material, used do display an object normals.
///
/// It keeps a cache of already-loaded materials. Note that this is only a cache, nothing more.
/// Thus, its usage is not required to load materials.
pub struct MaterialManager {
    default_material: Rc<RefCell<Box<Material + 'static>>>,
    materials: HashMap<String, Rc<RefCell<Box<Material + 'static>>>>,
}

impl MaterialManager {
    /// Creates a new material manager.
    pub fn new() -> MaterialManager {
        // load the default ObjectMaterial and the LineMaterial
        let mut materials = HashMap::new();

        let om = Rc::new(RefCell::new(
            Box::new(ObjectMaterial::new()) as Box<Material + 'static>
        ));
        let _ = materials.insert("object".to_string(), om.clone());

        let nm = Rc::new(RefCell::new(
            Box::new(NormalsMaterial::new()) as Box<Material + 'static>
        ));
        let _ = materials.insert("normals".to_string(), nm.clone());

        let um = Rc::new(RefCell::new(
            Box::new(UvsMaterial::new()) as Box<Material + 'static>
        ));
        let _ = materials.insert("uvs".to_string(), um.clone());

        MaterialManager {
            default_material: om,
            materials: materials,
        }
    }

    /// Mutably applies a function to the material manager.
    pub fn get_global_manager<T, F: FnMut(&mut MaterialManager) -> T>(mut f: F) -> T {
        KEY_MATERIAL_MANAGER.with(|manager| f(&mut *manager.borrow_mut()))
    }

    /// Gets the default material to draw objects.
    pub fn get_default(&self) -> Rc<RefCell<Box<Material + 'static>>> {
        self.default_material.clone()
    }

    /// Get a material with the specified name. Returns `None` if the material is not registered.
    pub fn get(&mut self, name: &str) -> Option<Rc<RefCell<Box<Material + 'static>>>> {
        self.materials.get(&name.to_string()).map(|t| t.clone())
    }

    /// Adds a material with the specified name to this cache.
    pub fn add(&mut self, material: Rc<RefCell<Box<Material + 'static>>>, name: &str) {
        let _ = self.materials.insert(name.to_string(), material);
    }

    /// Removes a mesh from this cache.
    pub fn remove(&mut self, name: &str) {
        let _ = self.materials.remove(&name.to_string());
    }
}
