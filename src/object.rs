use std::sys;
use std::libc;
use std::num::{One, Zero};
use std::ptr;
use std::cast;
use std::vec;
use extra::rc::{Rc, RcMut};
use gl;
use gl::types::*;
use nalgebra::mat::{Indexable, ToHomogeneous, Transformation, Transform, Rotation, Rotate, Translation};
use nalgebra::vec::{Cross, AlgebraicVec};
use nalgebra::adaptors::transform;
use nalgebra::adaptors::rotmat::Rotmat;
use nalgebra::mat::{Mat3, Mat4};
use nalgebra::vec::Vec3;
use window::Window;
use resources::shaders_manager::ObjectShaderContext;
use resources::textures_manager::Texture;

#[path = "error.rs"]
mod error;

type Transform3d = transform::Transform<Rotmat<Mat3<f64>>, Vec3<f64>>;
type Scale3d     = Mat3<GLfloat>;

pub enum Geometry {
    VerticesNormalsTriangles(~[Vec3<f32>], ~[Vec3<f32>], ~[(GLuint, GLuint, GLuint)]),
    Deleted
}

#[doc(hidden)]
pub struct GeometryIndices {
    priv offset:         uint,
    priv size:           i32,
    priv element_buffer: GLuint,
    priv normal_buffer:  GLuint,
    priv vertex_buffer:  GLuint,
    priv texture_buffer: GLuint
}

impl GeometryIndices {
    #[doc(hidden)]
    pub fn new(offset:         uint,
               size:           i32,
               element_buffer: GLuint,
               normal_buffer:  GLuint,
               vertex_buffer:  GLuint,
               texture_buffer: GLuint) -> GeometryIndices {
        GeometryIndices {
            offset:         offset,
            size:           size,
            element_buffer: element_buffer,
            normal_buffer:  normal_buffer,
            vertex_buffer:  vertex_buffer,
            texture_buffer: texture_buffer
        }
    }
}

/// Set of datas identifying a scene node.
pub struct ObjectData {
    priv texture:     Rc<Texture>,
    priv scale:       Scale3d,
    priv transform:   Transform3d,
    priv color:       Vec3<f32>,
    priv igeometry:   GeometryIndices,
    priv geometry:    Geometry
}

/// Structure of all 3d objects on the scene. This is the only interface to manipulate the object
/// position, color, vertices and texture.
#[deriving(Clone)]
pub struct Object {
    priv data: RcMut<ObjectData>
}

impl Object {
    #[doc(hidden)]
    pub fn new(igeometry: GeometryIndices,
               r:         f32,
               g:         f32,
               b:         f32,
               texture:   Rc<Texture>,
               sx:        GLfloat,
               sy:        GLfloat,
               sz:        GLfloat,
               geometry:  Geometry) -> Object {
        let data = ObjectData {
            scale:     Mat3::new(sx, 0.0, 0.0,
                                 0.0, sy, 0.0,
                                 0.0, 0.0, sz),
            transform:   One::one(),
            igeometry:   igeometry,
            geometry:    geometry,
            color:       Vec3::new(r, g, b),
            texture:     texture
        };

        Object {
            data: RcMut::from_freeze(data)
        }
    }

    #[doc(hidden)]
    pub fn upload_geometry(&mut self) {
        do self.data.with_mut_borrow |data| {
            match data.geometry {
                VerticesNormalsTriangles(ref v, ref n, _) =>
                    unsafe {
                        gl::BindBuffer(gl::ARRAY_BUFFER, data.igeometry.vertex_buffer);
                        gl::BufferSubData(
                            gl::ARRAY_BUFFER,
                            0,
                            (v.len() * 3 * sys::size_of::<GLfloat>()) as GLsizeiptr,
                            cast::transmute(&v[0])
                            );

                        gl::BindBuffer(gl::ARRAY_BUFFER, data.igeometry.normal_buffer);
                        gl::BufferSubData(
                            gl::ARRAY_BUFFER,
                            0,
                            (n.len() * 3 * sys::size_of::<GLfloat>()) as GLsizeiptr,
                            cast::transmute(&n[0])
                            );
                    },
                    Deleted => { }
            }
        }
    }

    #[doc(hidden)]
    pub fn upload(&self, context: &ObjectShaderContext) {
        do self.data.with_borrow |data| {
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
                verify!(gl::BindBuffer(gl::ARRAY_BUFFER, data.igeometry.vertex_buffer));
                verify!(gl::VertexAttribPointer(context.pos, 3, gl::FLOAT, gl::FALSE as u8, 0, ptr::null()));
                verify!(gl::BindBuffer(gl::ARRAY_BUFFER, data.igeometry.normal_buffer));
                verify!(gl::VertexAttribPointer(context.normal, 3, gl::FLOAT, gl::FALSE as u8, 0, ptr::null()));
                verify!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, data.igeometry.element_buffer));
                verify!(gl::BindTexture(gl::TEXTURE_2D, data.texture.borrow().id()));
                verify!(gl::BindBuffer(gl::ARRAY_BUFFER, data.igeometry.texture_buffer));
                verify!(gl::VertexAttribPointer(context.tex_coord, 2, gl::FLOAT, gl::FALSE as u8, 0, ptr::null()));

                verify!(gl::DrawElements(gl::TRIANGLES,
                data.igeometry.size,
                gl::UNSIGNED_INT,
                (data.igeometry.offset * sys::size_of::<GLuint>()) as *libc::c_void));
            }
        }
    }

    // /// The object geometry. Some geometries might not be
    // /// available (because they are only loaded on graphics memory); in this case this is a no-op.
    // pub fn geometry<'r>(&'r self) -> &'r Geometry {
    //     do self.data.with_borrow |data| {
    //         &'r data.geometry
    //     }
    // }

    /// Applies a user-defined callback on the object geometry. Some geometries might not be
    /// available (because they are only loaded on graphics memory); in this case this is a no-op.
    ///
    /// # Arguments
    ///   * `f` - A user-defined callback called on the object geometry. If it returns `true`, the
    ///   geometry will be updated on graphics memory too. Otherwise, the modification will not have
    ///   any effect on the 3d display.
    pub fn modify_geometry(&mut self,
                           f: &fn(vertices:  &mut ~[Vec3<f32>],
                           normals:   &mut ~[Vec3<f32>],
                           triangles: &mut ~[(GLuint, GLuint, GLuint)]) -> bool) {
        let update = do self.data.with_mut_borrow |d| {
            match d.geometry {
                VerticesNormalsTriangles(ref mut v, ref mut n, ref mut t) => f(v, n, t),
                Deleted => false
            }
        };

        if update {
            self.upload_geometry()
        }
    }

    /// Applies a user-defined callback on the object vertices. Some geometries might not be
    /// available (because they are only loaded on graphics memory); in this case this is a no-op.
    ///
    /// # Arguments
    ///   * `f` - A user-defined callback called on the object vertice. The normals are automatically
    ///   recomputed. If it returns `true`, the the geometry will be updated on graphics memory too.
    ///   Otherwise, the modifications will not have any effect on the 3d display.
    pub fn modify_vertices(&mut self, f: &fn(&mut ~[Vec3<f32>]) -> bool) {
        let (update, normals) =
            do self.data.with_mut_borrow |d| {
                match d.geometry {
                    VerticesNormalsTriangles(ref mut v, _, _) => (f(v), true),
                    Deleted => (false, false)
                }
            };

        if normals {
            self.recompute_normals()
        }

        if update {
            self.upload_geometry()
        }
    }

    fn recompute_normals(&mut self) {
        do self.data.with_mut_borrow |d| {
            match d.geometry {
                VerticesNormalsTriangles(ref vs, ref mut ns, ref ts) => {
                    let mut divisor = vec::from_elem(vs.len(), 0f32);

                    // ... and compute the mean
                    for n in ns.mut_iter() {
                        *n = Zero::zero()
                    }

                    // accumulate normals...
                    for &(v1, v2, v3) in ts.iter() {
                        let edge1 = vs[v2] - vs[v1];
                        let edge2 = vs[v3] - vs[v1];
                        let normal = edge1.cross(&edge2).normalized();

                        ns[v1] = ns[v1] + normal;
                        ns[v2] = ns[v2] + normal;
                        ns[v3] = ns[v3] + normal;

                        divisor[v1] = divisor[v1] + 1.0;
                        divisor[v2] = divisor[v2] + 1.0;
                        divisor[v3] = divisor[v3] + 1.0;
                    }

                    // ... and compute the mean
                    for (n, divisor) in ns.mut_iter().zip(divisor.iter()) {
                        *n = *n / *divisor
                    }
                },
                Deleted => { }
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
    // FIXME: find a way to avoid the `parent` argument.
    // Maybe using singletons for resourses managers?
    pub fn set_texture(&mut self, parent: &mut Window, path: &str) {
        do self.data.with_mut_borrow |d| {
            d.texture = parent.add_texture(path);
        }
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

// FIXME: there is something weird in nalgebra which prevent his from compiling
// impl Translate<Vec3<f64>> for Object {
//     fn translate(&self, v: &Vec3<f64>) -> Vec3<f64> {
//         self.data.with_borrow(|d| d.transform.translate(v))
//     }
// 
//     fn inv_translate(&self, v: &Vec3<f64>) -> Vec3<f64> {
//         self.data.with_borrow(|d| d.transform.inv_translate(v))
//     }
// } 
