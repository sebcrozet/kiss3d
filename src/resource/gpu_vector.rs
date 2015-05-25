//! Wrapper for an OpenGL buffer object.

use std::mem;
use gl;
use gl::types::*;
use resource::gl_primitive::GLPrimitive;

#[path = "../error.rs"]
mod error;

struct GLHandle {
    handle: GLuint
}

impl GLHandle {
    pub fn new(handle: GLuint) -> GLHandle {
        GLHandle {
            handle: handle
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
    trash:      bool,
    len:        usize,
    buf_type:   BufferType,
    alloc_type: AllocationType,
    handle:     Option<(usize, GLHandle)>,
    data:       Option<Vec<T>>,
}

// FIXME: implement Clone

impl<T: GLPrimitive> GPUVector<T> {
    /// Creates a new `GPUVector` that is not yet uploaded to the GPU.
    pub fn new(data: Vec<T>, buf_type: BufferType, alloc_type: AllocationType) -> GPUVector<T> {
        GPUVector {
            trash:      true,
            len:        data.len(),
            buf_type:   buf_type,
            alloc_type: alloc_type,
            handle:     None,
            data:       Some(data)
        }
    }

    /// The length of this vector.
    #[inline]
    pub fn len(&self) -> usize {
        if self.trash {
            match self.data {
                Some(ref d) => d.len(),
                None        => panic!("This should never happend.")

            }
        }
        else {
            self.len
        }
    }

    /// Mutably accesses the vector if it is available on RAM.
    ///
    /// This method will mark this vector as `trash`.
    #[inline]
    pub fn data_mut<'a>(&'a mut self) -> &'a mut Option<Vec<T>> {
        self.trash = true;

        &mut self.data
    }

    /// Immutably accesses the vector if it is available on RAM.
    #[inline]
    pub fn data<'a>(&'a self) -> &'a Option<Vec<T>> {
        &self.data
    }

    /// Returns `true` if this vector is already uploaded to the GPU.
    #[inline]
    pub fn is_on_gpu(&self) -> bool {
        self.handle.is_some()
    }

    /// Returns `true` if the cpu data and gpu data are out of sync.
    #[inline]
    pub fn trash(&self) -> bool {
        self.trash
    }

    /// Returns `true` if this vector is available on RAM.
    ///
    /// Note that a `GPUVector` may be both on RAM and on the GPU.
    #[inline]
    pub fn is_on_ram(&self) -> bool {
        self.data.is_some()
    }

    /// Loads the vector from the RAM to the GPU.
    ///
    /// If the vector is not available on RAM or already loaded to the GPU, nothing will happen.
    #[inline]
    pub fn load_to_gpu(&mut self) {
        if !self.is_on_gpu() {
            let buf_type   = self.buf_type;
            let alloc_type = self.alloc_type;
            let len        = &mut self.len;

            self.handle = self.data.as_ref().map(|d| {
                *len = d.len();
                (d.len(), GLHandle::new(upload_buffer(&d[..], buf_type, alloc_type)))
            });
        }
        else if self.trash() {
            for d in self.data.iter() {
                self.len = d.len();

                match self.handle {
                    None => { },
                    Some((ref mut len, ref handle)) => {
                        let handle = handle.handle();

                        *len = update_buffer(&d[..], *len, handle, self.buf_type, self.alloc_type)
                    }
                }
            }
        }

        self.trash = false;
    }

    /// Binds this vector to the appropriate gpu array.
    ///
    /// This does not associate this buffer with any shader attribute.
    #[inline]
    pub fn bind(&mut self) {
        self.load_to_gpu();

        let handle = self.handle.as_ref().map(|&(_, ref h)| h.handle()).expect("Could not bind the vector: data unavailable.");
        verify!(gl::BindBuffer(self.buf_type.to_gl(), handle));
    }

    /// Unbind this vector to the corresponding gpu buffer.
    #[inline]
    pub fn unbind(&mut self) {
        if self.is_on_gpu() {
            verify!(gl::BindBuffer(self.buf_type.to_gl(), 0));
        }
    }

    /// Loads the vector from the GPU to the RAM.
    ///
    /// If the vector is not available on the GPU or already loaded to the RAM, nothing will
    /// happen.
    #[inline]
    pub fn load_to_ram(&mut self) {
        if !self.is_on_ram() && self.is_on_gpu() {
            assert!(!self.trash);
            let     handle = self.handle.as_ref().map(|&(_, ref h)| h.handle()).unwrap();
            let mut data   = Vec::with_capacity(self.len);

            unsafe { data.set_len(self.len) };
            download_buffer(handle, self.buf_type, &mut data[..]);
            self.data = Some(data);
        }
    }

    /// Unloads this resource from the GPU.
    #[inline]
    pub fn unload_from_gpu(&mut self) {
        let _ = self.handle.as_ref().map(|&(_, ref h)| unsafe { verify!(gl::DeleteBuffers(1, &h.handle())) });
        self.len    = self.len();
        self.handle = None;
        self.trash  = false;
    }

    /// Removes this resource from the RAM.
    ///
    /// This is useful to save memory for vectors required on the GPU only.
    #[inline]
    pub fn unload_from_ram(&mut self) {
        if self.trash && self.is_on_gpu() {
            self.load_to_gpu();
        }

        self.data = None;
    }
}

impl<T: Clone + GLPrimitive> GPUVector<T> {
    /// Returns this vector as an owned vector if it is available on RAM.
    ///
    /// If it has been uploaded to the GPU, and unloaded from the RAM, call `load_to_ram` first to
    /// make the data accessible.
    #[inline]
    pub fn to_owned(&self) -> Option<Vec<T>> {
        self.data.as_ref().map(|d| d.clone())
    }
}

/// Type of gpu buffer.
#[derive(Clone, Copy)]
pub enum BufferType {
    /// An array buffer bindable to a gl::ARRAY_BUFFER.
    Array,
    /// An array buffer bindable to a gl::ELEMENT_ARRAY_BUFFER.
    ElementArray
}

impl BufferType {
    #[inline]
    fn to_gl(&self) -> GLuint {
        match *self {
            BufferType::Array        => gl::ARRAY_BUFFER,
            BufferType::ElementArray => gl::ELEMENT_ARRAY_BUFFER
        }
    }
}

/// Allocation type of gpu buffers.
#[derive(Clone, Copy)]
pub enum AllocationType {
    /// STATIC_DRAW allocation type.
    StaticDraw,
    /// DYNAMIC_DRAW allocation type.
    DynamicDraw,
    /// STREAM_DRAW allocation type.
    StreamDraw
}

impl AllocationType {
    #[inline]
    fn to_gl(&self) -> GLuint {
        match *self {
            AllocationType::StaticDraw  => gl::STATIC_DRAW,
            AllocationType::DynamicDraw => gl::DYNAMIC_DRAW,
            AllocationType::StreamDraw  => gl::STREAM_DRAW
        }
    }
}

/// Allocates and uploads a buffer to the gpu.
#[inline]
pub fn upload_buffer<T: GLPrimitive>(buf:             &[T],
                                     buf_type:        BufferType,
                                     allocation_type: AllocationType)
                                     -> GLuint {
    // Upload values of vertices
    let mut buf_id: GLuint = 0;

    unsafe {
        verify!(gl::GenBuffers(1, &mut buf_id));
        let _ = update_buffer(buf, 0, buf_id, buf_type, allocation_type);
    }

    buf_id
}

/// Downloads a buffer from the gpu.
///
/// 
#[inline]
pub fn download_buffer<T: GLPrimitive>(buf_id: GLuint, buf_type: BufferType, out: &mut [T]) {
    unsafe {
        verify!(gl::BindBuffer(buf_type.to_gl(), buf_id));
        verify!(gl::GetBufferSubData(
                buf_type.to_gl(),
                0,
                (out.len() * mem::size_of::<T>()) as GLsizeiptr,
                mem::transmute(&out[0])));
    }
}

/// Updates a buffer to the gpu.
///
/// Returns the number of element the bufer on the gpu can hold.
#[inline]
pub fn update_buffer<T: GLPrimitive>(buf:                 &[T],
                                     gpu_buf_len:         usize,
                                     gpu_buf_id:          GLuint,
                                     gpu_buf_type:        BufferType,
                                     gpu_allocation_type: AllocationType)
                                     -> usize {
    unsafe {
        verify!(gl::BindBuffer(gpu_buf_type.to_gl(), gpu_buf_id));

        if buf.len() < gpu_buf_len {
            verify!(gl::BufferSubData(
                    gpu_buf_type.to_gl(),
                    0,
                    (buf.len() * mem::size_of::<T>()) as GLsizeiptr,
                    mem::transmute(&buf[0])));

            gpu_buf_len
        }
        else {
            verify!(gl::BufferData(
                    gpu_buf_type.to_gl(),
                    (buf.len() * mem::size_of::<T>()) as GLsizeiptr,
                    mem::transmute(&buf[0]),
                    gpu_allocation_type.to_gl()));

            buf.len()
        }
    }
}
