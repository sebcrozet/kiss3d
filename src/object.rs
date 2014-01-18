//! Data structure of a scene node.

use std::borrow;
use std::cell::RefCell;
use std::rc::Rc;
use gl::types::*;
use nalgebra::na::{Mat3, Vec2, Vec3, Iso3, Rotation, Rotate, Translation, Transformation};
use nalgebra::na;
use resource::{Texture, TextureManager, Material, Mesh};
use camera::Camera;
use light::Light;

#[path = "error.rs"]
mod error;

type Transform3d = Iso3<f32>;
type Scale3d     = Mat3<GLfloat>;

/// Set of datas identifying a scene node.
pub struct ObjectData {
    priv scale:     Scale3d,
    priv transform: Transform3d,
    priv texture:   Rc<Texture>,
    priv color:     Vec3<f32>,
    priv visible:   bool,
    priv user_data: ~Any
}

impl ObjectData {
    /// The scale matrix of this object.
    pub fn scale<'a>(&'a self) -> &'a Scale3d {
        &'a self.scale
    }

    /// The transformation matrix (scaling excluded) of this object.
    pub fn transform<'a>(&'a self) -> &'a Transform3d {
        &'a self.transform
    }

    /// The texture of this object.
    pub fn texture<'a>(&'a self) -> &'a Rc<Texture> {
        &'a self.texture
    }

    /// The color of this object.
    pub fn color<'a>(&'a self) -> &'a Vec3<f32> {
        &'a self.color
    }

    /// Checks whether this objet is visible or not.
    pub fn visible(&self) -> bool {
        self.visible
    }

    /// An user-defined data.
    ///
    /// Use dynamic typing capabilities of the `Any` type to recover the actual datas.
    pub fn user_data<'a>(&'a self) -> &'a Any {
        let res: &'a Any = self.user_data;

        res
    }
}

/// A 3d objects on the scene.
///
/// This is the only interface to manipulate the object position, color, vertices and texture.
#[deriving(Clone)]
pub struct Object {
    priv material: Rc<RefCell<Rc<RefCell<~Material>>>>,
    priv data:     Rc<RefCell<ObjectData>>,
    priv mesh:     Rc<RefCell<Mesh>>
}

impl Object {
    #[doc(hidden)]
    pub fn new(mesh:     Rc<RefCell<Mesh>>,
               r:        f32,
               g:        f32,
               b:        f32,
               texture:  Rc<Texture>,
               sx:       GLfloat,
               sy:       GLfloat,
               sz:       GLfloat,
               material: Rc<RefCell<~Material>>) -> Object {
        let data = ObjectData {
            scale:     Mat3::new(sx, 0.0, 0.0,
                                 0.0, sy, 0.0,
                                 0.0, 0.0, sz),
            transform: na::one(),
            color:     Vec3::new(r, g, b),
            texture:   texture,
            visible:   true,
            user_data: ~() as ~Any
        };

        Object {
            data:     Rc::new(RefCell::new(data)),
            mesh:     mesh,
            material: Rc::new(RefCell::new(material))
        }
    }

    #[doc(hidden)]
    pub fn render(&self, pass: uint, camera: &mut Camera, light: &Light) {
        let visible = self.data.borrow().borrow().get().visible;

        if visible {
            self.material.borrow().borrow().get().borrow().borrow_mut().get().render(
                pass,
                camera,
                light,
                self.data.borrow().borrow().get(),
                self.mesh.borrow().borrow_mut().get());
        }
    }

    /// Gets the data of this object.
    #[inline]
    pub fn data<'a>(&'a self) -> &'a Rc<RefCell<ObjectData>> {
        &'a self.data
    }

    /// Attaches user-defined datas to this object.
    #[inline]
    pub fn set_user_data(&mut self, user_data: ~Any) {
        self.data.borrow().borrow_mut().get().user_data = user_data;
    }

    /// Gets the material of this object.
    #[inline]
    pub fn material(&self) -> Rc<RefCell<~Material>> {
        self.material.borrow().borrow().get().clone()
    }

    /// Sets the material of this object.
    #[inline]
    pub fn set_material(&mut self, material: Rc<RefCell<~Material>>) {
        *self.material.borrow().borrow_mut().get() = material;
    }

    /// Sets the visible state of this object. An invisible object does not draw itself.
    #[inline]
    pub fn set_visible(&mut self, visible: bool) {
        self.data.borrow().borrow_mut().get().visible = visible
    }

    /// Returns true if this object can be visible.
    #[inline]
    pub fn visible(&self) -> bool {
        self.data.borrow().borrow().get().visible
    }

    /// Sets the local scaling factor of the object.
    #[inline]
    pub fn set_scale(&mut self, sx: f32, sy: f32, sz: f32) {
        self.data.borrow().borrow_mut().get().scale = Mat3::new(sx, 0.0, 0.0,
                                                                0.0, sy, 0.0,
                                                                0.0, 0.0, sz)
    }

    /// This object's mesh.
    #[inline]
    pub fn mesh<'a>(&'a self) -> &'a Rc<RefCell<Mesh>> {
        &'a self.mesh
    }

    /// Mutably access the object's vertices.
    #[inline(always)]
    pub fn modify_vertices(&mut self, f: |&mut ~[Vec3<GLfloat>]| -> ()) {
        let     m  = self.mesh();
        let mut bm = m.borrow().borrow_mut();

        let coords = bm.get().coords();

        coords.write(|coords| coords.write(|coords| { f(coords) }));
    }

    /// Access the object's vertices.
    #[inline(always)]
    pub fn read_vertices(&self, f: |&[Vec3<GLfloat>]| -> ()) {
        let m  = self.mesh();
        let bm = m.borrow().borrow();

        let coords = bm.get().coords();

        coords.read(|coords| coords.read(|coords| { f(coords) }));
    }

    /// Recomputes the normals of this object's mesh.
    #[inline]
    pub fn recompute_normals(&mut self) {
        let     m  = self.mesh();
        let mut bm = m.borrow().borrow_mut();

        bm.get().recompute_normals();
    }

    /// Mutably access the object's normals.
    #[inline(always)]
    pub fn modify_normals(&mut self, f: |&mut ~[Vec3<GLfloat>]| -> ()) {
        let     m  = self.mesh();
        let mut bm = m.borrow().borrow_mut();

        let normals = bm.get().normals();

        normals.write(|normals| normals.write(|normals| { f(normals) }));
    }

    /// Access the object's normals.
    #[inline(always)]
    pub fn read_normals(&self, f: |&[Vec3<GLfloat>]| -> ()) {
        let m  = self.mesh();
        let bm = m.borrow().borrow();

        let normals = bm.get().normals();

        normals.read(|normals| normals.read(|normals| { f(normals) }));
    }

    /// Mutably access the object's faces.
    #[inline(always)]
    pub fn modify_faces(&mut self, f: |&mut ~[Vec3<GLuint>]| -> ()) {
        let     m  = self.mesh();
        let mut bm = m.borrow().borrow_mut();

        let faces = bm.get().faces();

        faces.write(|faces| faces.write(|faces| { f(faces) }));
    }

    /// Access the object's faces.
    #[inline(always)]
    pub fn read_faces(&self, f: |&[Vec3<GLuint>]| -> ()) {
        let m  = self.mesh();
        let bm = m.borrow().borrow();

        let faces = bm.get().faces();

        faces.read(|faces| faces.read(|faces| { f(faces) }));
    }

    /// Mutably access the object's uvs.
    #[inline(always)]
    pub fn modify_uvs(&mut self, f: |&mut ~[Vec2<GLfloat>]| -> ()) {
        let     m  = self.mesh();
        let mut bm = m.borrow().borrow_mut();

        let uvs = bm.get().uvs();

        uvs.write(|uvs| uvs.write(|uvs| { f(uvs) }));
    }

    /// Access the object's uvs.
    #[inline(always)]
    pub fn read_uvs(&self, f: |&[Vec2<GLfloat>]| -> ()) {
        let m  = self.mesh();
        let bm = m.borrow().borrow();

        let uvs = bm.get().uvs();

        uvs.read(|uvs| uvs.read(|uvs| { f(uvs) }));
    }


    /// Sets the color of the object. Colors components must be on the range `[0.0, 1.0]`.
    #[inline]
    pub fn set_color(&mut self, r: f32, g: f32, b: f32) {
        let mut d = self.data.borrow().borrow_mut();
        let d = d.get();

        d.color.x = r;
        d.color.y = g;
        d.color.z = b;
    }

    /// Sets the texture of the object.
    ///
    /// # Arguments
    ///   * `path` - relative path of the texture on the disk
    #[inline]
    pub fn set_texture(&mut self, path: &Path, name: &str) {
        let texture = TextureManager::get_global_manager(|tm| tm.add(path, name));

        self.data.borrow().borrow_mut().get().texture = texture;
    }

    /// Sets the texture of the object.
    ///
    /// The texture must already have been registered as `name`.
    #[inline]
    pub fn set_texture_with_name(&mut self, name: &str) {
        let texture = TextureManager::get_global_manager(|tm| tm.get(name).unwrap_or_else(
            || fail!("Invalid attempt to use the unregistered texture: " + name)));

        self.data.borrow().borrow_mut().get().texture = texture;
    }

    /// Move and orient the object such that it is placed at the point `eye` and have its `x` axis
    /// oriented toward `at`.
    #[inline]
    pub fn look_at(&mut self, eye: &Vec3<f32>, at: &Vec3<f32>, up: &Vec3<f32>) {
        self.data.borrow().borrow_mut().get().transform.look_at(eye, at, up)
    }

    /// Move and orient the object such that it is placed at the point `eye` and have its `z` axis
    /// oriented toward `at`.
    #[inline]
    pub fn look_at_z(&mut self, eye: &Vec3<f32>, at: &Vec3<f32>, up: &Vec3<f32>) {
        self.data.borrow().borrow_mut().get().transform.look_at_z(eye, at, up)
    }
}

impl Transformation<Transform3d> for Object {
    #[inline]
    fn transformation(&self) -> Transform3d {
        self.data.borrow().borrow().get().transform.clone()
    }

    #[inline]
    fn inv_transformation(&self) -> Transform3d {
        self.data.borrow().borrow().get().transform.inv_transformation()
    }

    #[inline]
    fn append_transformation(&mut self, t: &Transform3d) {
        self.data.borrow().borrow_mut().get().transform.append_transformation(t)
    }

    #[inline]
    fn append_transformation_cpy(_: &Object, _: &Transform3d) -> Object {
        fail!("Cannot clone an object.")
    }

    #[inline]
    fn prepend_transformation(&mut self, t: &Transform3d) {
        self.data.borrow().borrow_mut().get().transform.prepend_transformation(t)
    }

    #[inline]
    fn prepend_transformation_cpy(_: &Object, _: &Transform3d) -> Object {
        fail!("Cannot clone an object.")
    }

    #[inline]
    fn set_transformation(&mut self, t: Transform3d) {
        self.data.borrow().borrow_mut().get().transform.set_transformation(t)
    }
}

impl na::Transform<Vec3<f32>> for Object {
    #[inline]
    fn transform(&self, v: &Vec3<f32>) -> Vec3<f32> {
        self.data.borrow().borrow().get().transform.transform(v)
    }

    #[inline]
    fn inv_transform(&self, v: &Vec3<f32>) -> Vec3<f32> {
        self.data.borrow().borrow().get().transform.inv_transform(v)
    }
} 

impl Rotation<Vec3<f32>> for Object {
    #[inline]
    fn rotation(&self) -> Vec3<f32> {
        self.data.borrow().borrow().get().transform.rotation()
    }

    #[inline]
    fn inv_rotation(&self) -> Vec3<f32> {
        self.data.borrow().borrow().get().transform.inv_rotation()
    }

    #[inline]
    fn append_rotation(&mut self, t: &Vec3<f32>) {
        self.data.borrow().borrow_mut().get().transform.append_rotation(t)
    }

    #[inline]
    fn append_rotation_cpy(_: &Object, _: &Vec3<f32>) -> Object {
        fail!("Cannot clone an object.")
    }

    #[inline]
    fn prepend_rotation(&mut self, t: &Vec3<f32>) {
        self.data.borrow().borrow_mut().get().transform.prepend_rotation(t)
    }

    #[inline]
    fn prepend_rotation_cpy(_: &Object, _: &Vec3<f32>) -> Object {
        fail!("Cannot clone an object.")
    }

    #[inline]
    fn set_rotation(&mut self, r: Vec3<f32>) {
        self.data.borrow().borrow_mut().get().transform.set_rotation(r)
    }
}

impl Rotate<Vec3<f32>> for Object {
    #[inline]
    fn rotate(&self, v: &Vec3<f32>) -> Vec3<f32> {
        self.data.borrow().borrow().get().transform.rotate(v)
    }

    #[inline]
    fn inv_rotate(&self, v: &Vec3<f32>) -> Vec3<f32> {
        self.data.borrow().borrow().get().transform.inv_rotate(v)
    }
} 

impl Translation<Vec3<f32>> for Object {
    #[inline]
    fn translation(&self) -> Vec3<f32> {
        self.data.borrow().borrow().get().transform.translation()
    }

    #[inline]
    fn inv_translation(&self) -> Vec3<f32> {
        self.data.borrow().borrow().get().transform.inv_translation()
    }

    #[inline]
    fn append_translation(&mut self, t: &Vec3<f32>) {
        self.data.borrow().borrow_mut().get().transform.append_translation(t)
    }

    #[inline]
    fn append_translation_cpy(_: &Object, _: &Vec3<f32>) -> Object {
        fail!("Cannot clone an object.")
    }

    #[inline]
    fn prepend_translation(&mut self, t: &Vec3<f32>) {
        self.data.borrow().borrow_mut().get().transform.prepend_translation(t)
    }

    #[inline]
    fn prepend_translation_cpy(_: &Object, _: &Vec3<f32>) -> Object {
        fail!("Cannot clone an object.")
    }

    #[inline]
    fn set_translation(&mut self, t: Vec3<f32>) {
        self.data.borrow().borrow_mut().get().transform.set_translation(t)
    }
}

impl Eq for Object {
    #[inline]
    fn eq(&self, other: &Object) -> bool {
        let d1 = self.data.borrow().borrow();
        let d2 = other.data.borrow().borrow();

        borrow::ref_eq(d1.get(), d2.get())
    }
}
