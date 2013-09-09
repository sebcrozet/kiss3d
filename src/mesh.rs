use std::ptr;
use std::num::Zero;
use std::vec;
use std::sys;
use std::cast;
use gl;
use gl::types::*;
use nalgebra::vec::{Vec2, Vec3, Cross, Norm, Dim};

pub type Coord  = Vec3<GLfloat>;
pub type Normal = Vec3<GLfloat>;
pub type UV     = Vec2<GLfloat>;
pub type Vertex = GLuint;
pub type Face   = Vec3<Vertex>;

#[path = "error.rs"]
mod error;

pub struct Mesh {
    priv coords:  ~[Coord],
    priv faces:   ~[Face],
    priv normals: ~[Normal],
    priv uvs:     ~[UV],
    priv ebuf:    GLuint,
    priv nbuf:    GLuint,
    priv vbuf:    GLuint,
    priv tbuf:    GLuint
}

impl Mesh {
    pub fn new(coords:   ~[Coord],
               faces:    ~[Face],
               normals:  Option<~[Normal]>,
               uvs:      Option<~[UV]>)
               -> Mesh {
        let normals = match normals {
            Some(ns) => ns,
            None     => compute_normals_array(coords, faces)
        };

        let uvs = match uvs {
            Some(us) => us,
            None     => vec::from_elem(coords.len(), Zero::zero()) // dummy uvs
        };

        Mesh {
            ebuf:    load_buffer(faces, ElementArrayBuffer, StaticDraw),
            nbuf:    load_buffer(normals, ArrayBuffer, StaticDraw),
            vbuf:    load_buffer(coords, ArrayBuffer, StaticDraw),
            tbuf:    load_buffer(uvs, ArrayBuffer, StaticDraw),
            coords:  coords,
            faces:   faces,
            normals: normals,
            uvs:     uvs
        }
    }

    pub fn bind(&self, coords: GLuint, normals: GLuint, uvs: GLuint) {
        unsafe {
            verify!(gl::BindBuffer(gl::ARRAY_BUFFER, self.vbuf));
            verify!(gl::VertexAttribPointer(coords, 3, gl::FLOAT, gl::FALSE as u8, 0, ptr::null()));

            verify!(gl::BindBuffer(gl::ARRAY_BUFFER, self.nbuf));
            verify!(gl::VertexAttribPointer(normals, 3, gl::FLOAT, gl::FALSE as u8, 0, ptr::null()));

            verify!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebuf));

            verify!(gl::BindBuffer(gl::ARRAY_BUFFER, self.tbuf));
            verify!(gl::VertexAttribPointer(uvs, 2, gl::FLOAT, gl::FALSE as u8, 0, ptr::null()));
        }
    }

    pub fn num_pts(&self) -> uint {
        self.faces.len()
    }

    pub fn recompute_normals(&mut self) {
        compute_normals(self.coords, self.faces, &mut self.normals);
    }

    pub fn faces<'r>(&'r self) -> &'r [Face] {
        let res: &'r [Face] = self.faces;

        res
    }

    pub fn faces_mut<'r>(&'r mut self) -> &'r mut [Face] {
        let res: &'r mut [Face] = self.faces;

        res
    }

    pub fn normals<'r>(&'r self) -> &'r [Normal] {
        let res: &'r [Normal] = self.normals;

        res
    }

    pub fn normals_mut<'r>(&'r mut self) -> &'r mut [Normal] {
        let res: &'r mut [Normal] = self.normals;

        res
    }

    pub fn coordinates<'r>(&'r self) -> &'r [Coord] {
        let res: &'r [Coord] = self.coords;

        res
    }

    pub fn coordinates_mut<'r>(&'r mut self) -> &'r mut [Coord] {
        let res: &'r mut [Coord] = self.coords;

        res
    }

    pub fn uvs<'r>(&'r self) -> &'r [UV] {
        let res: &'r [UV] = self.uvs;

        res
    }

    pub fn uvs_mut<'r>(&'r mut self) -> &'r mut [UV] {
        let res: &'r mut [UV] = self.uvs;

        res
    }
}

pub fn compute_normals_array(coordinates: &[Coord],
                             faces:       &[Face])
                             -> ~[Normal] {
    let mut res = ~[];

    compute_normals(coordinates, faces, &mut res);

    res
}

pub fn compute_normals(coordinates: &[Coord],
                       faces:       &[Face],
                       normals:     &mut ~[Normal]) {
    let mut divisor = vec::from_elem(coordinates.len(), 0f32);

    // Shrink the output buffer if it is too big.
    if normals.len() > coordinates.len() {
        normals.truncate(coordinates.len())
    }

    // Reinit all normals to zero.
    for n in normals.mut_iter() {
        *n = Zero::zero()
    }

    // Grow the output buffer if it is too small.
    normals.grow_set(coordinates.len() - 1, &Zero::zero(), Zero::zero());

    // Accumulate normals ...
    for f in faces.iter() {
        let edge1  = coordinates[f.y] - coordinates[f.x];
        let edge2  = coordinates[f.z] - coordinates[f.x];
        let normal = edge1.cross(&edge2).normalized();

        normals[f.x] = normals[f.x] + normal;
        normals[f.y] = normals[f.y] + normal;
        normals[f.z] = normals[f.z] + normal;

        divisor[f.x] = divisor[f.x] + 1.0;
        divisor[f.y] = divisor[f.y] + 1.0;
        divisor[f.z] = divisor[f.z] + 1.0;
    }

    // ... and compute the mean
    for (n, divisor) in normals.mut_iter().zip(divisor.iter()) {
        *n = *n / *divisor
    }
}

pub enum BufferType {
    ArrayBuffer,
    ElementArrayBuffer
}

impl BufferType {
    pub fn to_gl(&self) -> GLuint {
        match *self {
            ArrayBuffer        => gl::ARRAY_BUFFER,
            ElementArrayBuffer => gl::ELEMENT_ARRAY_BUFFER
        }
    }
}

pub enum AllocationType {
    StaticDraw,
    DynamicDraw
}

impl AllocationType {
    pub fn to_gl(&self) -> GLuint {
        match *self {
            StaticDraw  => gl::STATIC_DRAW,
            DynamicDraw => gl::DYNAMIC_DRAW
        }
    }
}

pub fn load_buffer<T>(buf: &[T], buf_type: BufferType, allocation_type: AllocationType) -> GLuint {
    // Upload values of vertices
    let buf_id: GLuint = 0;

    unsafe {
        verify!(gl::GenBuffers(1, &buf_id));
        verify!(gl::BindBuffer(buf_type.to_gl(), buf_id));
        verify!(gl::BufferData(
                buf_type.to_gl(),
                (buf.len() * sys::size_of::<T>()) as GLsizeiptr,
                cast::transmute(&buf[0]),
                allocation_type.to_gl()));
    }

    buf_id
}
