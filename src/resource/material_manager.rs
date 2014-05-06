//! A resource manager to load materials.

use std::rc::Rc;
use std::cell::RefCell;
use std::local_data;
use collections::HashMap;
use resource::Material;
use builtin::{ObjectMaterial, NormalsMaterial};

local_data_key!(KEY_MATERIAL_MANAGER: MaterialManager)

/// The material manager.
///
/// Upon construction, it contains:
/// * the `object` material, used as the default to render objects.
/// * the `normals` material, used do display an object normals.
///
/// It keeps a cache of already-loaded materials. Note that this is only a cache, nothing more.
/// Thus, its usage is not required to load materials.
pub struct MaterialManager {
    default_material: Rc<RefCell<~Material:'static>>,
    materials:        HashMap<~str, Rc<RefCell<~Material:'static>>>
}

impl MaterialManager {
    /// Creates a new material manager.
    pub fn new() -> MaterialManager {
        // load the default ObjectMaterial and the LineMaterial
        let mut materials = HashMap::new();

        let om = Rc::new(RefCell::new(~ObjectMaterial::new() as ~Material:'static));
        materials.insert(~"object", om.clone());

        let nm = Rc::new(RefCell::new(~NormalsMaterial::new() as ~Material:'static));
        materials.insert(~"normals", nm.clone());

        MaterialManager {
            default_material: om,
            materials:        materials
        }
    }

    /// Mutably applies a function to the material manager.
    pub fn get_global_manager<T>(f: |&mut MaterialManager| -> T) -> T {
        if local_data::get(KEY_MATERIAL_MANAGER, |mm| mm.is_none()) {
            local_data::set(KEY_MATERIAL_MANAGER, MaterialManager::new())
        }

        local_data::get_mut(KEY_MATERIAL_MANAGER, |mm| f(mm.unwrap()))
    }

    /// Gets the default material to draw objects.
    pub fn get_default(&self) -> Rc<RefCell<~Material:'static>> {
        self.default_material.clone()
    }

    /// Get a material with the specified name. Returns `None` if the material is not registered.
    pub fn get(&mut self, name: &str) -> Option<Rc<RefCell<~Material:'static>>> {
        self.materials.find(&name.to_owned()).map(|t| t.clone())
    }

    /// Adds a material with the specified name to this cache.
    pub fn add(&mut self, material: Rc<RefCell<~Material:'static>>, name: &str) {
        let _ = self.materials.insert(name.to_owned(), material);
    }

    /// Removes a mesh from this cache.
    pub fn remove(&mut self, name: &str) {
        self.materials.remove(&name.to_owned());
    }
}
