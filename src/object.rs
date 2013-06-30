use std::sys;
use std::libc;
use std::num::One;
use std::ptr;
use glcore::types::GL_VERSION_1_0::*;
use glcore::functions::GL_VERSION_1_1::*;
use glcore::functions::GL_VERSION_2_0::*;
use glcore::consts::GL_VERSION_1_1::*;
use nalgebra::traits::homogeneous::ToHomogeneous;
use nalgebra::traits::transpose::Transpose;
use nalgebra::adaptors::transform::Transform;
use nalgebra::adaptors::rotmat::Rotmat;
use nalgebra::mat::{Mat3, Mat4};
use nalgebra::vec::Vec3;

type Transform3d = Transform<Rotmat<Mat3<GLfloat>>, Vec3<GLfloat>>;
type Scale3d     = Mat3<GLfloat>;

pub struct GeometryIndices
{
  priv offset: uint,
  priv size:   i32
}

impl GeometryIndices
{
  pub fn new(offset: uint, size: i32) -> GeometryIndices
  {
    GeometryIndices {
      offset: offset,
      size:   size
    }
  }
}

pub struct Object
{
  priv scale:     Scale3d,
  priv transform: Transform3d,
  priv color:     Vec3<f32>,
  priv geometry:  GeometryIndices
}

impl Object
{
  pub fn new(geometry: GeometryIndices,
             r: f32,
             g: f32,
             b: f32,
             sx: GLfloat,
             sy: GLfloat,
             sz: GLfloat) -> Object
  {
    Object {
      scale:     Mat3::new( [
                              sx, 0.0, 0.0,
                              0.0, sy, 0.0,
                              0.0, 0.0, sz,
                            ] ),
      transform: One::one(),
      geometry:  geometry,
      color:     Vec3::new([r, g, b])
    }
  }

  pub fn upload(&self,
                color_location:            i32,
                transform_location:        i32,
                scale_location:            i32,
                normal_transform_location: i32)
  {
    let mut formated_transform:  Mat4<GLfloat> = self.transform.to_homogeneous();
    let mut formated_ntransform: Mat3<GLfloat> = self.transform.submat().submat();

    formated_transform.transpose();
    formated_ntransform.transpose();

    unsafe {
      glUniformMatrix4fv(transform_location,
                         1,
                         GL_FALSE,
                         ptr::to_unsafe_ptr(&formated_transform.mij[0]));

      glUniformMatrix3fv(normal_transform_location,
                         1,
                         GL_FALSE,
                         ptr::to_unsafe_ptr(&formated_ntransform.mij[0]));

      glUniformMatrix3fv(scale_location,
                         1,
                         GL_FALSE,
                         ptr::to_unsafe_ptr(&self.scale.mij[0]));

      glUniform3f(color_location, self.color.at[0], self.color.at[1], self.color.at[2]);
      glDrawElements(GL_TRIANGLES,
                     self.geometry.size,
                     GL_UNSIGNED_INT,
                     self.geometry.offset * sys::size_of::<GLuint>() as *libc::c_void);
    }
  }

  pub fn transformation<'r>(&'r mut self) -> &'r mut Transform3d
  { &mut self.transform }

  pub fn set_color(@mut self, r: f32, g: f32, b: f32) -> @mut Object
  {
    self.color.at[0] = r;
    self.color.at[1] = g;
    self.color.at[2] = b;

    self
  }
}
