//! Wrapper for an OpenGL buffer object.

use std::cast;
use std::mem;
use std::ptr;
use std::vec;
use gl;
use gl::types::*;
use std::util::NonCopyable;
use gl_primitive::GLPrimitive;

#[path = "error.rs"]
mod error;

struct GLHandle {
    priv handle: GLuint,
    priv nocpy:  NonCopyable
}

impl GLHandle {
    pub fn new(handle: GLuint) -> GLHandle {
        GLHandle {
            handle: handle,
            nocpy:  NonCopyable
        }
    }

    pub fn handle(&self) -> GLuint {
        self.handle
    }
}

impl Drop for GLHandle {
    fn drop(&mut self) {
        unsafe {
            verify!(gl::DeleteBuffers(1, &self.handle))
        }
    }
}

// FIXME: generalize this for any resource: GPUResource
/// A vector of elements that can be loaded to the GPU, on the RAM, or both.
pub struct GPUVector<T> {
    priv len:        uint,
    priv buf_type:   BufferType,
    priv alloc_type: AllocationType,
    priv handle:     Option<GLHandle>,
    priv data:       Option<~[T]>,
}

// FIXME: implement Clone

impl<T: GLPrimitive> GPUVector<T> {
    /// Creates a new `GpuVector` that is not yet uploaded to the GPU.
    pub fn new(data: ~[T], buf_type: BufferType, alloc_type: AllocationType) -> GPUVector<T> {
        GPUVector {
            len:        data.len(),
            buf_type:   buf_type,
            alloc_type: alloc_type,
            handle:     None,
            data:       Some(data)
        }
    }

    /// The length of this vector.
    pub fn len(&self) -> uint {
        self.len
    }

    /// Modifies this vector.
    ///
    /// This will do nothing if the vector os not available on RAM.
    pub fn write(&mut self, f: |&mut ~[T]| -> ()) {
        match self.data {
            None            => { }
            Some(ref mut d) => {
                f(d)
            }
        }

        if self.is_on_gpu() {
            self.reload_to_gpu();
        }
    }

    /// Immutably accesses the vector.
    pub fn read(&self, f: |&[T]| -> ()) {
        match self.data {
            None            => { }
            Some(ref d) => { f(*d) }
        }
    }

    /// Returns `true` if this vector is already uploaded to the GPU.
    pub fn is_on_gpu(&self) -> bool {
        self.handle.is_some()
    }

    /// Retuns `true` if this vector is available on RAM.
    ///
    /// Note that a GPUVector may be both on RAM and on the GPU.
    pub fn is_on_ram(&self) -> bool {
        self.data.is_some()
    }

    /// Loads the vector from the RAM to the GPU.
    ///
    /// If the vector is not available on RAM or already loaded to the GPU, nothing will happen.
    pub fn load_to_gpu(&mut self) {
        if !self.is_on_gpu() {
            self.handle = self.data.as_ref().map(|d| GLHandle::new(
                    upload_buffer(*d, self.buf_type, self.alloc_type)
                )
            );
        }
    }

    fn reload_to_gpu(&mut self) {
        if self.is_on_gpu() {
            let handle = self.handle.as_ref().map(|h| h.handle()).unwrap();

            self.data.as_ref().map(|d| {
                update_buffer(*d, handle, self.buf_type, self.alloc_type)
            });
        }
    }

    /// Binds this vector to gpu buffer and attribute.
    pub fn bind(&mut self, attribute: Option<GLuint>) {
        self.load_to_gpu();

        unsafe {
            let handle = self.handle.as_ref().map(|h| h.handle()).unwrap();
            verify!(gl::BindBuffer(self.buf_type.to_gl(), handle));
            attribute.map(|attr|
                verify!(gl::VertexAttribPointer(
                        attr,
                        GLPrimitive::size(None::<T>) as i32,
                        GLPrimitive::gl_type(None::<T>),
                        gl::FALSE as u8,
                        0,
                        ptr::null())));
        }
    }

    /// Unbind this vector to the corresponding gpu buffer.
    pub fn unbind(&mut self) {
        if self.is_on_gpu() {
            unsafe {
                let handle = self.handle.as_ref().map(|h| h.handle()).unwrap();
                verify!(gl::BindBuffer(self.buf_type.to_gl(), handle));
            }
        }
    }

    /// Loads the vector from the GPU to the RAM.
    ///
    /// If the vector is not available on the GPU or already loaded to the RAM, nothing will
    /// happen.
    pub fn load_to_ram(&mut self) {
        if !self.is_on_ram() && self.is_on_gpu() {
            let     handle = self.handle.as_ref().map(|h| h.handle()).unwrap();
            let mut data   = vec::with_capacity(self.len);

            unsafe { data.set_len(self.len) };
            download_buffer(handle, self.buf_type, data);
            self.data = Some(data);
        }
    }

    /// Unloads this resourse from the GPU.
    pub fn unload_from_gpu(&mut self) {
        self.handle.as_ref().map(|h| unsafe { verify!(gl::DeleteBuffers(1, &h.handle())) });
        self.handle = None;
    }

    /// Removes this resource from the RAM.
    ///
    /// This is useful to save memory for vectors required on the GPU only.
    pub fn unload_from_ram(&mut self) {
        self.data = None;
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
pub fn upload_buffer<T: GLPrimitive>(buf:             &[T],
                                     buf_type:        BufferType,
                                     allocation_type: AllocationType)
                                     -> GLuint {
    // Upload values of vertices
    let mut buf_id: GLuint = 0;

    unsafe {
        verify!(gl::GenBuffers(1, &mut buf_id));
        update_buffer(buf, buf_id, buf_type, allocation_type);
    }

    buf_id
}

/// Downloads a buffer from the gpu.
///
/// 
pub fn download_buffer<T: GLPrimitive>(buf_id: GLuint, buf_type: BufferType, out: &mut [T]) {
    unsafe {
        verify!(gl::BindBuffer(buf_type.to_gl(), buf_id));
        verify!(gl::GetBufferSubData(
                buf_type.to_gl(),
                0,
                (out.len() * mem::size_of::<T>()) as GLsizeiptr,
                cast::transmute(&out[0])));
    }
}

/// Updates a buffer to the gpu.
pub fn update_buffer<T: GLPrimitive>(buf:             &[T],
                                     buf_id:          GLuint,
                                     buf_type:        BufferType,
                                     allocation_type: AllocationType) {
    unsafe {
        verify!(gl::BindBuffer(buf_type.to_gl(), buf_id));
        verify!(gl::BufferData(
                buf_type.to_gl(),
                (buf.len() * mem::size_of::<T>()) as GLsizeiptr,
                cast::transmute(&buf[0]),
                allocation_type.to_gl()));
    }
}
