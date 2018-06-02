use std::rc::Rc;
use std::sync::Once;

use context::{AbstractContext, AbstractContextConst, GLenum, GLintptr};
use gl;

use na::{Matrix2, Matrix3, Matrix4};
use resource::{GLPrimitive, PrimitiveArray};

#[derive(Clone)]
pub struct GLContext;

impl GLContext {
    pub fn new() -> Self {
        GLContext
    }
}

impl AbstractContextConst for GLContext {
    const FLOAT: u32 = gl::FLOAT;
    const INT: u32 = gl::INT;
    const UNSIGNED_INT: u32 = gl::UNSIGNED_INT;
    const UNSIGNED_SHORT: u32 = gl::UNSIGNED_SHORT;
    const STATIC_DRAW: u32 = gl::STATIC_DRAW;
    const DYNAMIC_DRAW: u32 = gl::DYNAMIC_DRAW;
    const STREAM_DRAW: u32 = gl::STREAM_DRAW;
    const ARRAY_BUFFER: u32 = gl::ARRAY_BUFFER;
    const ELEMENT_ARRAY_BUFFER: u32 = gl::ELEMENT_ARRAY_BUFFER;
    const VERTEX_SHADER: u32 = gl::VERTEX_SHADER;
    const FRAGMENT_SHADER: u32 = gl::FRAGMENT_SHADER;
    const COMPILE_STATUS: u32 = gl::COMPILE_STATUS;
    const FRAMEBUFFER: u32 = gl::FRAMEBUFFER;
    const DEPTH_ATTACHMENT: u32 = gl::DEPTH_ATTACHMENT;
    const COLOR_ATTACHMENT0: u32 = gl::COLOR_ATTACHMENT0;
    const TEXTURE_2D: u32 = gl::TEXTURE_2D;
    const DEPTH_COMPONENT: u32 = gl::DEPTH_COMPONENT;
    const UNSIGNED_BYTE: u32 = gl::UNSIGNED_BYTE;
    const TEXTURE_WRAP_S: u32 = gl::TEXTURE_WRAP_S;
    const TEXTURE_WRAP_T: u32 = gl::TEXTURE_WRAP_T;
    const TEXTURE_MIN_FILTER: u32 = gl::TEXTURE_MIN_FILTER;
    const TEXTURE_MAG_FILTER: u32 = gl::TEXTURE_MAG_FILTER;
    const LINEAR: u32 = gl::LINEAR;
    const CLAMP_TO_EDGE: u32 = gl::CLAMP_TO_EDGE;
    const RGB: u32 = gl::RGB;
    const RGBA: u32 = gl::RGBA;
    const TEXTURE0: u32 = gl::TEXTURE0;
    const TEXTURE1: u32 = gl::TEXTURE1;
    const REPEAT: u32 = gl::REPEAT;
    const LINEAR_MIPMAP_LINEAR: u32 = gl::LINEAR_MIPMAP_LINEAR;
    const TRIANGLES: u32 = gl::TRIANGLES;
    const CULL_FACE: u32 = gl::CULL_FACE;
    const FRONT_AND_BACK: u32 = gl::FRONT_AND_BACK;
    const LINES: u32 = gl::LINES;
    const POINTS: u32 = gl::POINTS;
    const TRIANGLE_STRIP: u32 = gl::TRIANGLE_STRIP;
    const COLOR_BUFFER_BIT: u32 = gl::COLOR_BUFFER_BIT;
    const DEPTH_BUFFER_BIT: u32 = gl::DEPTH_BUFFER_BIT;
    const CCW: u32 = gl::CCW;
    const DEPTH_TEST: u32 = gl::DEPTH_TEST;
    const SCISSOR_TEST: u32 = gl::SCISSOR_TEST;
    const LEQUAL: u32 = gl::LEQUAL;
    const BACK: u32 = gl::BACK;
    const PACK_ALIGNMENT: u32 = gl::PACK_ALIGNMENT;

    // Not supported.
    const PROGRAM_POINT_SIZE: u32 = 0;
    const LINE: u32 = 0;
    const POINT: u32 = 0;
    const FILL: u32 = 0;
}

impl AbstractContext for GLContext {
    type UniformLocation = u32;
    type Buffer = u32;
    type Shader = u32;
    type Program = u32;
    type Framebuffer = u32;
    type Texture = u32;

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
            PrimitiveArray::UInt16(arr) => {
                let abuf = TypedArray::<u16>::from(arr);
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
            PrimitiveArray::UInt16(arr) => {
                let abuf = TypedArray::<u16>::from(arr);
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

    fn scissor(&self, x: i32, y: i32, width: i32, height: i32) {
        self.ctxt.scissor(x, y, width, height)
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
    ) {
        match pixels {
            Some(pixels) => self.ctxt.tex_image2_d(
                target,
                level,
                internalformat,
                width,
                height,
                border,
                format,
                Self::UNSIGNED_BYTE,
                Some(pixels),
            ),
            None => self.ctxt.tex_image2_d(
                target,
                level,
                internalformat,
                width,
                height,
                border,
                format,
                Self::UNSIGNED_BYTE,
                None::<&TypedArray<u8>>,
            ),
        }
    }

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
    ) {
        match pixels {
            Some(pixels) => self.ctxt.tex_image2_d(
                target,
                level,
                internalformat,
                width,
                height,
                border,
                format,
                Self::UNSIGNED_INT,
                Some(pixels),
            ),
            None => self.ctxt.tex_image2_d(
                target,
                level,
                internalformat,
                width,
                height,
                border,
                format,
                Self::UNSIGNED_INT,
                None::<&TypedArray<i32>>,
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

    fn enable(&self, cap: GLenum) {
        self.ctxt.enable(cap)
    }

    fn disable(&self, cap: GLenum) {
        self.ctxt.disable(cap)
    }

    fn draw_elements(&self, mode: GLenum, count: i32, type_: GLenum, offset: GLintptr) {
        self.ctxt.draw_elements(mode, count, type_, offset)
    }

    fn draw_arrays(&self, mode: GLenum, first: i32, count: i32) {
        self.ctxt.draw_arrays(mode, first, count)
    }

    fn point_size(&self, _: f32) {
        // Not supported.
    }

    fn line_width(&self, size: f32) {
        self.ctxt.line_width(size)
    }

    fn clear(&self, mask: u32) {
        self.ctxt.clear(mask)
    }

    fn clear_color(&self, r: f32, g: f32, b: f32, a: f32) {
        self.ctxt.clear_color(r, g, b, a)
    }

    fn polygon_mode(&self, _: GLenum, _: GLenum) {
        // Not supported.
    }

    fn front_face(&self, mode: GLenum) {
        self.front_face(mode)
    }

    fn depth_func(&self, mode: GLenum) {
        self.depth_func(mode)
    }

    fn cull_face(&self, mode: GLenum) {
        self.cull_face(mode)
    }

    fn read_pixels(
        &self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        format: GLenum,
        pixels: Option<&mut [u8]>,
    ) {
        if let Some(pixels) = pixels {
            let abuf = TypedArray::<u8>::from(&*pixels);
            self.ctxt.read_pixels(
                x,
                y,
                width,
                height,
                format,
                Self::UNSIGNED_BYTE,
                Some(&abuf),
            );
            let v = Vec::<u8>::from(abuf);
            pixels.copy_from_slice(&v[..]);
        }
    }

    fn pixel_storei(&self, pname: GLenum, param: i32) {
        self.ctxt.pixel_storei(pname, param)
    }
}
