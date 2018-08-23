#![allow(missing_docs)]

use std::sync::{Once, ONCE_INIT};

#[cfg(not(any(target_arch = "wasm32", target_arch = "asmjs")))]
use context::GLContext as ContextImpl;
#[cfg(any(target_arch = "wasm32", target_arch = "asmjs"))]
use context::WebGLContext as ContextImpl;

#[cfg(any(target_arch = "wasm32", target_arch = "asmjs"))]
use super::webgl_bindings::{
    GLenum as GLenumTy, GLintptr as GLintptrTy, GLsizeiptr as GLsizeiptrTy,
};
#[cfg(not(any(target_arch = "wasm32", target_arch = "asmjs")))]
use gl::{
    types::GLenum as GLenumTy, types::GLintptr as GLintptrTy, types::GLsizeiptr as GLsizeiptrTy,
};

use na::{Matrix2, Matrix3, Matrix4};
use resource::GLPrimitive;

#[path = "../error.rs"]
mod error;

pub type GLenum = GLenumTy;
pub type GLintptr = GLintptrTy;
pub type GLsizeiptr = GLsizeiptrTy;
pub struct UniformLocation(<ContextImpl as AbstractContext>::UniformLocation);
pub struct Buffer(<ContextImpl as AbstractContext>::Buffer);
pub struct Program(<ContextImpl as AbstractContext>::Program);
pub struct Shader(<ContextImpl as AbstractContext>::Shader);
pub struct Framebuffer(<ContextImpl as AbstractContext>::Framebuffer);
pub struct Texture(<ContextImpl as AbstractContext>::Texture);

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
    pub const UNSIGNED_INT: u32 = ContextImpl::UNSIGNED_INT;
    pub const UNSIGNED_SHORT: u32 = ContextImpl::UNSIGNED_SHORT;
    pub const STATIC_DRAW: u32 = ContextImpl::STATIC_DRAW;
    pub const DYNAMIC_DRAW: u32 = ContextImpl::DYNAMIC_DRAW;
    pub const STREAM_DRAW: u32 = ContextImpl::STREAM_DRAW;
    pub const ARRAY_BUFFER: u32 = ContextImpl::ARRAY_BUFFER;
    pub const ELEMENT_ARRAY_BUFFER: u32 = ContextImpl::ELEMENT_ARRAY_BUFFER;
    pub const VERTEX_SHADER: u32 = ContextImpl::VERTEX_SHADER;
    pub const FRAGMENT_SHADER: u32 = ContextImpl::FRAGMENT_SHADER;
    pub const COMPILE_STATUS: u32 = ContextImpl::COMPILE_STATUS;
    pub const FRAMEBUFFER: u32 = ContextImpl::FRAMEBUFFER;
    pub const DEPTH_ATTACHMENT: u32 = ContextImpl::DEPTH_ATTACHMENT;
    pub const COLOR_ATTACHMENT0: u32 = ContextImpl::COLOR_ATTACHMENT0;
    pub const TEXTURE_2D: u32 = ContextImpl::TEXTURE_2D;
    pub const DEPTH_COMPONENT: u32 = ContextImpl::DEPTH_COMPONENT;
    pub const UNSIGNED_BYTE: u32 = ContextImpl::UNSIGNED_BYTE;
    pub const TEXTURE_WRAP_S: u32 = ContextImpl::TEXTURE_WRAP_S;
    pub const TEXTURE_WRAP_T: u32 = ContextImpl::TEXTURE_WRAP_T;
    pub const TEXTURE_MIN_FILTER: u32 = ContextImpl::TEXTURE_MIN_FILTER;
    pub const TEXTURE_MAG_FILTER: u32 = ContextImpl::TEXTURE_MAG_FILTER;
    pub const LINEAR: u32 = ContextImpl::LINEAR;
    pub const CLAMP_TO_EDGE: u32 = ContextImpl::CLAMP_TO_EDGE;
    pub const RGB: u32 = ContextImpl::RGB;
    pub const RGBA: u32 = ContextImpl::RGBA;
    pub const TEXTURE0: u32 = ContextImpl::TEXTURE0;
    pub const TEXTURE1: u32 = ContextImpl::TEXTURE1;
    pub const REPEAT: u32 = ContextImpl::REPEAT;
    pub const MIRRORED_REPEAT: u32 = ContextImpl::MIRRORED_REPEAT;
    pub const LINEAR_MIPMAP_LINEAR: u32 = ContextImpl::LINEAR_MIPMAP_LINEAR;
    pub const TRIANGLES: u32 = ContextImpl::TRIANGLES;
    pub const CULL_FACE: u32 = ContextImpl::CULL_FACE;
    pub const FRONT_AND_BACK: u32 = ContextImpl::FRONT_AND_BACK;
    pub const FILL: u32 = ContextImpl::FILL;
    pub const LINE: u32 = ContextImpl::LINE;
    pub const POINT: u32 = ContextImpl::POINT;
    pub const LINES: u32 = ContextImpl::LINES;
    pub const POINTS: u32 = ContextImpl::POINTS;
    pub const TRIANGLE_STRIP: u32 = ContextImpl::TRIANGLE_STRIP;
    pub const COLOR_BUFFER_BIT: u32 = ContextImpl::COLOR_BUFFER_BIT;
    pub const DEPTH_BUFFER_BIT: u32 = ContextImpl::DEPTH_BUFFER_BIT;
    pub const CCW: u32 = ContextImpl::CCW;
    pub const DEPTH_TEST: u32 = ContextImpl::DEPTH_TEST;
    pub const SCISSOR_TEST: u32 = ContextImpl::SCISSOR_TEST;
    pub const PROGRAM_POINT_SIZE: u32 = ContextImpl::PROGRAM_POINT_SIZE;
    pub const LEQUAL: u32 = ContextImpl::LEQUAL;
    pub const BACK: u32 = ContextImpl::BACK;
    pub const PACK_ALIGNMENT: u32 = ContextImpl::PACK_ALIGNMENT;
    pub const BLEND: u32 = ContextImpl::BLEND;
    pub const SRC_ALPHA: u32 = ContextImpl::SRC_ALPHA;
    pub const ONE_MINUS_SRC_ALPHA: u32 = ContextImpl::ONE_MINUS_SRC_ALPHA;
    pub const UNPACK_ALIGNMENT: u32 = ContextImpl::UNPACK_ALIGNMENT;
    pub const ALPHA: u32 = ContextImpl::ALPHA;
    pub const RED: u32 = ContextImpl::RED;

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

    pub fn viewport(&self, x: i32, y: i32, width: i32, height: i32) {
        self.ctxt.viewport(x, y, width, height)
    }

    pub fn scissor(&self, x: i32, y: i32, width: i32, height: i32) {
        self.ctxt.scissor(x, y, width, height)
    }

    pub fn create_framebuffer(&self) -> Option<Framebuffer> {
        self.ctxt.create_framebuffer().map(|e| Framebuffer(e))
    }

    pub fn is_framebuffer(&self, framebuffer: Option<&Framebuffer>) -> bool {
        self.ctxt.is_framebuffer(framebuffer.map(|e| &e.0))
    }

    pub fn bind_framebuffer(&self, target: GLenum, framebuffer: Option<&Framebuffer>) {
        self.ctxt
            .bind_framebuffer(target, framebuffer.map(|e| &e.0))
    }

    pub fn delete_framebuffer(&self, framebuffer: Option<&Framebuffer>) {
        self.ctxt.delete_framebuffer(framebuffer.map(|e| &e.0))
    }

    pub fn framebuffer_texture2d(
        &self,
        target: GLenum,
        attachment: GLenum,
        textarget: GLenum,
        texture: Option<&Texture>,
        level: i32,
    ) {
        self.ctxt
            .framebuffer_texture2d(target, attachment, textarget, texture.map(|e| &e.0), level)
    }

    pub fn bind_texture(&self, target: GLenum, texture: Option<&Texture>) {
        self.ctxt.bind_texture(target, texture.map(|e| &e.0))
    }

    pub fn tex_image2d(
        &self,
        target: GLenum,
        level: i32,
        internalformat: i32,
        width: i32,
        height: i32,
        border: i32,
        format: GLenum,
        pixels: Option<&[u8]>,
    ) {
        self.ctxt.tex_image2d(
            target,
            level,
            internalformat,
            width,
            height,
            border,
            format,
            pixels,
        )
    }

    pub fn tex_image2di(
        &self,
        target: GLenum,
        level: i32,
        internalformat: i32,
        width: i32,
        height: i32,
        border: i32,
        format: GLenum,
        pixels: Option<&[i32]>,
    ) {
        self.ctxt.tex_image2di(
            target,
            level,
            internalformat,
            width,
            height,
            border,
            format,
            pixels,
        )
    }

    pub fn tex_sub_image2d(
        &self,
        target: GLenum,
        level: i32,
        xoffset: i32,
        yoffset: i32,
        width: i32,
        height: i32,
        format: GLenum,
        pixels: Option<&[u8]>,
    ) {
        self.ctxt.tex_sub_image2d(
            target, level, xoffset, yoffset, width, height, format, pixels,
        )
    }

    pub fn tex_parameteri(&self, target: GLenum, pname: GLenum, param: i32) {
        self.ctxt.tex_parameteri(target, pname, param)
    }

    pub fn is_texture(&self, texture: Option<&Texture>) -> bool {
        self.ctxt.is_texture(texture.map(|e| &e.0))
    }

    pub fn create_texture(&self) -> Option<Texture> {
        self.ctxt.create_texture().map(|e| Texture(e))
    }

    pub fn delete_texture(&self, texture: Option<&Texture>) {
        self.ctxt.delete_texture(texture.map(|e| &e.0))
    }

    pub fn active_texture(&self, texture: GLenum) {
        self.ctxt.active_texture(texture)
    }

    pub fn enable(&self, cap: GLenum) {
        self.ctxt.enable(cap)
    }

    pub fn disable(&self, cap: GLenum) {
        self.ctxt.disable(cap)
    }

    pub fn draw_elements(&self, mode: GLenum, count: i32, type_: GLenum, offset: GLintptr) {
        self.ctxt.draw_elements(mode, count, type_, offset)
    }

    pub fn draw_arrays(&self, mode: GLenum, first: i32, count: i32) {
        self.ctxt.draw_arrays(mode, first, count)
    }

    pub fn point_size(&self, size: f32) {
        self.ctxt.point_size(size)
    }

    pub fn line_width(&self, size: f32) {
        self.ctxt.line_width(size)
    }

    pub fn clear(&self, mask: u32) {
        self.ctxt.clear(mask)
    }

    pub fn clear_color(&self, r: f32, g: f32, b: f32, a: f32) {
        self.ctxt.clear_color(r, g, b, a)
    }

    pub fn polygon_mode(&self, face: GLenum, mode: GLenum) -> bool {
        self.ctxt.polygon_mode(face, mode)
    }

    pub fn front_face(&self, mode: GLenum) {
        self.ctxt.front_face(mode)
    }

    pub fn depth_func(&self, mode: GLenum) {
        self.ctxt.depth_func(mode)
    }

    pub fn cull_face(&self, mode: GLenum) {
        self.ctxt.cull_face(mode)
    }

    pub fn read_pixels(
        &self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        format: GLenum,
        pixels: Option<&mut [u8]>,
    ) {
        self.ctxt.read_pixels(x, y, width, height, format, pixels)
    }

    pub fn pixel_storei(&self, pname: GLenum, param: i32) {
        self.ctxt.pixel_storei(pname, param)
    }

    pub fn blend_func(&self, sfactor: GLenum, dfactor: GLenum) {
        self.ctxt.blend_func(sfactor, dfactor)
    }
}

pub(crate) trait AbstractContextConst {
    const FLOAT: u32;
    const INT: u32;
    const UNSIGNED_INT: u32;
    const UNSIGNED_SHORT: u32;
    const STATIC_DRAW: u32;
    const DYNAMIC_DRAW: u32;
    const STREAM_DRAW: u32;
    const ARRAY_BUFFER: u32;
    const ELEMENT_ARRAY_BUFFER: u32;
    const VERTEX_SHADER: u32;
    const FRAGMENT_SHADER: u32;
    const COMPILE_STATUS: u32;
    const FRAMEBUFFER: u32;
    const DEPTH_ATTACHMENT: u32;
    const COLOR_ATTACHMENT0: u32;
    const TEXTURE_2D: u32;
    const DEPTH_COMPONENT: u32;
    const UNSIGNED_BYTE: u32;
    const TEXTURE_WRAP_S: u32;
    const TEXTURE_WRAP_T: u32;
    const TEXTURE_MIN_FILTER: u32;
    const TEXTURE_MAG_FILTER: u32;
    const LINEAR: u32;
    const CLAMP_TO_EDGE: u32;
    const RGB: u32;
    const RGBA: u32;
    const TEXTURE0: u32;
    const TEXTURE1: u32;
    const REPEAT: u32;
    const MIRRORED_REPEAT: u32;
    const LINEAR_MIPMAP_LINEAR: u32;
    const TRIANGLES: u32;
    const CULL_FACE: u32;
    const FRONT_AND_BACK: u32;
    const FILL: u32;
    const LINE: u32;
    const POINT: u32;
    const LINES: u32;
    const POINTS: u32;
    const TRIANGLE_STRIP: u32;
    const COLOR_BUFFER_BIT: u32;
    const DEPTH_BUFFER_BIT: u32;
    const CCW: u32;
    const DEPTH_TEST: u32;
    const SCISSOR_TEST: u32;
    const PROGRAM_POINT_SIZE: u32;
    const LEQUAL: u32;
    const BACK: u32;
    const PACK_ALIGNMENT: u32;
    const BLEND: u32;
    const SRC_ALPHA: u32;
    const ONE_MINUS_SRC_ALPHA: u32;
    const UNPACK_ALIGNMENT: u32;
    const ALPHA: u32;
    const RED: u32;
}

pub(crate) trait AbstractContext {
    type UniformLocation;
    type Buffer;
    type Shader;
    type Program;
    type Texture;
    type Framebuffer;

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

    fn viewport(&self, x: i32, y: i32, width: i32, height: i32);
    fn scissor(&self, x: i32, y: i32, width: i32, height: i32);
    fn create_framebuffer(&self) -> Option<Self::Framebuffer>;
    fn is_framebuffer(&self, framebuffer: Option<&Self::Framebuffer>) -> bool;
    fn bind_framebuffer(&self, target: GLenum, framebuffer: Option<&Self::Framebuffer>);
    fn delete_framebuffer(&self, framebuffer: Option<&Self::Framebuffer>);
    fn framebuffer_texture2d(
        &self,
        target: GLenum,
        attachment: GLenum,
        textarget: GLenum,
        texture: Option<&Self::Texture>,
        level: i32,
    );
    fn bind_texture(&self, target: GLenum, texture: Option<&Self::Texture>);
    fn tex_image2d(
        &self,
        target: GLenum,
        level: i32,
        internalformat: i32,
        width: i32,
        height: i32,
        border: i32,
        format: GLenum,
        pixels: Option<&[u8]>,
    );
    fn tex_image2di(
        &self,
        target: GLenum,
        level: i32,
        internalformat: i32,
        width: i32,
        height: i32,
        border: i32,
        format: GLenum,
        pixels: Option<&[i32]>,
    );
    fn tex_sub_image2d(
        &self,
        target: GLenum,
        level: i32,
        xoffset: i32,
        yoffset: i32,
        width: i32,
        height: i32,
        format: GLenum,
        pixels: Option<&[u8]>,
    );
    fn tex_parameteri(&self, target: GLenum, pname: GLenum, param: i32);
    fn is_texture(&self, texture: Option<&Self::Texture>) -> bool;
    fn create_texture(&self) -> Option<Self::Texture>;
    fn delete_texture(&self, texture: Option<&Self::Texture>);
    fn active_texture(&self, texture: GLenum);

    fn enable(&self, cap: GLenum);
    fn disable(&self, cap: GLenum);

    fn draw_elements(&self, mode: GLenum, count: i32, type_: GLenum, offset: GLintptr);
    fn draw_arrays(&self, mode: GLenum, first: i32, count: i32);

    fn point_size(&self, size: f32);
    fn line_width(&self, size: f32);

    fn clear(&self, mask: u32);
    fn clear_color(&self, r: f32, g: f32, b: f32, a: f32);

    fn polygon_mode(&self, face: GLenum, mode: GLenum) -> bool;

    fn front_face(&self, mode: GLenum);
    fn depth_func(&self, mode: GLenum);
    fn cull_face(&self, mode: GLenum);

    fn read_pixels(
        &self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        format: GLenum,
        pixels: Option<&mut [u8]>,
    );
    fn pixel_storei(&self, pname: GLenum, param: i32);

    fn blend_func(&self, sfactor: GLenum, dfactor: GLenum);
}
