//! Data structure of a scene node geometry.

use extra::arc::Arc;
use std::ptr;
use std::vec;
use std::mem;
use std::cast;
use gl;
use gl::types::*;
use nalgebra::na::{Vec2, Vec3};
use nalgebra::na;

pub type Coord  = Vec3<GLfloat>;
pub type Normal = Vec3<GLfloat>;
pub type UV     = Vec2<GLfloat>;
pub type Vertex = GLuint;
pub type Face   = Vec3<Vertex>;

#[path = "error.rs"]
mod error;

/// Enumeration of different storage type: shared or note.
pub enum StorageLocation<T> {
    /// The stored data is shared on an Arc.
    SharedImmutable(Arc<T>),
    /// The stored data is not shared.
    NotShared(T)
    // FIXME: add a GPU-only storage location
    // FIXME: add SharedMutable
}

impl<T: Send + Freeze + Clone> Clone for StorageLocation<T> {
    fn clone(&self) -> StorageLocation<T> {
        match *self {
            SharedImmutable(ref t) => SharedImmutable(t.clone()),
            NotShared(ref t)       => NotShared(t.clone())
        }
    }
}

impl<T: Send + Freeze> StorageLocation<T> {
    /// Wraps a new data on the relevant storage location.
    pub fn new(t: T, shared: bool) -> StorageLocation<T> {
        if shared {
            SharedImmutable(Arc::new(t))
        }
        else {
            NotShared(t)
        }
    }

    /// Reads the stored data.
    pub fn get<'r>(&'r self) -> &'r T {
        match *self {
            SharedImmutable(ref s) => s.get(),
            NotShared(ref s)     => s
        }
    }

    /// Indicates whether or not the stored data is shared.
    pub fn is_shared(&self) -> bool {
        match *self {
            SharedImmutable(_) => true,
            NotShared(_)       => false
        }
    }
}

impl<T: Send + Freeze + Clone> StorageLocation<T> {
    /// Applies a function to the wrapped data. If it is shared, then the data is copied.
    pub fn write_cow<'r>(&'r mut self, f: |&mut T| -> ()) {
        match *self {
            SharedImmutable(ref mut s) => {
                let mut cpy = s.get().clone();
                f(&mut cpy);

                *s = Arc::new(cpy);
            },
            NotShared(ref mut s) => f(s)
        }
    }
}

/// A Mesh contains all geometric data of a mesh: vertex buffer, index buffer, normals and uvs.
/// It also contains the GPU location of those buffers.
pub struct Mesh {
    priv coords:  StorageLocation<~[Coord]>,
    priv faces:   StorageLocation<~[Face]>,
    priv normals: StorageLocation<~[Normal]>,
    priv uvs:     StorageLocation<~[UV]>,
    priv ebuf:    GLuint,
    priv nbuf:    GLuint,
    priv vbuf:    GLuint,
    priv tbuf:    GLuint
}

impl Mesh {
    /// Creates a new mesh. Arguments set to `None` are automatically computed.
    pub fn new(coords:          StorageLocation<~[Coord]>,
               faces:           StorageLocation<~[Face]>,
               normals:         Option<StorageLocation<~[Normal]>>,
               uvs:             Option<StorageLocation<~[UV]>>,
               fast_modifiable: bool)
               -> Mesh {
        let normals = match normals {
            Some(ns) => ns,
            None     => {
                let normals = compute_normals_array(*coords.get(), *faces.get());
                StorageLocation::new(normals, coords.is_shared())
            }
        };

        let uvs = match uvs {
            Some(us) => us,
            None     => {
                let uvs = vec::from_elem(coords.get().len(), na::zero());
                StorageLocation::new(uvs, coords.is_shared())
            }
        };

        let draw_location = if fast_modifiable { DynamicDraw } else { StaticDraw };
        Mesh {
            ebuf:    load_buffer(*faces.get(), ElementArrayBuffer, draw_location),
            nbuf:    load_buffer(*normals.get(), ArrayBuffer, draw_location),
            vbuf:    load_buffer(*coords.get(), ArrayBuffer, draw_location),
            tbuf:    load_buffer(*uvs.get(), ArrayBuffer, draw_location),
            coords:  coords,
            faces:   faces,
            normals: normals,
            uvs:     uvs
        }
    }

    /// Upload this mesh datas to the GPU.
    pub fn upload(&self) {
        upload_buffer(*self.faces.get(), self.ebuf, ElementArrayBuffer, StaticDraw);
        upload_buffer(*self.normals.get(), self.nbuf, ArrayBuffer, StaticDraw);
        upload_buffer(*self.coords.get(), self.vbuf, ArrayBuffer, StaticDraw);
        upload_buffer(*self.uvs.get(), self.tbuf, ArrayBuffer, StaticDraw);
    }

    /// Binds this mesh buffers to vertex attributes.
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

    /// Unbind this mesh buffers to vertex attributes.
    pub fn unbind(&self) {
        verify!(gl::BindBuffer(gl::ARRAY_BUFFER, 0));
        verify!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0));
    }

    /// Number of points needed to draw this mesh.
    pub fn num_pts(&self) -> uint {
        self.faces.get().len() * 3
    }

    /// Recompute this mesh normals.
    pub fn recompute_normals(&mut self) {
        self.normals.write_cow(
            |normals| compute_normals(*self.coords.get(), *self.faces.get(), normals)
        )
    }

    /// This mesh faces.
    pub fn faces<'r>(&'r self) -> &'r [Face] {
        let res: &'r [Face] = *self.faces.get();

        res
    }

    /// This mesh faces.
    pub fn mut_faces<'r>(&'r mut self) -> &'r mut StorageLocation<~[Face]> {
        &'r mut self.faces
    }

    /// This mesh normals.
    pub fn normals<'r>(&'r self) -> &'r [Normal] {
        let res: &'r [Normal] = *self.normals.get();

        res
    }

    /// This mesh normals.
    pub fn mut_normals<'r>(&'r mut self) -> &'r mut StorageLocation<~[Normal]> {
        &'r mut self.normals
    }

    /// This mesh vertices coordinates.
    pub fn coords<'r>(&'r self) -> &'r [Coord] {
        let res: &'r [Coord] = *self.coords.get();

        res
    }

    /// This mesh vertices coordinates.
    pub fn mut_coords<'r>(&'r mut self) -> &'r mut StorageLocation<~[Coord]> {
        &'r mut self.coords
    }

    /// This mesh texture coordinates.
    pub fn uvs<'r>(&'r self) -> &'r [UV] {
        let res: &'r [UV] = *self.uvs.get();

        res
    }

    /// This mesh texture coordinates.
    pub fn mut_uvs<'r>(&'r mut self) -> &'r mut StorageLocation<~[UV]> {
        &'r mut self.uvs
    }
}

/// Comutes normals from a set of faces.
pub fn compute_normals_array(coordinates: &[Coord],
                             faces:       &[Face])
                             -> ~[Normal] {
    let mut res = ~[];

    compute_normals(coordinates, faces, &mut res);

    res
}

/// Comutes normals from a set of faces.
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
        *n = na::zero()
    }

    // Grow the output buffer if it is too small.
    normals.grow_set(coordinates.len() - 1, &na::zero(), na::zero());

    // Accumulate normals ...
    for f in faces.iter() {
        let edge1  = coordinates[f.y] - coordinates[f.x];
        let edge2  = coordinates[f.z] - coordinates[f.x];
        let normal = na::normalize(&na::cross(&edge1, &edge2));

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

/// Type of gpu buffer.
pub enum BufferType {
    /// An array buffer bindable to a gl::ARRAY_BUFFER.
    ArrayBuffer,
    /// An array buffer bindable to a gl::ELEMENT_ARRAY_BUFFER.
    ElementArrayBuffer
}

impl BufferType {
    fn to_gl(&self) -> GLuint {
        match *self {
            ArrayBuffer        => gl::ARRAY_BUFFER,
            ElementArrayBuffer => gl::ELEMENT_ARRAY_BUFFER
        }
    }
}

/// Allocation type of gpu buffers.
pub enum AllocationType {
    /// STATIC_DRAW allocation type.
    StaticDraw,
    /// DYNAMIC_DRAW allocation type.
    DynamicDraw,
    /// STREAM_DRAW allocation type.
    StreamDraw
}

impl AllocationType {
    fn to_gl(&self) -> GLuint {
        match *self {
            StaticDraw  => gl::STATIC_DRAW,
            DynamicDraw => gl::DYNAMIC_DRAW,
            StreamDraw  => gl::STREAM_DRAW
        }
    }
}

/// Allocates and uploads a buffer to the gpu.
pub fn load_buffer<T>(buf: &[T], buf_type: BufferType, allocation_type: AllocationType) -> GLuint {
    // Upload values of vertices
    let mut buf_id: GLuint = 0;

    unsafe {
        verify!(gl::GenBuffers(1, &mut buf_id));
        upload_buffer(buf, buf_id, buf_type, allocation_type);
    }

    buf_id
}

/// Allocates and uploads a buffer to the gpu.
pub fn upload_buffer<T>(buf: &[T], buf_id: GLuint, buf_type: BufferType, allocation_type: AllocationType) {
    unsafe {
        verify!(gl::BindBuffer(buf_type.to_gl(), buf_id));
        verify!(gl::BufferData(
                buf_type.to_gl(),
                (buf.len() * mem::size_of::<T>()) as GLsizeiptr,
                cast::transmute(&buf[0]),
                allocation_type.to_gl()));
    }
}

impl Drop for Mesh {
    fn drop(&mut self) {
        unsafe {
            verify!(gl::DeleteBuffers(1, &self.ebuf));
            verify!(gl::DeleteBuffers(1, &self.nbuf));
            verify!(gl::DeleteBuffers(1, &self.vbuf));
            verify!(gl::DeleteBuffers(1, &self.tbuf));
        }
    }
}
