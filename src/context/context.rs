use std::sync::{Once, ONCE_INIT};

#[cfg(not(target_arch = "wasm32"))]
use context::GLContext as ContextImpl;
#[cfg(target_arch = "wasm32")]
use context::WebGLContext as ContextImpl;

#[cfg(not(target_arch = "wasm32"))]
use gl;
#[cfg(target_arch = "wasm32")]
use webgl as gl;

use na::{Matrix2, Matrix3, Matrix4};
use resource::GLPrimitive;

#[path = "../error.rs"]
mod error;

pub type GLenum = gl::GLenum;
pub type GLintptr = gl::GLintptr;
pub struct UniformLocation(<ContextImpl as AbstractContext>::UniformLocation);
pub struct Buffer(<ContextImpl as AbstractContext>::Buffer);
pub struct Program(<ContextImpl as AbstractContext>::Program);
pub struct Shader(<ContextImpl as AbstractContext>::Shader);

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            let ctxt = Context::get();
            if ctxt.is_buffer(Some(&self)) {
                verify!(ctxt.delete_buffer(Some(&self)))
            }
        }
    }
}

#[derive(Clone)]
pub struct Context {
    ctxt: ContextImpl,
}

impl Context {
    pub const FLOAT: u32 = ContextImpl::FLOAT;
    pub const INT: u32 = ContextImpl::INT;
    pub const STATIC_DRAW: u32 = ContextImpl::STATIC_DRAW;
    pub const DYNAMIC_DRAW: u32 = ContextImpl::DYNAMIC_DRAW;
    pub const STREAM_DRAW: u32 = ContextImpl::STREAM_DRAW;
    pub const ARRAY_BUFFER: u32 = ContextImpl::ARRAY_BUFFER;
    pub const ELEMENT_ARRAY_BUFFER: u32 = ContextImpl::ELEMENT_ARRAY_BUFFER;
    pub const VERTEX_SHADER: u32 = ContextImpl::VERTEX_SHADER;
    pub const FRAGMENT_SHADER: u32 = ContextImpl::FRAGMENT_SHADER;

    pub fn get() -> Context {
        static mut CONTEXT_SINGLETON: Option<Context> = None;
        static INIT: Once = ONCE_INIT;

        unsafe {
            INIT.call_once(|| {
                CONTEXT_SINGLETON = Some(Context {
                    ctxt: ContextImpl::new(),
                });
            });

            CONTEXT_SINGLETON.clone().unwrap()
        }
    }

    pub fn get_error(&self) -> GLenum {
        self.ctxt.get_error()
    }

    pub fn uniform_matrix2fv(
        &self,
        location: Option<&UniformLocation>,
        transpose: bool,
        m: &Matrix2<f32>,
    ) {
        self.ctxt
            .uniform_matrix2fv(location.map(|e| &e.0), transpose, m)
    }

    pub fn uniform_matrix3fv(
        &self,
        location: Option<&UniformLocation>,
        transpose: bool,
        m: &Matrix3<f32>,
    ) {
        self.ctxt
            .uniform_matrix3fv(location.map(|e| &e.0), transpose, m)
    }

    pub fn uniform_matrix4fv(
        &self,
        location: Option<&UniformLocation>,
        transpose: bool,
        m: &Matrix4<f32>,
    ) {
        self.ctxt
            .uniform_matrix4fv(location.map(|e| &e.0), transpose, m)
    }

    pub fn uniform3f(&self, location: Option<&UniformLocation>, x: f32, y: f32, z: f32) {
        self.ctxt.uniform3f(location.map(|e| &e.0), x, y, z)
    }

    pub fn uniform2f(&self, location: Option<&UniformLocation>, x: f32, y: f32) {
        self.ctxt.uniform2f(location.map(|e| &e.0), x, y)
    }

    pub fn uniform1f(&self, location: Option<&UniformLocation>, x: f32) {
        self.ctxt.uniform1f(location.map(|e| &e.0), x)
    }

    pub fn uniform3i(&self, location: Option<&UniformLocation>, x: i32, y: i32, z: i32) {
        self.ctxt.uniform3i(location.map(|e| &e.0), x, y, z)
    }

    pub fn uniform2i(&self, location: Option<&UniformLocation>, x: i32, y: i32) {
        self.ctxt.uniform2i(location.map(|e| &e.0), x, y)
    }

    pub fn uniform1i(&self, location: Option<&UniformLocation>, x: i32) {
        self.ctxt.uniform1i(location.map(|e| &e.0), x)
    }

    pub fn create_buffer(&self) -> Option<Buffer> {
        self.ctxt.create_buffer().map(|e| Buffer(e))
    }

    pub fn delete_buffer(&self, buffer: Option<&Buffer>) {
        self.ctxt.delete_buffer(buffer.map(|e| &e.0))
    }

    pub fn bind_buffer(&self, target: GLenum, buffer: Option<&Buffer>) {
        self.ctxt.bind_buffer(target, buffer.map(|e| &e.0))
    }

    pub fn is_buffer(&self, buffer: Option<&Buffer>) -> bool {
        self.ctxt.is_buffer(buffer.map(|e| &e.0))
    }

    pub fn buffer_data<T: GLPrimitive>(&self, target: GLenum, data: &[T], usage: GLenum) {
        self.ctxt.buffer_data(target, data, usage)
    }

    pub fn buffer_sub_data<T: GLPrimitive>(&self, target: GLenum, offset: GLintptr, data: &[T]) {
        self.ctxt.buffer_sub_data(target, offset, data)
    }

    pub fn create_shader(&self, type_: GLenum) -> Option<Shader> {
        self.ctxt.create_shader(type_).map(|e| Shader(e))
    }

    pub fn create_program(&self) -> Option<Program> {
        self.ctxt.create_program().map(|e| Program(e))
    }

    pub fn delete_program(&self, program: Option<&Program>) {
        self.ctxt.delete_program(program.map(|e| &e.0))
    }

    pub fn delete_shader(&self, shader: Option<&Shader>) {
        self.ctxt.delete_shader(shader.map(|e| &e.0))
    }

    pub fn is_shader(&self, shader: Option<&Shader>) -> bool {
        self.ctxt.is_shader(shader.map(|e| &e.0))
    }
    pub fn is_program(&self, program: Option<&Program>) -> bool {
        self.ctxt.is_program(program.map(|e| &e.0))
    }

    pub fn shader_source(&self, shader: &Shader, source: &str) {
        self.ctxt.shader_source(&shader.0, source)
    }

    pub fn compile_shader(&self, shader: &Shader) {
        self.ctxt.compile_shader(&shader.0)
    }

    pub fn link_program(&self, program: &Program) {
        self.ctxt.link_program(&program.0)
    }

    pub fn use_program(&self, program: Option<&Program>) {
        self.ctxt.use_program(program.map(|e| &e.0))
    }

    pub fn attach_shader(&self, program: &Program, shader: &Shader) {
        self.ctxt.attach_shader(&program.0, &shader.0)
    }

    pub fn get_shader_parameter_int(&self, shader: &Shader, pname: GLenum) -> Option<i32> {
        self.ctxt.get_shader_parameter_int(&shader.0, pname)
    }

    pub fn get_shader_info_log(&self, shader: &Shader) -> Option<String> {
        self.ctxt.get_shader_info_log(&shader.0)
    }

    pub fn vertex_attrib_pointer(
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

    pub fn enable_vertex_attrib_array(&self, index: u32) {
        self.ctxt.enable_vertex_attrib_array(index)
    }

    pub fn disable_vertex_attrib_array(&self, index: u32) {
        self.ctxt.disable_vertex_attrib_array(index)
    }

    pub fn get_attrib_location(&self, program: &Program, name: &str) -> i32 {
        self.ctxt.get_attrib_location(&program.0, name)
    }

    pub fn get_uniform_location(&self, program: &Program, name: &str) -> Option<UniformLocation> {
        self.ctxt
            .get_uniform_location(&program.0, name)
            .map(|e| UniformLocation(e))
    }
}

pub(crate) trait AbstractContextConst {
    const FLOAT: u32;
    const INT: u32;
    const STATIC_DRAW: u32;
    const DYNAMIC_DRAW: u32;
    const STREAM_DRAW: u32;
    const ARRAY_BUFFER: u32;
    const ELEMENT_ARRAY_BUFFER: u32;
    const VERTEX_SHADER: u32;
    const FRAGMENT_SHADER: u32;
}

pub(crate) trait AbstractContext {
    type UniformLocation;
    type Buffer;
    type Shader;
    type Program;

    fn get_error(&self) -> GLenum;
    fn uniform_matrix2fv(
        &self,
        location: Option<&Self::UniformLocation>,
        transpose: bool,
        m: &Matrix2<f32>,
    );
    fn uniform_matrix3fv(
        &self,
        location: Option<&Self::UniformLocation>,
        transpose: bool,
        m: &Matrix3<f32>,
    );
    fn uniform_matrix4fv(
        &self,
        location: Option<&Self::UniformLocation>,
        transpose: bool,
        m: &Matrix4<f32>,
    );
    fn uniform3f(&self, location: Option<&Self::UniformLocation>, x: f32, y: f32, z: f32);
    fn uniform2f(&self, location: Option<&Self::UniformLocation>, x: f32, y: f32);
    fn uniform1f(&self, location: Option<&Self::UniformLocation>, x: f32);
    fn uniform3i(&self, location: Option<&Self::UniformLocation>, x: i32, y: i32, z: i32);
    fn uniform2i(&self, location: Option<&Self::UniformLocation>, x: i32, y: i32);
    fn uniform1i(&self, location: Option<&Self::UniformLocation>, x: i32);

    fn create_buffer(&self) -> Option<Self::Buffer>;
    fn delete_buffer(&self, buffer: Option<&Self::Buffer>);
    fn is_buffer(&self, buffer: Option<&Self::Buffer>) -> bool;
    fn bind_buffer(&self, target: GLenum, buffer: Option<&Self::Buffer>);
    fn buffer_data<T: GLPrimitive>(&self, target: GLenum, data: &[T], usage: GLenum);
    fn buffer_sub_data<T: GLPrimitive>(&self, target: GLenum, offset: GLintptr, data: &[T]);

    fn create_shader(&self, type_: GLenum) -> Option<Self::Shader>;
    fn create_program(&self) -> Option<Self::Program>;
    fn delete_program(&self, program: Option<&Self::Program>);
    fn delete_shader(&self, shader: Option<&Self::Shader>);
    fn is_shader(&self, shader: Option<&Self::Shader>) -> bool;
    fn is_program(&self, program: Option<&Self::Program>) -> bool;
    fn shader_source(&self, shader: &Self::Shader, source: &str);
    fn compile_shader(&self, shader: &Self::Shader);
    fn link_program(&self, program: &Self::Program);
    fn use_program(&self, program: Option<&Self::Program>);
    fn attach_shader(&self, program: &Self::Program, shader: &Self::Shader);
    fn get_shader_parameter_int(&self, shader: &Self::Shader, pname: GLenum) -> Option<i32>;
    fn get_shader_info_log(&self, shader: &Self::Shader) -> Option<String>;
    fn vertex_attrib_pointer(
        &self,
        index: u32,
        size: i32,
        type_: GLenum,
        normalized: bool,
        stride: i32,
        offset: GLintptr,
    );
    fn enable_vertex_attrib_array(&self, index: u32);
    fn disable_vertex_attrib_array(&self, index: u32);

    fn get_attrib_location(&self, program: &Self::Program, name: &str) -> i32;
    fn get_uniform_location(
        &self,
        program: &Self::Program,
        name: &str,
    ) -> Option<Self::UniformLocation>;
}
