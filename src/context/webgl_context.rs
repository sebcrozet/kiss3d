use std::sync::Once;

use context::{AbstractContext, AbstractContextConst, GLenum, GLintptr};
use stdweb::web::{self, html_element::CanvasElement, IParentNode, TypedArray};
use stdweb::{unstable::TryInto, Value};
use webgl::{
    WebGLBuffer, WebGLFramebuffer, WebGLProgram, WebGLRenderingContext, WebGLShader, WebGLTexture,
    WebGLUniformLocation,
};

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
    const VERTEX_SHADER: u32 = WebGLRenderingContext::VERTEX_SHADER;
    const FRAGMENT_SHADER: u32 = WebGLRenderingContext::FRAGMENT_SHADER;
    const COMPILE_STATUS: u32 = WebGLRenderingContext::COMPILE_STATUS;
    const FRAMEBUFFER: u32 = WebGLRenderingContext::FRAMEBUFFER;
    const DEPTH_ATTACHMENT: u32 = WebGLRenderingContext::DEPTH_ATTACHMENT;
    const COLOR_ATTACHMENT0: u32 = WebGLRenderingContext::COLOR_ATTACHMENT0;
    const TEXTURE_2D: u32 = WebGLRenderingContext::TEXTURE_2D;
    const DEPTH_COMPONENT: u32 = WebGLRenderingContext::DEPTH_COMPONENT;
    const UNSIGNED_BYTE: u32 = WebGLRenderingContext::UNSIGNED_BYTE;
    const TEXTURE_WRAP_S: u32 = WebGLRenderingContext::TEXTURE_WRAP_S;
    const TEXTURE_WRAP_T: u32 = WebGLRenderingContext::TEXTURE_WRAP_T;
    const TEXTURE_MIN_FILTER: u32 = WebGLRenderingContext::TEXTURE_MIN_FILTER;
    const TEXTURE_MAG_FILTER: u32 = WebGLRenderingContext::TEXTURE_MAG_FILTER;
    const LINEAR: u32 = WebGLRenderingContext::LINEAR;
    const CLAMP_TO_EDGE: u32 = WebGLRenderingContext::CLAMP_TO_EDGE;
    const RGBA: u32 = WebGLRenderingContext::RGBA;
    const TEXTURE0: u32 = WebGLRenderingContext::TEXTURE0;
    const TEXTURE1: u32 = WebGLRenderingContext::TEXTURE1;
}

impl AbstractContext for WebGLContext {
    type UniformLocation = WebGLUniformLocation;
    type Buffer = WebGLBuffer;
    type Shader = WebGLShader;
    type Program = WebGLProgram;
    type Framebuffer = WebGLFramebuffer;
    type Texture = WebGLTexture;

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

    fn uniform3i(&self, location: Option<&Self::UniformLocation>, x: i32, y: i32, z: i32) {
        self.ctxt.uniform3i(location, x, y, z)
    }

    fn uniform2i(&self, location: Option<&Self::UniformLocation>, x: i32, y: i32) {
        self.ctxt.uniform2i(location, x, y)
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

    fn create_shader(&self, type_: GLenum) -> Option<Self::Shader> {
        self.ctxt.create_shader(type_)
    }

    fn create_program(&self) -> Option<Self::Program> {
        self.ctxt.create_program()
    }

    fn delete_program(&self, program: Option<&Self::Program>) {
        self.ctxt.delete_program(program)
    }

    fn delete_shader(&self, shader: Option<&Self::Shader>) {
        self.ctxt.delete_shader(shader)
    }

    fn is_shader(&self, shader: Option<&Self::Shader>) -> bool {
        self.ctxt.is_shader(shader)
    }

    fn is_program(&self, program: Option<&Self::Program>) -> bool {
        self.ctxt.is_program(program)
    }

    fn shader_source(&self, shader: &Self::Shader, source: &str) {
        self.ctxt.shader_source(shader, source)
    }

    fn compile_shader(&self, shader: &Self::Shader) {
        self.ctxt.compile_shader(shader)
    }

    fn link_program(&self, program: &Self::Program) {
        self.ctxt.link_program(program)
    }

    fn use_program(&self, program: Option<&Self::Program>) {
        self.ctxt.use_program(program)
    }

    fn attach_shader(&self, program: &Self::Program, shader: &Self::Shader) {
        self.ctxt.attach_shader(program, shader)
    }

    fn get_shader_parameter_int(&self, shader: &Self::Shader, pname: GLenum) -> Option<i32> {
        match self.ctxt.get_shader_parameter(shader, pname) {
            Value::Number(n) => n.try_into().ok(),
            _ => None,
        }
    }

    fn get_shader_info_log(&self, shader: &Self::Shader) -> Option<String> {
        self.ctxt.get_shader_info_log(shader)
    }

    fn vertex_attrib_pointer(
        &self,
        index: u32,
        size: i32,
        type_: GLenum,
        normalized: bool,
        stride: i32,
        offset: GLintptr,
    ) {
        self.ctxt
            .vertex_attrib_pointer(index, size, type_, normalized, stride, offset)
    }

    fn enable_vertex_attrib_array(&self, index: u32) {
        self.ctxt.enable_vertex_attrib_array(index)
    }

    fn disable_vertex_attrib_array(&self, index: u32) {
        self.ctxt.disable_vertex_attrib_array(index)
    }

    fn get_attrib_location(&self, program: &Self::Program, name: &str) -> i32 {
        self.ctxt.get_attrib_location(program, name)
    }

    fn get_uniform_location(
        &self,
        program: &Self::Program,
        name: &str,
    ) -> Option<Self::UniformLocation> {
        self.ctxt.get_uniform_location(program, name)
    }

    fn viewport(&self, x: i32, y: i32, width: i32, height: i32) {
        self.ctxt.viewport(x, y, width, height)
    }

    fn create_framebuffer(&self) -> Option<Self::Framebuffer> {
        self.ctxt.create_framebuffer()
    }

    fn is_framebuffer(&self, framebuffer: Option<&Self::Framebuffer>) -> bool {
        self.ctxt.is_framebuffer(framebuffer)
    }

    fn bind_framebuffer(&self, target: GLenum, framebuffer: Option<&Self::Framebuffer>) {
        self.ctxt.bind_framebuffer(target, framebuffer)
    }

    fn delete_framebuffer(&self, framebuffer: Option<&Self::Framebuffer>) {
        self.ctxt.delete_framebuffer(framebuffer)
    }

    fn framebuffer_texture2d(
        &self,
        target: GLenum,
        attachment: GLenum,
        textarget: GLenum,
        texture: Option<&Self::Texture>,
        level: i32,
    ) {
        self.ctxt
            .framebuffer_texture2_d(target, attachment, textarget, texture, level)
    }

    fn bind_texture(&self, target: GLenum, texture: Option<&Self::Texture>) {
        self.ctxt.bind_texture(target, texture)
    }

    fn tex_image2d<T: GLPrimitive>(
        &self,
        target: GLenum,
        level: i32,
        internalformat: i32,
        width: i32,
        height: i32,
        border: i32,
        format: GLenum,
        type_: GLenum,
        pixels: Option<&[T]>,
    ) {
        match pixels {
            Some(pixels) => match T::flatten(pixels) {
                PrimitiveArray::Float32(arr) => {
                    let abuf = TypedArray::<f32>::from(arr);
                    self.ctxt.tex_image2_d(
                        target,
                        level,
                        internalformat,
                        width,
                        height,
                        border,
                        format,
                        type_,
                        Some(&abuf.buffer()),
                    )
                }
                PrimitiveArray::Int32(arr) => {
                    let abuf = TypedArray::<i32>::from(arr);
                    self.ctxt.tex_image2_d(
                        target,
                        level,
                        internalformat,
                        width,
                        height,
                        border,
                        format,
                        type_,
                        Some(&abuf.buffer()),
                    )
                }
            },
            None => self.ctxt.tex_image2_d(
                target,
                level,
                internalformat,
                width,
                height,
                border,
                format,
                type_,
                None,
            ),
        }
    }

    fn tex_parameteri(&self, target: GLenum, pname: GLenum, param: i32) {
        self.ctxt.tex_parameteri(target, pname, param)
    }

    fn is_texture(&self, texture: Option<&Self::Texture>) -> bool {
        self.ctxt.is_texture(texture)
    }

    fn create_texture(&self) -> Option<Self::Texture> {
        self.ctxt.create_texture()
    }

    fn delete_texture(&self, texture: Option<&Self::Texture>) {
        self.ctxt.delete_texture(texture)
    }

    fn active_texture(&self, texture: GLenum) {
        self.ctxt.active_texture(texture)
    }
}
