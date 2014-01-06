//! Data structure of a scene node.

use std::ptr;
use std::cast;
use std::borrow;
use std::cell::RefCell;
use std::rc::Rc;
use gl;
use gl::types::*;
use nalgebra::na::{Mat3, Mat4, Vec3, Iso3, Rotation, Rotate, Translation, Transformation};
use nalgebra::na;
use resources::shaders_manager::ObjectShaderContext;
use resources::textures_manager;
use resources::textures_manager::Texture;
use mesh::Mesh;

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
    priv visible:   bool
}

/// Structure of all 3d objects on the scene. This is the only interface to manipulate the object
/// position, color, vertices and texture.
#[deriving(Clone)]
pub struct Object {
    priv data:    Rc<RefCell<ObjectData>>,
    priv mesh:    Rc<RefCell<Mesh>>
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
               sz:       GLfloat) -> Object {
        let data = ObjectData {
            scale:     Mat3::new(sx, 0.0, 0.0,
                                 0.0, sy, 0.0,
                                 0.0, 0.0, sz),
            transform: na::one(),
            color:     Vec3::new(r, g, b),
            texture:   texture,
            visible:   true
        };

        Object {
            data:    Rc::from_mut(RefCell::new(data)),
            mesh:    mesh,
        }
    }

    #[doc(hidden)]
    pub fn upload(&self, context: &ObjectShaderContext) {
        let mut data = self.data.borrow().borrow_mut();
        let data = data.get();
        if data.visible {
            let formated_transform:  Mat4<f32> = na::to_homogeneous(&data.transform);
            let formated_ntransform: Mat3<f32> = *data.transform.rotation.submat();

            // we convert the matrix elements
            unsafe {
                verify!(gl::UniformMatrix4fv(context.transform,
                                             1,
                                             gl::FALSE as u8,
                                             cast::transmute(&formated_transform)));

                verify!(gl::UniformMatrix3fv(context.ntransform,
                                             1,
                                             gl::FALSE as u8,
                                             cast::transmute(&formated_ntransform)));

                verify!(gl::UniformMatrix3fv(context.scale, 1, gl::FALSE as u8, cast::transmute(&data.scale)));

                verify!(gl::Uniform3f(context.color, data.color.x, data.color.y, data.color.z));

                // FIXME: we should not switch the buffers if the last drawn shape uses the same.
                let mesh = self.mesh.borrow().borrow();
                let mesh = mesh.get();
                mesh.bind(context.pos, context.normal, context.tex_coord);

                verify!(gl::ActiveTexture(gl::TEXTURE0));
                verify!(gl::BindTexture(gl::TEXTURE_2D, data.texture.borrow().id()));

                verify!(gl::DrawElements(gl::TRIANGLES,
                                         mesh.num_pts() as GLint,
                                         gl::UNSIGNED_INT,
                                         ptr::null()));

                mesh.unbind();
            }
        }
    }

    /// Sets the visible state of this object. An invisible object does not draw itself.
    pub fn set_visible(&mut self, visible: bool) {
        self.data.borrow().borrow_mut().get().visible = visible
    }

    /// Returns true if this object can be visible.
    pub fn visible(&self) -> bool {
        self.data.borrow().borrow().get().visible
    }

    /// Sets the local scaling factor of the object.
    pub fn set_scale(&mut self, sx: f32, sy: f32, sz: f32) {
        self.data.borrow().borrow_mut().get().scale = Mat3::new(sx, 0.0, 0.0,
                                                                0.0, sy, 0.0,
                                                                0.0, 0.0, sz)
    }

    /// Get a write access to the geometry mesh. Return true if the geometry needs to be
    /// re-uploaded to the GPU.
    pub fn modify_mesh(&mut self, f: |&mut Mesh| -> bool) {
        let mut m = self.mesh.borrow().borrow_mut();
        let m = m.get();
        
        if f(m) {
            // FIXME: find a way to upload only the modified parts.
            m.upload()
        }
    }

    /// Sets the color of the object. Colors components must be on the range `[0.0, 1.0]`.
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
    pub fn set_texture(&mut self, path: &Path, name: &str) {
        let texture = textures_manager::get(|tm| tm.add(path, name));

        self.data.borrow().borrow_mut().get().texture = texture;
    }

    /// Sets the texture of the object.
    ///
    /// The texture must already have been registered as `name`.
    pub fn set_texture_with_name(&mut self, name: &str) {
        let texture = textures_manager::get(|tm| tm.get(name).unwrap_or_else(
            || fail!("Invalid attempt to use the unregistered texture: " + name)));

        self.data.borrow().borrow_mut().get().texture = texture;
    }

    /// Move and orient the object such that it is placed at the point `eye` and have its `x` axis
    /// oriented toward `at`.
    pub fn look_at(&mut self, eye: &Vec3<f32>, at: &Vec3<f32>, up: &Vec3<f32>) {
        let mut data = self.data.borrow().borrow_mut();
        data.get().transform.look_at(eye, at, up)
    }

    /// Move and orient the object such that it is placed at the point `eye` and have its `z` axis
    /// oriented toward `at`.
    pub fn look_at_z(&mut self, eye: &Vec3<f32>, at: &Vec3<f32>, up: &Vec3<f32>) {
        let mut data = self.data.borrow().borrow_mut();
        data.get().transform.look_at_z(eye, at, up)
    }
}

impl Transformation<Transform3d> for Object {
    fn transformation(&self) -> Transform3d {
        let data = self.data.borrow().borrow();
        data.get().transform.clone()
    }

    fn inv_transformation(&self) -> Transform3d {
        let data = self.data.borrow().borrow();
        data.get().transform.inv_transformation()
    }

    fn append_transformation(&mut self, t: &Transform3d) {
        let mut data = self.data.borrow().borrow_mut();
        data.get().transform.append_transformation(t)
    }

    fn append_transformation_cpy(_: &Object, _: &Transform3d) -> Object {
        fail!("Cannot clone an object.")
    }

    fn prepend_transformation(&mut self, t: &Transform3d) {
        let mut data = self.data.borrow().borrow_mut();
        data.get().transform.prepend_transformation(t)
    }

    fn prepend_transformation_cpy(_: &Object, _: &Transform3d) -> Object {
        fail!("Cannot clone an object.")
    }

    fn set_transformation(&mut self, t: Transform3d) {
        let mut data = self.data.borrow().borrow_mut();
        data.get().transform.set_transformation(t)
    }
}

impl na::Transform<Vec3<f32>> for Object {
    fn transform(&self, v: &Vec3<f32>) -> Vec3<f32> {
        let data = self.data.borrow().borrow();;
        data.get().transform.transform(v)
    }

    fn inv_transform(&self, v: &Vec3<f32>) -> Vec3<f32> {
        let data = self.data.borrow().borrow();
        data.get().transform.inv_transform(v)
    }
} 

impl Rotation<Vec3<f32>> for Object {
    fn rotation(&self) -> Vec3<f32> {
        let data = self.data.borrow().borrow();
        data.get().transform.rotation()
    }

    fn inv_rotation(&self) -> Vec3<f32> {
        let data = self.data.borrow().borrow();
        data.get().transform.inv_rotation()
    }

    fn append_rotation(&mut self, t: &Vec3<f32>) {
        let mut data = self.data.borrow().borrow_mut();
        data.get().transform.append_rotation(t)
    }

    fn append_rotation_cpy(_: &Object, _: &Vec3<f32>) -> Object {
        fail!("Cannot clone an object.")
    }

    fn prepend_rotation(&mut self, t: &Vec3<f32>) {
        let mut data = self.data.borrow().borrow_mut();
        data.get().transform.prepend_rotation(t)
    }

    fn prepend_rotation_cpy(_: &Object, _: &Vec3<f32>) -> Object {
        fail!("Cannot clone an object.")
    }

    fn set_rotation(&mut self, r: Vec3<f32>) {
        let mut data = self.data.borrow().borrow_mut();
        data.get().transform.set_rotation(r)
    }
}

impl Rotate<Vec3<f32>> for Object {
    fn rotate(&self, v: &Vec3<f32>) -> Vec3<f32> {
        let data = self.data.borrow().borrow();
        data.get().transform.rotate(v)
    }

    fn inv_rotate(&self, v: &Vec3<f32>) -> Vec3<f32> {
        let data = self.data.borrow().borrow();
        data.get().transform.inv_rotate(v)
    }
} 

impl Translation<Vec3<f32>> for Object {
    fn translation(&self) -> Vec3<f32> {
        let data = self.data.borrow().borrow();
        data.get().transform.translation()
    }

    fn inv_translation(&self) -> Vec3<f32> {
        let data = self.data.borrow().borrow();
        data.get().transform.inv_translation()
    }

    fn append_translation(&mut self, t: &Vec3<f32>) {
        let mut data = self.data.borrow().borrow_mut();
        data.get().transform.append_translation(t)
    }

    fn append_translation_cpy(_: &Object, _: &Vec3<f32>) -> Object {
        fail!("Cannot clone an object.")
    }

    fn prepend_translation(&mut self, t: &Vec3<f32>) {
        let mut data = self.data.borrow().borrow_mut();
        data.get().transform.prepend_translation(t)
    }

    fn prepend_translation_cpy(_: &Object, _: &Vec3<f32>) -> Object {
        fail!("Cannot clone an object.")
    }

    fn set_translation(&mut self, t: Vec3<f32>) {
        let mut data = self.data.borrow().borrow_mut();
        data.get().transform.set_translation(t)
    }
}

impl Eq for Object {
    fn eq(&self, other: &Object) -> bool {
        let d1 = self.data.borrow().borrow();
        let d2 = other.data.borrow().borrow();

        borrow::ref_eq(d1.get(), d2.get())
    }
}
