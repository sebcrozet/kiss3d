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

pub struct Object
{
  priv transform: Transform3d,
  priv vertices: [i32, ..2],
  priv color:    [f32, ..3],

}

impl Object
{
  pub fn new(v1: i32, v2: i32, r: f32, g: f32, b: f32) -> Object
  {
    Object {
      transform: One::one(),
      vertices: [v1, v2],
      color:    [r, g, b]
    }
  }

  pub fn upload(&self,
                color_location:            i32,
                transform_location:        i32,
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

      glUniform3f(color_location, self.color[0], self.color[1], self.color[2]);
      glDrawArrays(GL_TRIANGLES, self.vertices[0], self.vertices[1]);
    }
  }

  pub fn transformation<'r>(&'r mut self) -> &'r mut Transform3d
  { &mut self.transform }

  pub fn set_color(@mut self, r: f32, g: f32, b: f32) -> @mut Object
  {
    self.color[0] = r;
    self.color[1] = g;
    self.color[2] = b;

    self
  }
}
