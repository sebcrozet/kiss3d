use std::sync::Once;

use context::{AbstractContext, AbstractContextConst, GLenum, GLintptr};
use stdweb::unstable::TryInto;
use stdweb::web::{self, html_element::CanvasElement, IParentNode, TypedArray};
use webgl::{WebGLBuffer, WebGLRenderingContext, WebGLUniformLocation};

use na::{Matrix2, Matrix3, Matrix4};
use resource::{GLPrimitive, PrimitiveArray};

#[derive(Clone)]
pub struct WebGLContext {
    ctxt: WebGLRenderingContext,
}

impl WebGLContext {
    pub fn new() -> Self {
        let canvas: CanvasElement = web::document()
            .query_selector("#canvas")
            .expect("No canvas found.")
            .unwrap()
            .try_into()
            .unwrap();
        let ctxt: WebGLRenderingContext = canvas.get_context().unwrap();

        WebGLContext { ctxt }
    }
}

impl AbstractContextConst for WebGLContext {
    const FLOAT: u32 = WebGLRenderingContext::FLOAT;
    const INT: u32 = WebGLRenderingContext::INT;
    const STATIC_DRAW: u32 = WebGLRenderingContext::STATIC_DRAW;
    const DYNAMIC_DRAW: u32 = WebGLRenderingContext::DYNAMIC_DRAW;
    const STREAM_DRAW: u32 = WebGLRenderingContext::STREAM_DRAW;
    const ARRAY_BUFFER: u32 = WebGLRenderingContext::ARRAY_BUFFER;
    const ELEMENT_ARRAY_BUFFER: u32 = WebGLRenderingContext::ELEMENT_ARRAY_BUFFER;
}

impl AbstractContext for WebGLContext {
    type UniformLocation = WebGLUniformLocation;
    type Buffer = WebGLBuffer;

    fn get_error(&self) -> GLenum {
        self.ctxt.get_error()
    }

    fn uniform_matrix2fv(
        &self,
        location: Option<&Self::UniformLocation>,
        transpose: bool,
        m: &Matrix2<f32>,
    ) {
        self.ctxt
            .uniform_matrix2fv(location, transpose, m.as_slice())
    }

    fn uniform_matrix3fv(
        &self,
        location: Option<&Self::UniformLocation>,
        transpose: bool,
        m: &Matrix3<f32>,
    ) {
        self.ctxt
            .uniform_matrix3fv(location, transpose, m.as_slice())
    }

    fn uniform_matrix4fv(
        &self,
        location: Option<&Self::UniformLocation>,
        transpose: bool,
        m: &Matrix4<f32>,
    ) {
        self.ctxt
            .uniform_matrix4fv(location, transpose, m.as_slice())
    }

    fn uniform3f(&self, location: Option<&Self::UniformLocation>, x: f32, y: f32, z: f32) {
        self.ctxt.uniform3f(location, x, y, z)
    }

    fn uniform2f(&self, location: Option<&Self::UniformLocation>, x: f32, y: f32) {
        self.ctxt.uniform2f(location, x, y)
    }

    fn uniform1f(&self, location: Option<&Self::UniformLocation>, x: f32) {
        self.ctxt.uniform1f(location, x)
    }

    fn uniform1i(&self, location: Option<&Self::UniformLocation>, x: i32) {
        self.ctxt.uniform1i(location, x)
    }

    fn create_buffer(&self) -> Option<Self::Buffer> {
        self.ctxt.create_buffer()
    }

    fn delete_buffer(&self, buffer: Option<&Self::Buffer>) {
        self.ctxt.delete_buffer(buffer)
    }

    fn bind_buffer(&self, target: GLenum, buffer: Option<&Self::Buffer>) {
        self.ctxt.bind_buffer(target, buffer)
    }

    fn is_buffer(&self, buffer: Option<&Self::Buffer>) -> bool {
        self.ctxt.is_buffer(buffer)
    }

    fn buffer_data<T: GLPrimitive>(&self, target: GLenum, data: &[T], usage: GLenum) {
        match T::flatten(data) {
            PrimitiveArray::Float32(arr) => {
                let abuf = TypedArray::<f32>::from(arr);
                self.ctxt.buffer_data_1(target, Some(&abuf.buffer()), usage)
            }
            PrimitiveArray::Int32(arr) => {
                let abuf = TypedArray::<i32>::from(arr);
                self.ctxt.buffer_data_1(target, Some(&abuf.buffer()), usage)
            }
        }
    }

    fn buffer_sub_data<T: GLPrimitive>(&self, target: GLenum, offset: GLintptr, data: &[T]) {
        match T::flatten(data) {
            PrimitiveArray::Float32(arr) => {
                let abuf = TypedArray::<f32>::from(arr);
                self.ctxt.buffer_sub_data(target, offset, &abuf.buffer())
            }
            PrimitiveArray::Int32(arr) => {
                let abuf = TypedArray::<i32>::from(arr);
                self.ctxt.buffer_sub_data(target, offset, &abuf.buffer())
            }
        }
    }
}
