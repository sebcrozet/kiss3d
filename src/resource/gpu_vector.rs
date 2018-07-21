//! Wrapper for an OpenGL buffer object.

use context::{Buffer, Context};
use resource::gl_primitive::GLPrimitive;

#[path = "../error.rs"]
mod error;

// FIXME: generalize this for any resource: GPUResource
/// A vector of elements that can be loaded to the GPU, on the RAM, or both.
pub struct GPUVec<T> {
    trash: bool,
    len: usize,
    buf_type: BufferType,
    alloc_type: AllocationType,
    buffer: Option<(usize, Buffer)>,
    data: Option<Vec<T>>,
}

// FIXME: implement Clone
impl<T: GLPrimitive> GPUVec<T> {
    /// Creates a new `GPUVec` that is not yet uploaded to the GPU.
    pub fn new(data: Vec<T>, buf_type: BufferType, alloc_type: AllocationType) -> GPUVec<T> {
        GPUVec {
            trash: true,
            len: data.len(),
            buf_type: buf_type,
            alloc_type: alloc_type,
            buffer: None,
            data: Some(data),
        }
    }

    /// The length of this vector.
    #[inline]
    pub fn len(&self) -> usize {
        if self.trash {
            match self.data {
                Some(ref d) => d.len(),
                None => panic!("This should never happend."),
            }
        } else {
            self.len
        }
    }

    /// Mutably accesses the vector if it is available on RAM.
    ///
    /// This method will mark this vector as `trash`.
    #[inline]
    pub fn data_mut(&mut self) -> &mut Option<Vec<T>> {
        self.trash = true;

        &mut self.data
    }

    /// Immutably accesses the vector if it is available on RAM.
    #[inline]
    pub fn data(&self) -> &Option<Vec<T>> {
        &self.data
    }

    /// Returns `true` if this vector is already uploaded to the GPU.
    #[inline]
    pub fn is_on_gpu(&self) -> bool {
        self.buffer.is_some()
    }

    /// Returns `true` if the cpu data and gpu data are out of sync.
    #[inline]
    pub fn trash(&self) -> bool {
        self.trash
    }

    /// Returns `true` if this vector is available on RAM.
    ///
    /// Note that a `GPUVec` may be both on RAM and on the GPU.
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
            let buf_type = self.buf_type;
            let alloc_type = self.alloc_type;
            let len = &mut self.len;

            self.buffer = self.data.as_ref().map(|d| {
                *len = d.len();
                (d.len(), upload_array(&d[..], buf_type, alloc_type))
            });
        } else if self.trash() {
            for d in self.data.iter() {
                self.len = d.len();

                if let Some((ref mut len, ref buffer)) = self.buffer {
                    *len = update_buffer(&d[..], *len, buffer, self.buf_type, self.alloc_type)
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

        let buffer = self.buffer.as_ref().map(|e| &e.1);
        verify!(Context::get().bind_buffer(self.buf_type.to_gl(), buffer));
    }

    /// Unbind this vector to the corresponding gpu buffer.
    #[inline]
    pub fn unbind(&mut self) {
        if self.is_on_gpu() {
            verify!(Context::get().bind_buffer(self.buf_type.to_gl(), None));
        }
    }

    // /// Loads the vector from the GPU to the RAM.
    // ///
    // /// If the vector is not available on the GPU or already loaded to the RAM, nothing will
    // /// happen.
    // #[inline]
    // pub fn load_to_ram(&mut self) {
    //     if !self.is_on_ram() && self.is_on_gpu() {
    //         assert!(!self.trash);
    //         let handle = self.buffer.as_ref().unwrap().1;
    //         let mut data = Vec::with_capacity(self.len);

    //         unsafe { data.set_len(self.len) };
    //         download_buffer(handle, self.buf_type, &mut data[..]);
    //         self.data = Some(data);
    //     }
    // }

    /// Unloads this resource from the GPU.
    #[inline]
    pub fn unload_from_gpu(&mut self) {
        let _ = self
            .buffer
            .as_ref()
            .map(|&(_, ref h)| unsafe { verify!(Context::get().delete_buffer(Some(h))) });
        self.len = self.len();
        self.buffer = None;
        self.trash = false;
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

impl<T: Clone + GLPrimitive> GPUVec<T> {
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
    ElementArray,
}

impl BufferType {
    #[inline]
    fn to_gl(&self) -> u32 {
        match *self {
            BufferType::Array => Context::ARRAY_BUFFER,
            BufferType::ElementArray => Context::ELEMENT_ARRAY_BUFFER,
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
    StreamDraw,
}

impl AllocationType {
    #[inline]
    fn to_gl(&self) -> u32 {
        match *self {
            AllocationType::StaticDraw => Context::STATIC_DRAW,
            AllocationType::DynamicDraw => Context::DYNAMIC_DRAW,
            AllocationType::StreamDraw => Context::STREAM_DRAW,
        }
    }
}

/// Allocates and uploads a buffer to the gpu.
#[inline]
pub fn upload_array<T: GLPrimitive>(
    arr: &[T],
    buf_type: BufferType,
    allocation_type: AllocationType,
) -> Buffer {
    // Upload values of vertices
    let buf = verify!(
        Context::get()
            .create_buffer()
            .expect("Could not create GPU buffer.")
    );
    let _ = update_buffer(arr, 0, &buf, buf_type, allocation_type);
    buf
}

// XXX: this requires webgl 2.0.
// /// Downloads a buffer from the gpu.
// #[inline]
// pub fn download_buffer<T: GLPrimitive>(buf_id: u32, buf_type: BufferType, out: &mut [T]) {
//     unsafe {
//         verify!(gl::BindBuffer(buf_type.to_gl(), buf_id));
//         verify!(gl::GetBufferSubData(
//             buf_type.to_gl(),
//             0,
//             (out.len() * mem::size_of::<T>()) as GLsizeiptr,
//             mem::transmute(&out[0])
//         ));
//     }
// }

/// Updates a buffer to the gpu.
///
/// Returns the number of element the bufer on the gpu can hold.
#[inline]
pub fn update_buffer<T: GLPrimitive>(
    arr: &[T],
    gpu_buf_len: usize,
    gpu_buf: &Buffer,
    gpu_buf_type: BufferType,
    gpu_allocation_type: AllocationType,
) -> usize {
    unsafe {
        let ctxt = Context::get();

        verify!(ctxt.bind_buffer(gpu_buf_type.to_gl(), Some(gpu_buf)));

        if arr.len() < gpu_buf_len {
            verify!(ctxt.buffer_sub_data(gpu_buf_type.to_gl(), 0, arr));
            gpu_buf_len
        } else {
            verify!(ctxt.buffer_data(gpu_buf_type.to_gl(), arr, gpu_allocation_type.to_gl()));
            arr.len()
        }
    }
}
