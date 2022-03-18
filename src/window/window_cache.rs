use std::{cell::RefCell, mem::take};

use crate::resource::{MaterialManager, MeshManager, TextureManager};

#[derive(Default)]
/// Globally accessible cache of objects
pub(crate) struct WindowCache {
    pub(crate) mesh_manager: Option<MeshManager>,
    pub(crate) texture_manager: Option<TextureManager>,
    pub(crate) material_manager: Option<MaterialManager>,
}

thread_local!(pub(crate) static WINDOW_CACHE: RefCell<WindowCache>  = RefCell::new(WindowCache::default()));

impl WindowCache {
    /// Initialize resource managers
    pub fn populate() {
        WINDOW_CACHE.with(|cache| {
            cache.borrow_mut().mesh_manager = Some(MeshManager::new());
            cache.borrow_mut().texture_manager = Some(TextureManager::new());
            cache.borrow_mut().material_manager = Some(MaterialManager::new());
        });
    }

    /// Clear the cache dropping all resources
    #[allow(unused_results)]
    pub fn clear() {
        WINDOW_CACHE.with(|cache| take(&mut *cache.borrow_mut()));
    }
}
