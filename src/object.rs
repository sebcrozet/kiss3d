use std::num::One;
use std::ptr;
use std::cast;
use std::borrow;
use extra::rc::{Rc, RcMut};
use gl;
use gl::types::*;
use nalgebra::mat::{Indexable, ToHomogeneous, Transformation, Transform, Rotation, Rotate, Translation};
use nalgebra::mat::{Mat3, Mat4};
use nalgebra::vec::Vec3;
use nalgebra::types::Iso3f64;
use resources::shaders_manager::ObjectShaderContext;
use resources::textures_manager;
use resources::textures_manager::Texture;
use mesh::Mesh;

#[path = "error.rs"]
mod error;

type Transform3d = Iso3f64;
type Scale3d     = Mat3<GLfloat>;

/// Set of datas identifying a scene node.
pub struct ObjectData {
    priv texture:   Rc<Texture>,
    priv scale:     Scale3d,
    priv transform: Transform3d,
    priv color:     Vec3<f32>,
    priv visible:   bool
}

/// Structure of all 3d objects on the scene. This is the only interface to manipulate the object
/// position, color, vertices and texture.
#[deriving(Clone)]
pub struct Object {
    priv data:    RcMut<ObjectData>,
    priv mesh:    RcMut<Mesh>
}

impl Object {
    #[doc(hidden)]
    pub fn new(mesh:     RcMut<Mesh>,
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
            transform: One::one(),
            color:     Vec3::new(r, g, b),
            texture:   texture,
            visible:   true
        };

        Object {
            data:    RcMut::from_freeze(data),
            mesh:    mesh,
        }
    }

    #[doc(hidden)]
    pub fn upload(&self, context: &ObjectShaderContext) {
        do self.data.with_borrow |data| {
            if data.visible {
                let formated_transform:  Mat4<f64> = data.transform.to_homogeneous();
                let formated_ntransform: Mat3<f64> = data.transform.submat().submat();

                // we convert the matrix elements and do the transposition at the same time
                let transform_glf = Mat4::new(
                    formated_transform.at((0, 0)) as GLfloat,
                    formated_transform.at((1, 0)) as GLfloat,
                    formated_transform.at((2, 0)) as GLfloat,
                    formated_transform.at((3, 0)) as GLfloat,

                    formated_transform.at((0, 1)) as GLfloat,
                    formated_transform.at((1, 1)) as GLfloat,
                    formated_transform.at((2, 1)) as GLfloat,
                    formated_transform.at((3, 1)) as GLfloat,

                    formated_transform.at((0, 2)) as GLfloat,
                    formated_transform.at((1, 2)) as GLfloat,
                    formated_transform.at((2, 2)) as GLfloat,
                    formated_transform.at((3, 2)) as GLfloat,

                    formated_transform.at((0, 3)) as GLfloat,
                    formated_transform.at((1, 3)) as GLfloat,
                    formated_transform.at((2, 3)) as GLfloat,
                    formated_transform.at((3, 3)) as GLfloat
                    );

                let ntransform_glf = Mat3::new(
                    formated_ntransform.at((0, 0)) as GLfloat,
                    formated_ntransform.at((1, 0)) as GLfloat,
                    formated_ntransform.at((2, 0)) as GLfloat,
                    formated_ntransform.at((0, 1)) as GLfloat,
                    formated_ntransform.at((1, 1)) as GLfloat,
                    formated_ntransform.at((2, 1)) as GLfloat,
                    formated_ntransform.at((0, 2)) as GLfloat,
                    formated_ntransform.at((1, 2)) as GLfloat,
                    formated_ntransform.at((2, 2)) as GLfloat
                    );

                unsafe {
                    verify!(gl::UniformMatrix4fv(context.transform,
                                                 1,
                                                 gl::FALSE as u8,
                                                 cast::transmute(&transform_glf)));

                    verify!(gl::UniformMatrix3fv(context.ntransform,
                                                 1,
                                                 gl::FALSE as u8,
                                                 cast::transmute(&ntransform_glf)));

                    verify!(gl::UniformMatrix3fv(context.scale, 1, gl::FALSE as u8, cast::transmute(&data.scale)));

                    verify!(gl::Uniform3f(context.color, data.color.x, data.color.y, data.color.z));

                    // FIXME: we should not switch the buffers if the last drawn shape uses the same.
                    self.mesh.with_borrow(|m| m.bind(context.pos, context.normal, context.tex_coord));

                    verify!(gl::ActiveTexture(gl::TEXTURE0));
                    verify!(gl::BindTexture(gl::TEXTURE_2D, self.data.with_borrow(|d| d.texture.borrow().id())));

                    verify!(gl::DrawElements(gl::TRIANGLES,
                                             self.mesh.with_borrow(|m| m.num_pts()) as GLint,
                                             gl::UNSIGNED_INT,
                                             ptr::null()));

                    self.mesh.with_borrow(|m| m.unbind());
                }
            }
        }
    }

    /// Sets the visible state of this object. An invisible object does not draw itself.
    pub fn set_visible(&mut self, visible: bool) {
        self.data.with_mut_borrow(|d| d.visible = visible)
    }

    /// Returns true if this object can be visible.
    pub fn visible(&self) -> bool {
        self.data.with_borrow(|d| d.visible)
    }

    /// Sets the local scaling factor of the object.
    pub fn set_scale(&mut self, sx: f64, sy: f64, sz: f64) {
        do self.data.with_mut_borrow |d| {
            d.scale = Mat3::new(
                sx as GLfloat, 0.0, 0.0,
                0.0, sy as GLfloat, 0.0,
                0.0, 0.0, sz as GLfloat)
        }
    }

    /// Get a write access to the geometry mesh. Return true if the geometry needs to be
    /// re-uploaded to the GPU.
    pub fn modify_mesh(&mut self, f: &fn(&mut Mesh) -> bool) {
        do self.mesh.with_mut_borrow |m| {
            if f(m) {
                // FIXME: find a way to upload only the modified parts.
                m.upload()
            }
        }
    }

    /// Sets the color of the object. Colors components must be on the range `[0.0, 1.0]`.
    pub fn set_color(&mut self, r: f32, g: f32, b: f32) {
        do self.data.with_mut_borrow |d| {
            d.color.x = r;
            d.color.y = g;
            d.color.z = b;
        }
    }

    /// Sets the texture of the object.
    ///
    /// # Arguments
    ///   * `path` - relative path of the texture on the disk
    pub fn set_texture(&mut self, path: &str) {
        self.data.with_mut_borrow(|d| d.texture = textures_manager::singleton().add(path));
    }

    /// Move and orient the object such that it is placed at the point `eye` and have its `x` axis
    /// oriented toward `at`.
    pub fn look_at(&mut self, eye: &Vec3<f64>, at: &Vec3<f64>, up: &Vec3<f64>) {
        self.data.with_mut_borrow(|d| d.transform.look_at(eye, at, up))
    }

    /// Move and orient the object such that it is placed at the point `eye` and have its `z` axis
    /// oriented toward `at`.
    pub fn look_at_z(&mut self, eye: &Vec3<f64>, at: &Vec3<f64>, up: &Vec3<f64>) {
        self.data.with_mut_borrow(|d| d.transform.look_at_z(eye, at, up))
    }
}

impl Transformation<Transform3d> for Object {
    fn transformation(&self) -> Transform3d {
        self.data.with_borrow(|d| d.transform.clone())
    }

    fn inv_transformation(&self) -> Transform3d {
        self.data.with_borrow(|d| d.transform.inv_transformation())
    }

    fn transform_by(&mut self, t: &Transform3d) {
        self.data.with_mut_borrow(|d| d.transform.transform_by(t))
    }

    fn transformed(&self, _: &Transform3d) -> Object {
        fail!("Cannot clone an object.")
    }

    fn set_transformation(&mut self, t: Transform3d) {
        self.data.with_mut_borrow(|d| d.transform.set_transformation(t))
    }
}

impl Transform<Vec3<f64>> for Object {
    fn transform(&self, v: &Vec3<f64>) -> Vec3<f64> {
        self.data.with_borrow(|d| d.transform.transform(v))
    }

    fn inv_transform(&self, v: &Vec3<f64>) -> Vec3<f64> {
        self.data.with_borrow(|d| d.transform.inv_transform(v))
    }
} 

impl Rotation<Vec3<f64>> for Object {
    fn rotation(&self) -> Vec3<f64> {
        self.data.with_borrow(|d| d.transform.rotation())
    }

    fn inv_rotation(&self) -> Vec3<f64> {
        self.data.with_borrow(|d| d.transform.inv_rotation())
    }

    fn rotate_by(&mut self, t: &Vec3<f64>) {
        self.data.with_mut_borrow(|d| d.transform.rotate_by(t))
    }

    fn rotated(&self, _: &Vec3<f64>) -> Object {
        fail!("Cannot clone an object.")
    }

    fn set_rotation(&mut self, r: Vec3<f64>) {
        self.data.with_mut_borrow(|d| d.transform.set_rotation(r))
    }
}

impl Rotate<Vec3<f64>> for Object {
    fn rotate(&self, v: &Vec3<f64>) -> Vec3<f64> {
        self.data.with_borrow(|d| d.transform.rotate(v))
    }

    fn inv_rotate(&self, v: &Vec3<f64>) -> Vec3<f64> {
        self.data.with_borrow(|d| d.transform.inv_rotate(v))
    }
} 

impl Translation<Vec3<f64>> for Object {
    fn translation(&self) -> Vec3<f64> {
        self.data.with_borrow(|d| d.transform.translation())
    }

    fn inv_translation(&self) -> Vec3<f64> {
        self.data.with_borrow(|d| d.transform.inv_translation())
    }

    fn translate_by(&mut self, t: &Vec3<f64>) {
        self.data.with_mut_borrow(|d| d.transform.translate_by(t))
    }

    fn translated(&self, _: &Vec3<f64>) -> Object {
        fail!("Cannot clone an object.")
    }

    fn set_translation(&mut self, t: Vec3<f64>) {
        self.data.with_mut_borrow(|d| d.transform.set_translation(t))
    }
}

impl Eq for Object {
    fn eq(&self, other: &Object) -> bool {
        self.data.with_borrow(|d1| other.data.with_borrow(|d2| borrow::ref_eq(d1, d2)))
    }
}
