use std::ffi::CString;
use std::iter;
use std::mem;
use std::ptr;

use context::{AbstractContext, AbstractContextConst, GLenum, GLintptr, GLsizeiptr};
use gl;
use num::Zero;

use na::{Matrix2, Matrix3, Matrix4};
use resource::GLPrimitive;

#[path = "../error.rs"]
mod error;

/// An OpenGL context.
#[derive(Clone)]
pub struct GLContext;

fn val<T: Copy + Zero>(val: Option<&T>) -> T {
    match val {
        Some(t) => *t,
        None => T::zero(),
    }
}

impl GLContext {
    /// Creates a new OpenGL context.
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
    const MIRRORED_REPEAT: u32 = gl::MIRRORED_REPEAT;
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
    const PROGRAM_POINT_SIZE: u32 = gl::PROGRAM_POINT_SIZE;
    const LINE: u32 = gl::LINE;
    const POINT: u32 = gl::POINT;
    const FILL: u32 = gl::FILL;
    const BLEND: u32 = gl::BLEND;
    const SRC_ALPHA: u32 = gl::SRC_ALPHA;
    const ONE_MINUS_SRC_ALPHA: u32 = gl::ONE_MINUS_SRC_ALPHA;
    const UNPACK_ALIGNMENT: u32 = gl::UNPACK_ALIGNMENT;
    const ALPHA: u32 = gl::ALPHA;
    const RED: u32 = gl::RED;
}

impl AbstractContext for GLContext {
    type UniformLocation = i32;
    type Buffer = u32;
    type Shader = u32;
    type Program = u32;
    type Framebuffer = u32;
    type Texture = u32;

    fn get_error(&self) -> GLenum {
        unsafe { gl::GetError() }
    }

    fn uniform_matrix2fv(
        &self,
        location: Option<&Self::UniformLocation>,
        transpose: bool,
        m: &Matrix2<f32>,
    ) {
        unsafe { gl::UniformMatrix2fv(val(location), 1, transpose as u8, mem::transmute(m)) }
    }

    fn uniform_matrix3fv(
        &self,
        location: Option<&Self::UniformLocation>,
        transpose: bool,
        m: &Matrix3<f32>,
    ) {
        unsafe { gl::UniformMatrix3fv(val(location), 1, transpose as u8, mem::transmute(m)) }
    }

    fn uniform_matrix4fv(
        &self,
        location: Option<&Self::UniformLocation>,
        transpose: bool,
        m: &Matrix4<f32>,
    ) {
        unsafe { gl::UniformMatrix4fv(val(location), 1, transpose as u8, mem::transmute(m)) }
    }

    fn uniform3f(&self, location: Option<&Self::UniformLocation>, x: f32, y: f32, z: f32) {
        unsafe { gl::Uniform3f(val(location), x, y, z) }
    }

    fn uniform2f(&self, location: Option<&Self::UniformLocation>, x: f32, y: f32) {
        unsafe { gl::Uniform2f(val(location), x, y) }
    }

    fn uniform1f(&self, location: Option<&Self::UniformLocation>, x: f32) {
        unsafe { gl::Uniform1f(val(location), x) }
    }

    fn uniform3i(&self, location: Option<&Self::UniformLocation>, x: i32, y: i32, z: i32) {
        unsafe { gl::Uniform3i(val(location), x, y, z) }
    }

    fn uniform2i(&self, location: Option<&Self::UniformLocation>, x: i32, y: i32) {
        unsafe { gl::Uniform2i(val(location), x, y) }
    }

    fn uniform1i(&self, location: Option<&Self::UniformLocation>, x: i32) {
        unsafe { gl::Uniform1i(val(location), x) }
    }

    fn create_buffer(&self) -> Option<Self::Buffer> {
        let mut buf = 0;
        unsafe { gl::GenBuffers(1, &mut buf) };
        checked!(buf)
    }

    fn delete_buffer(&self, buffer: Option<&Self::Buffer>) {
        unsafe { gl::DeleteBuffers(1, &val(buffer)) }
    }

    fn bind_buffer(&self, target: GLenum, buffer: Option<&Self::Buffer>) {
        unsafe { gl::BindBuffer(target, val(buffer)) }
    }

    fn is_buffer(&self, buffer: Option<&Self::Buffer>) -> bool {
        unsafe { gl::IsBuffer(val(buffer)) != 0 }
    }

    fn buffer_data<T: GLPrimitive>(&self, target: GLenum, data: &[T], usage: GLenum) {
        unsafe {
            gl::BufferData(
                target,
                (data.len() * mem::size_of::<T>()) as GLsizeiptr,
                mem::transmute(&data[0]),
                usage,
            )
        }
    }

    fn buffer_sub_data<T: GLPrimitive>(&self, target: GLenum, offset: GLintptr, data: &[T]) {
        unsafe {
            gl::BufferSubData(
                target,
                offset,
                (data.len() * mem::size_of::<T>()) as GLsizeiptr,
                mem::transmute(&data[0]),
            )
        }
    }

    fn create_shader(&self, type_: GLenum) -> Option<Self::Shader> {
        checked!(unsafe { gl::CreateShader(type_) })
    }

    fn create_program(&self) -> Option<Self::Program> {
        checked!(unsafe { gl::CreateProgram() })
    }

    fn delete_program(&self, program: Option<&Self::Program>) {
        unsafe { gl::DeleteProgram(val(program)) }
    }

    fn delete_shader(&self, shader: Option<&Self::Shader>) {
        unsafe { gl::DeleteShader(val(shader)) }
    }

    fn is_shader(&self, shader: Option<&Self::Shader>) -> bool {
        unsafe { gl::IsShader(val(shader)) != 0 }
    }

    fn is_program(&self, program: Option<&Self::Program>) -> bool {
        unsafe { gl::IsProgram(val(program)) != 0 }
    }

    fn shader_source(&self, shader: &Self::Shader, source: &str) {
        let source = CString::new(source.as_bytes()).unwrap();
        unsafe { gl::ShaderSource(*shader, 1, &source.as_ptr(), ptr::null()) }
    }

    fn compile_shader(&self, shader: &Self::Shader) {
        unsafe { gl::CompileShader(*shader) }
    }

    fn link_program(&self, program: &Self::Program) {
        unsafe { gl::LinkProgram(*program) }
    }

    fn use_program(&self, program: Option<&Self::Program>) {
        unsafe { gl::UseProgram(val(program)) }
    }

    fn attach_shader(&self, program: &Self::Program, shader: &Self::Shader) {
        unsafe { gl::AttachShader(*program, *shader) }
    }

    fn get_shader_parameter_int(&self, shader: &Self::Shader, pname: GLenum) -> Option<i32> {
        let mut res = 0;
        unsafe { gl::GetShaderiv(*shader, pname, &mut res) };
        Some(res)
    }

    fn get_shader_info_log(&self, shader: &Self::Shader) -> Option<String> {
        let mut info_log_len = 0;

        unsafe { gl::GetShaderiv(*shader, gl::INFO_LOG_LENGTH, &mut info_log_len) };

        if info_log_len > 0 {
            // Error check for memory allocation failure is omitted here.
            let mut chars_written = 0;
            let info_log: String = iter::repeat(' ').take(info_log_len as usize).collect();

            let c_str = CString::new(info_log.as_bytes()).unwrap();
            unsafe {
                gl::GetShaderInfoLog(
                    *shader,
                    info_log_len,
                    &mut chars_written,
                    c_str.as_ptr() as *mut _,
                )
            };

            let bytes = c_str.as_bytes();
            let bytes = &bytes[..bytes.len() - 1];
            String::from_utf8(bytes.to_vec()).ok()
        } else {
            None
        }
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
        unsafe {
            gl::VertexAttribPointer(
                index,
                size,
                type_,
                normalized as u8,
                stride,
                mem::transmute(offset),
            )
        }
    }

    fn enable_vertex_attrib_array(&self, index: u32) {
        unsafe { gl::EnableVertexAttribArray(index) }
    }

    fn disable_vertex_attrib_array(&self, index: u32) {
        unsafe { gl::DisableVertexAttribArray(index) }
    }

    fn get_attrib_location(&self, program: &Self::Program, name: &str) -> i32 {
        let c_str = CString::new(name.as_bytes()).expect("Invalid uniform name.");
        unsafe { gl::GetAttribLocation(*program, c_str.as_ptr()) }
    }

    fn get_uniform_location(
        &self,
        program: &Self::Program,
        name: &str,
    ) -> Option<Self::UniformLocation> {
        let c_str = CString::new(name.as_bytes()).expect("Invalid uniform name.");
        let location = unsafe { unsafe { gl::GetUniformLocation(*program, c_str.as_ptr()) } };

        if unsafe { gl::GetError() } == 0 && location != -1 {
            Some(location)
        } else {
            None
        }
    }

    fn viewport(&self, x: i32, y: i32, width: i32, height: i32) {
        unsafe { gl::Viewport(x, y, width, height) }
    }

    fn scissor(&self, x: i32, y: i32, width: i32, height: i32) {
        unsafe { gl::Scissor(x, y, width, height) }
    }

    fn create_framebuffer(&self) -> Option<Self::Framebuffer> {
        let mut fbo = 0;
        unsafe { gl::GenFramebuffers(1, &mut fbo) };
        checked!(fbo)
    }

    fn is_framebuffer(&self, framebuffer: Option<&Self::Framebuffer>) -> bool {
        unsafe { gl::IsFramebuffer(val(framebuffer)) != 0 }
    }

    fn bind_framebuffer(&self, target: GLenum, framebuffer: Option<&Self::Framebuffer>) {
        unsafe { gl::BindFramebuffer(target, val(framebuffer)) }
    }

    fn delete_framebuffer(&self, framebuffer: Option<&Self::Framebuffer>) {
        unsafe { gl::DeleteFramebuffers(1, &val(framebuffer)) }
    }

    fn framebuffer_texture2d(
        &self,
        target: GLenum,
        attachment: GLenum,
        textarget: GLenum,
        texture: Option<&Self::Texture>,
        level: i32,
    ) {
        unsafe { gl::FramebufferTexture2D(target, attachment, textarget, val(texture), level) }
    }

    fn bind_texture(&self, target: GLenum, texture: Option<&Self::Texture>) {
        unsafe { gl::BindTexture(target, val(texture)) }
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
            Some(pixels) => unsafe {
                gl::TexImage2D(
                    target,
                    level,
                    internalformat,
                    width,
                    height,
                    border,
                    format,
                    Self::UNSIGNED_BYTE,
                    mem::transmute(&pixels[0]),
                )
            },
            None => unsafe {
                gl::TexImage2D(
                    target,
                    level,
                    internalformat,
                    width,
                    height,
                    border,
                    format,
                    Self::UNSIGNED_BYTE,
                    ptr::null(),
                )
            },
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
            Some(pixels) => unsafe {
                gl::TexImage2D(
                    target,
                    level,
                    internalformat,
                    width,
                    height,
                    border,
                    format,
                    Self::UNSIGNED_INT,
                    mem::transmute(&pixels[0]),
                )
            },
            None => unsafe {
                gl::TexImage2D(
                    target,
                    level,
                    internalformat,
                    width,
                    height,
                    border,
                    format,
                    Self::UNSIGNED_INT,
                    ptr::null(),
                )
            },
        }
    }

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
    ) {
        match pixels {
            Some(pixels) => unsafe {
                gl::TexSubImage2D(
                    target,
                    level,
                    xoffset,
                    yoffset,
                    width,
                    height,
                    format,
                    Self::UNSIGNED_BYTE,
                    mem::transmute(&pixels[0]),
                )
            },
            None => unsafe {
                gl::TexSubImage2D(
                    target,
                    level,
                    xoffset,
                    yoffset,
                    width,
                    height,
                    format,
                    Self::UNSIGNED_BYTE,
                    ptr::null(),
                )
            },
        }
    }

    fn tex_parameteri(&self, target: GLenum, pname: GLenum, param: i32) {
        unsafe { gl::TexParameteri(target, pname, param) }
    }

    fn is_texture(&self, texture: Option<&Self::Texture>) -> bool {
        unsafe { gl::IsTexture(val(texture)) != 0 }
    }

    fn create_texture(&self) -> Option<Self::Texture> {
        let mut res = 0;
        unsafe { gl::GenTextures(1, &mut res) };
        checked!(res)
    }

    fn delete_texture(&self, texture: Option<&Self::Texture>) {
        unsafe { gl::DeleteTextures(1, &val(texture)) }
    }

    fn active_texture(&self, texture: GLenum) {
        unsafe { gl::ActiveTexture(texture) }
    }

    fn enable(&self, cap: GLenum) {
        unsafe { gl::Enable(cap) }
    }

    fn disable(&self, cap: GLenum) {
        unsafe { gl::Disable(cap) }
    }

    fn draw_elements(&self, mode: GLenum, count: i32, type_: GLenum, offset: GLintptr) {
        unsafe { gl::DrawElements(mode, count, type_, mem::transmute(offset)) }
    }

    fn draw_arrays(&self, mode: GLenum, first: i32, count: i32) {
        unsafe { gl::DrawArrays(mode, first, count) }
    }

    fn point_size(&self, size: f32) {
        unsafe { gl::PointSize(size) }
    }

    fn line_width(&self, width: f32) {
        unsafe { gl::LineWidth(width) }
    }

    fn clear(&self, mask: u32) {
        unsafe { gl::Clear(mask) }
    }

    fn clear_color(&self, r: f32, g: f32, b: f32, a: f32) {
        unsafe { gl::ClearColor(r, g, b, a) }
    }

    fn polygon_mode(&self, face: GLenum, mode: GLenum) -> bool {
        unsafe {
            gl::PolygonMode(face, mode);
        }

        true
    }

    fn front_face(&self, mode: GLenum) {
        unsafe { gl::FrontFace(mode) }
    }

    fn depth_func(&self, mode: GLenum) {
        unsafe { gl::DepthFunc(mode) }
    }

    fn cull_face(&self, mode: GLenum) {
        unsafe { gl::CullFace(mode) }
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
            // FIXME: this may segfault?
            unsafe {
                gl::ReadPixels(
                    x,
                    y,
                    width,
                    height,
                    format,
                    Self::UNSIGNED_BYTE,
                    mem::transmute(&mut pixels[0]),
                );
            }
        }
    }

    fn pixel_storei(&self, pname: GLenum, param: i32) {
        unsafe { gl::PixelStorei(pname, param) }
    }

    fn blend_func(&self, sfactor: GLenum, dfactor: GLenum) {
        unsafe { gl::BlendFunc(sfactor, dfactor) }
    }
}
