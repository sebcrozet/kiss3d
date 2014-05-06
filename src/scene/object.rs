//! Data structure of a scene node.

use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
use gl::types::*;
use nalgebra::na::{Vec2, Vec3, Iso3};
use resource::{Texture, TextureManager, Material, Mesh};
use camera::Camera;
use light::Light;

#[path = "../error.rs"]
mod error;

/// Set of data identifying a scene node.
pub struct ObjectData {
    material:     Rc<RefCell<~Material:'static>>,
    texture:      Rc<Texture>,
    color:        Vec3<f32>,
    wlines:       f32,
    wpoints:      f32,
    draw_surface: bool,
    cull:         bool,
    user_data:    ~Any
}

impl ObjectData {
    /// The texture of this object.
    #[inline]
    pub fn texture<'a>(&'a self) -> &'a Rc<Texture> {
        &'a self.texture
    }

    /// The color of this object.
    #[inline]
    pub fn color<'a>(&'a self) -> &'a Vec3<f32> {
        &'a self.color
    }

    /// The width of the lines draw for this object.
    #[inline]
    pub fn lines_width(&self) -> f32 {
        self.wlines
    }

    /// The size of the points draw for this object.
    #[inline]
    pub fn points_size(&self) -> f32 {
        self.wpoints
    }

    /// Whether this object has its surface rendered or not.
    #[inline]
    pub fn surface_rendering_active(&self) -> bool {
        self.draw_surface
    }

    /// Whether this object uses backface culling or not.
    #[inline]
    pub fn backface_culling_enabled(&self) -> bool {
        self.cull
    }

    /// An user-defined data.
    ///
    /// Use dynamic typing capabilities of the `Any` type to recover the actual data.
    #[inline]
    pub fn user_data<'a>(&'a self) -> &'a Any {
        let res: &'a Any = self.user_data;

        res
    }
}

/// A 3d objects on the scene.
///
/// This is the only interface to manipulate the object position, color, vertices and texture.
pub struct Object {
    // FIXME: should Mesh and Object be merged?
    // (thus removing the need of ObjectData at all.)
    data:     ObjectData,
    mesh:     Rc<RefCell<Mesh>>
}

impl Object {
    #[doc(hidden)]
    pub fn new(mesh:         Rc<RefCell<Mesh>>,
               r:            f32,
               g:            f32,
               b:            f32,
               texture:      Rc<Texture>,
               material:     Rc<RefCell<~Material:'static>>) -> Object {
        let data = ObjectData {
            color:        Vec3::new(r, g, b),
            texture:      texture,
            wlines:       0.0,
            wpoints:      0.0,
            draw_surface: true,
            cull:         true,
            material:     material,
            user_data:    ~() as ~Any
        };

        Object {
            data:     data,
            mesh:     mesh
        }
    }

    #[doc(hidden)]
    pub fn render(&self,
                  transform: &Iso3<f32>,
                  scale:     &Vec3<f32>,
                  pass:      uint,
                  camera:    &mut Camera,
                  light:     &Light) {
        self.data.material.borrow_mut().render(
            pass,
            transform,
            scale,
            camera,
            light,
            &self.data,
            self.mesh.borrow_mut().deref_mut());
    }

    /// Gets the data of this object.
    #[inline]
    pub fn data<'a>(&'a self) -> &'a ObjectData {
        &'a self.data
    }

    /// Gets the data of this object.
    #[inline]
    pub fn data_mut<'a>(&'a mut self) -> &'a mut ObjectData {
        &'a mut self.data
    }

    /// Enables or disables backface culling for this object.
    #[inline]
    pub fn enable_backface_culling(&mut self, active: bool) {
        self.data.cull = active;
    }

    /// Attaches user-defined data to this object.
    #[inline]
    pub fn set_user_data(&mut self, user_data: ~Any) {
        self.data.user_data = user_data;
    }

    /// Gets the material of this object.
    #[inline]
    pub fn material(&self) -> Rc<RefCell<~Material:'static>> {
        self.data.material.clone()
    }

    /// Sets the material of this object.
    #[inline]
    pub fn set_material(&mut self, material: Rc<RefCell<~Material:'static>>) {
        self.data.material = material;
    }

    /// Sets the width of the lines drawn for this object.
    #[inline]
    pub fn set_lines_width(&mut self, width: f32) {
        self.data.wlines = width
    }

    /// Returns the width of the lines drawn for this object.
    #[inline]
    pub fn lines_width(&self) -> f32 {
        self.data.wlines
    }

    /// Sets the size of the points drawn for this object.
    #[inline]
    pub fn set_points_size(&mut self, size: f32) {
        self.data.wpoints = size
    }

    /// Returns the size of the points drawn for this object.
    #[inline]
    pub fn points_size(&self) -> f32 {
        self.data.wpoints
    }

    /// Activate or deactivate the rendering of this object surface.
    #[inline]
    pub fn set_surface_rendering_activation(&mut self, active: bool) {
        self.data.draw_surface = active
    }

    /// Activate or deactivate the rendering of this object surface.
    #[inline]
    pub fn surface_rendering_activation(&self) -> bool {
        self.data.draw_surface
    }

    /// This object's mesh.
    #[inline]
    pub fn mesh<'a>(&'a self) -> &'a Rc<RefCell<Mesh>> {
        &'a self.mesh
    }

    /// Mutably access the object's vertices.
    #[inline(always)]
    pub fn modify_vertices(&mut self, f: &mut |&mut Vec<Vec3<GLfloat>>| -> ()) {
        let _ = self.mesh.borrow_mut().coords().write().data_mut().as_mut().map(|coords| (*f)(coords));
    }

    /// Access the object's vertices.
    #[inline(always)]
    pub fn read_vertices(&self, f: &mut |&[Vec3<GLfloat>]| -> ()) {
        let _ = self.mesh.borrow().coords().read().data().as_ref().map(|coords| (*f)(coords.as_slice()));
    }

    /// Recomputes the normals of this object's mesh.
    #[inline]
    pub fn recompute_normals(&mut self) {
        self.mesh.borrow_mut().recompute_normals();
    }

    /// Mutably access the object's normals.
    #[inline(always)]
    pub fn modify_normals(&mut self, f: &mut |&mut Vec<Vec3<GLfloat>>| -> ()) {
        let _ = self.mesh.borrow_mut().normals().write().data_mut().as_mut().map(|normals| (*f)(normals));
    }

    /// Access the object's normals.
    #[inline(always)]
    pub fn read_normals(&self, f: &mut |&[Vec3<GLfloat>]| -> ()) {
        let _ = self.mesh.borrow().normals().read().data().as_ref().map(|normals| (*f)(normals.as_slice()));
    }

    /// Mutably access the object's faces.
    #[inline(always)]
    pub fn modify_faces(&mut self, f: &mut |&mut Vec<Vec3<GLuint>>| -> ()) {
        let _ = self.mesh.borrow_mut().faces().write().data_mut().as_mut().map(|faces| (*f)(faces));
    }

    /// Access the object's faces.
    #[inline(always)]
    pub fn read_faces(&self, f: &mut |&[Vec3<GLuint>]| -> ()) {
        let _ = self.mesh.borrow().faces().read().data().as_ref().map(|faces| (*f)(faces.as_slice()));
    }

    /// Mutably access the object's texture coordinates.
    #[inline(always)]
    pub fn modify_uvs(&mut self, f: &mut |&mut Vec<Vec2<GLfloat>>| -> ()) {
        let _ = self.mesh.borrow_mut().uvs().write().data_mut().as_mut().map(|uvs| (*f)(uvs));
    }

    /// Access the object's texture coordinates.
    #[inline(always)]
    pub fn read_uvs(&self, f: &mut |&[Vec2<GLfloat>]| -> ()) {
        let _ = self.mesh.borrow().uvs().read().data().as_ref().map(|uvs| (*f)(uvs.as_slice()));
    }


    /// Sets the color of the object.
    ///
    /// Colors components must be on the range `[0.0, 1.0]`.
    #[inline]
    pub fn set_color(&mut self, r: f32, g: f32, b: f32) {
        self.data.color.x = r;
        self.data.color.y = g;
        self.data.color.z = b;
    }

    /// Sets the texture of the object.
    ///
    /// The texture is loaded from a file and registered by the global `TextureManager`.
    ///
    /// # Arguments
    ///   * `path` - relative path of the texture on the disk
    #[inline]
    pub fn set_texture_from_file(&mut self, path: &Path, name: &str) {
        let texture = TextureManager::get_global_manager(|tm| tm.add(path, name));

        self.set_texture(texture)
    }

    /// Sets the texture of the object.
    ///
    /// The texture must already have been registered as `name`.
    #[inline]
    pub fn set_texture_with_name(&mut self, name: &str) {
        let texture = TextureManager::get_global_manager(|tm| tm.get(name).unwrap_or_else(
            || fail!("Invalid attempt to use the unregistered texture: " + name)));

        self.set_texture(texture)
    }

    /// Sets the texture of the object.
    #[inline]
    pub fn set_texture(&mut self, texture: Rc<Texture>) {
        self.data.texture = texture
    }
}
