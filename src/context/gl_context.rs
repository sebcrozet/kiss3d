use std::mem;
use std::sync::Arc;

use crate::context::{AbstractContext, AbstractContextConst, GLenum, GLintptr};

use crate::resource::GLPrimitive;
use glow::{Context, HasContext};
use na::{Matrix2, Matrix3, Matrix4};

#[path = "../error.rs"]
mod error;

/// An OpenGL context.
#[derive(Clone)]
pub struct GLContext {
    /// The underlying glow context.
    pub context: Arc<Context>,
}

impl GLContext {
    /// Creates a new OpenGL context.
    pub fn new(ctxt: Context) -> Self {
        Self {
            context: Arc::new(ctxt),
        }
    }
}

impl AbstractContextConst for GLContext {
    const FLOAT: u32 = glow::FLOAT;
    const INT: u32 = glow::INT;
    const UNSIGNED_INT: u32 = glow::UNSIGNED_INT;
    const UNSIGNED_SHORT: u32 = glow::UNSIGNED_SHORT;
    const STATIC_DRAW: u32 = glow::STATIC_DRAW;
    const DYNAMIC_DRAW: u32 = glow::DYNAMIC_DRAW;
    const STREAM_DRAW: u32 = glow::STREAM_DRAW;
    const ARRAY_BUFFER: u32 = glow::ARRAY_BUFFER;
    const ELEMENT_ARRAY_BUFFER: u32 = glow::ELEMENT_ARRAY_BUFFER;
    const VERTEX_SHADER: u32 = glow::VERTEX_SHADER;
    const FRAGMENT_SHADER: u32 = glow::FRAGMENT_SHADER;
    const COMPILE_STATUS: u32 = glow::COMPILE_STATUS;
    const FRAMEBUFFER: u32 = glow::FRAMEBUFFER;
    const RENDERBUFFER: u32 = glow::RENDERBUFFER;
    const DEPTH_ATTACHMENT: u32 = glow::DEPTH_ATTACHMENT;
    const COLOR_ATTACHMENT0: u32 = glow::COLOR_ATTACHMENT0;
    const TEXTURE_2D: u32 = glow::TEXTURE_2D;
    const DEPTH_COMPONENT: u32 = glow::DEPTH_COMPONENT;
    const DEPTH_COMPONENT16: u32 = glow::DEPTH_COMPONENT16;
    const UNSIGNED_BYTE: u32 = glow::UNSIGNED_BYTE;
    const TEXTURE_WRAP_S: u32 = glow::TEXTURE_WRAP_S;
    const TEXTURE_WRAP_T: u32 = glow::TEXTURE_WRAP_T;
    const TEXTURE_MIN_FILTER: u32 = glow::TEXTURE_MIN_FILTER;
    const TEXTURE_MAG_FILTER: u32 = glow::TEXTURE_MAG_FILTER;
    const LINEAR: u32 = glow::LINEAR;
    const NEAREST: u32 = glow::NEAREST;
    const CLAMP_TO_EDGE: u32 = glow::CLAMP_TO_EDGE;
    const RGB: u32 = glow::RGB;
    const RGBA: u32 = glow::RGBA;
    const TEXTURE0: u32 = glow::TEXTURE0;
    const TEXTURE1: u32 = glow::TEXTURE1;
    const REPEAT: u32 = glow::REPEAT;
    const MIRRORED_REPEAT: u32 = glow::MIRRORED_REPEAT;
    const LINEAR_MIPMAP_LINEAR: u32 = glow::LINEAR_MIPMAP_LINEAR;
    const TRIANGLES: u32 = glow::TRIANGLES;
    const CULL_FACE: u32 = glow::CULL_FACE;
    const FRONT_AND_BACK: u32 = glow::FRONT_AND_BACK;
    const LINES: u32 = glow::LINES;
    const POINTS: u32 = glow::POINTS;
    const TRIANGLE_STRIP: u32 = glow::TRIANGLE_STRIP;
    const COLOR_BUFFER_BIT: u32 = glow::COLOR_BUFFER_BIT;
    const DEPTH_BUFFER_BIT: u32 = glow::DEPTH_BUFFER_BIT;
    const CCW: u32 = glow::CCW;
    const DEPTH_TEST: u32 = glow::DEPTH_TEST;
    const SCISSOR_TEST: u32 = glow::SCISSOR_TEST;
    const LEQUAL: u32 = glow::LEQUAL;
    const BACK: u32 = glow::BACK;
    const PACK_ALIGNMENT: u32 = glow::PACK_ALIGNMENT;
    const PROGRAM_POINT_SIZE: u32 = glow::PROGRAM_POINT_SIZE;
    const LINE: u32 = glow::LINE;
    const POINT: u32 = glow::POINT;
    const FILL: u32 = glow::FILL;
    const BLEND: u32 = glow::BLEND;
    const SRC_ALPHA: u32 = glow::SRC_ALPHA;
    const ONE_MINUS_SRC_ALPHA: u32 = glow::ONE_MINUS_SRC_ALPHA;
    const ONE: u32 = glow::ONE;
    const UNPACK_ALIGNMENT: u32 = glow::UNPACK_ALIGNMENT;
    const ALPHA: u32 = glow::ALPHA;
    #[cfg(not(target_arch = "wasm32"))]
    const RED: u32 = glow::RED;
    #[cfg(target_arch = "wasm32")]
    const RED: u32 = glow::LUMINANCE; // WebGL 1
}

impl AbstractContext for GLContext {
    type UniformLocation = <Context as HasContext>::UniformLocation;
    type Buffer = <Context as HasContext>::Buffer;
    type Shader = <Context as HasContext>::Shader;
    type Program = <Context as HasContext>::Program;
    type Framebuffer = <Context as HasContext>::Framebuffer;
    type Renderbuffer = <Context as HasContext>::Renderbuffer;
    type Texture = <Context as HasContext>::Texture;
    type VertexArray = <Context as HasContext>::VertexArray;

    fn get_error(&self) -> GLenum {
        unsafe { self.context.get_error() }
    }

    fn uniform_matrix2fv(
        &self,
        location: Option<&Self::UniformLocation>,
        transpose: bool,
        m: &Matrix2<f32>,
    ) {
        unsafe {
            self.context.uniform_matrix_2_f32_slice(
                location,
                transpose,
                mem::transmute::<_, &[f32; 4]>(m),
            )
        }
    }

    fn uniform_matrix3fv(
        &self,
        location: Option<&Self::UniformLocation>,
        transpose: bool,
        m: &Matrix3<f32>,
    ) {
        unsafe {
            self.context.uniform_matrix_3_f32_slice(
                location,
                transpose,
                mem::transmute::<_, &[f32; 9]>(m),
            )
        }
    }

    fn uniform_matrix4fv(
        &self,
        location: Option<&Self::UniformLocation>,
        transpose: bool,
        m: &Matrix4<f32>,
    ) {
        unsafe {
            self.context.uniform_matrix_4_f32_slice(
                location,
                transpose,
                mem::transmute::<_, &[f32; 16]>(m),
            )
        }
    }

    fn uniform4f(&self, location: Option<&Self::UniformLocation>, x: f32, y: f32, z: f32, w: f32) {
        unsafe { self.context.uniform_4_f32(location, x, y, z, w) }
    }

    fn uniform3f(&self, location: Option<&Self::UniformLocation>, x: f32, y: f32, z: f32) {
        unsafe { self.context.uniform_3_f32(location, x, y, z) }
    }

    fn uniform2f(&self, location: Option<&Self::UniformLocation>, x: f32, y: f32) {
        unsafe { self.context.uniform_2_f32(location, x, y) }
    }

    fn uniform1f(&self, location: Option<&Self::UniformLocation>, x: f32) {
        unsafe { self.context.uniform_1_f32(location, x) }
    }

    fn uniform3i(&self, location: Option<&Self::UniformLocation>, x: i32, y: i32, z: i32) {
        unsafe { self.context.uniform_3_i32(location, x, y, z) }
    }

    fn uniform2i(&self, location: Option<&Self::UniformLocation>, x: i32, y: i32) {
        unsafe { self.context.uniform_2_i32(location, x, y) }
    }

    fn uniform1i(&self, location: Option<&Self::UniformLocation>, x: i32) {
        unsafe { self.context.uniform_1_i32(location, x) }
    }

    fn create_vertex_array(&self) -> Option<Self::VertexArray> {
        unsafe { self.context.create_vertex_array().ok() }
    }

    fn delete_vertex_array(&self, vertex_array: Option<&Self::VertexArray>) {
        if let Some(v) = vertex_array {
            unsafe { self.context.delete_vertex_array(v.clone()) }
        }
    }

    fn bind_vertex_array(&self, vertex_array: Option<&Self::VertexArray>) {
        unsafe { self.context.bind_vertex_array(vertex_array.cloned()) }
    }

    fn create_buffer(&self) -> Option<Self::Buffer> {
        unsafe { self.context.create_buffer().ok() }
    }

    fn delete_buffer(&self, buffer: Option<&Self::Buffer>) {
        if let Some(b) = buffer {
            unsafe { self.context.delete_buffer(b.clone()) }
        }
    }

    fn bind_buffer(&self, target: GLenum, buffer: Option<&Self::Buffer>) {
        unsafe { self.context.bind_buffer(target, buffer.cloned()) }
    }

    fn is_buffer(&self, buffer: Option<&Self::Buffer>) -> bool {
        if let Some(b) = buffer {
            unsafe { self.context.is_buffer(b.clone()) }
        } else {
            false
        }
    }

    fn buffer_data_uninitialized(&self, target: GLenum, len: usize, usage: GLenum) {
        unsafe { self.context.buffer_data_size(target, len as i32, usage) }
    }

    fn buffer_data<T: GLPrimitive>(&self, target: GLenum, data: &[T], usage: GLenum) {
        unsafe {
            let len = data.len() * mem::size_of::<T>();
            let ptr = data.as_ptr() as *const u8;
            let data = std::slice::from_raw_parts(ptr, len);
            self.context.buffer_data_u8_slice(target, data, usage)
        }
    }

    fn buffer_sub_data<T: GLPrimitive>(&self, target: GLenum, offset: u32, data: &[T]) {
        unsafe {
            let len = data.len() * mem::size_of::<T>();
            let ptr = data.as_ptr() as *const u8;
            let data: &[u8] = std::slice::from_raw_parts(ptr, len);
            self.context
                .buffer_sub_data_u8_slice(target, offset as i32, data)
        }
    }

    fn create_shader(&self, type_: GLenum) -> Option<Self::Shader> {
        unsafe { self.context.create_shader(type_).ok() }
    }

    fn create_program(&self) -> Option<Self::Program> {
        unsafe { self.context.create_program().ok() }
    }

    fn delete_program(&self, program: Option<&Self::Program>) {
        if let Some(p) = program {
            unsafe { self.context.delete_program(*p) }
        }
    }

    fn delete_shader(&self, shader: Option<&Self::Shader>) {
        if let Some(s) = shader {
            unsafe { self.context.delete_shader(*s) }
        }
    }

    fn is_shader(&self, shader: Option<&Self::Shader>) -> bool {
        if let Some(s) = shader {
            unsafe { self.context.is_shader(*s) }
        } else {
            false
        }
    }

    fn is_program(&self, program: Option<&Self::Program>) -> bool {
        if let Some(p) = program {
            unsafe { self.context.is_program(*p) }
        } else {
            false
        }
    }

    fn shader_source(&self, shader: &Self::Shader, source: &str) {
        unsafe { self.context.shader_source(*shader, source) }
    }

    fn compile_shader(&self, shader: &Self::Shader) {
        unsafe { self.context.compile_shader(*shader) }
    }

    fn link_program(&self, program: &Self::Program) {
        unsafe { self.context.link_program(*program) }
    }

    fn use_program(&self, program: Option<&Self::Program>) {
        unsafe { self.context.use_program(program.cloned()) }
    }

    fn attach_shader(&self, program: &Self::Program, shader: &Self::Shader) {
        unsafe { self.context.attach_shader(*program, *shader) }
    }

    fn get_shader_parameter_int(&self, shader: &Self::Shader, _pname: GLenum) -> Option<i32> {
        unsafe {
            if self.context.get_shader_compile_status(*shader) {
                Some(1)
            } else {
                Some(0)
            }
        }
    }

    fn get_shader_info_log(&self, shader: &Self::Shader) -> Option<String> {
        unsafe { Some(self.context.get_shader_info_log(*shader)) }
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
            self.context.vertex_attrib_pointer_f32(
                index,
                size,
                type_,
                normalized,
                stride,
                offset as i32,
            )
        }
    }

    fn enable_vertex_attrib_array(&self, index: u32) {
        unsafe { self.context.enable_vertex_attrib_array(index) }
    }

    fn disable_vertex_attrib_array(&self, index: u32) {
        unsafe { self.context.disable_vertex_attrib_array(index) }
    }

    fn get_attrib_location(&self, program: &Self::Program, name: &str) -> i32 {
        unsafe {
            self.context
                .get_attrib_location(*program, name)
                .unwrap_or(0) as i32
        }
    }

    fn get_uniform_location(
        &self,
        program: &Self::Program,
        name: &str,
    ) -> Option<Self::UniformLocation> {
        unsafe { self.context.get_uniform_location(*program, name) }
    }

    fn viewport(&self, x: i32, y: i32, width: i32, height: i32) {
        unsafe { self.context.viewport(x, y, width, height) }
    }

    fn scissor(&self, x: i32, y: i32, width: i32, height: i32) {
        unsafe { self.context.scissor(x, y, width, height) }
    }

    fn create_framebuffer(&self) -> Option<Self::Framebuffer> {
        unsafe { self.context.create_framebuffer().ok() }
    }

    fn is_framebuffer(&self, framebuffer: Option<&Self::Framebuffer>) -> bool {
        framebuffer.is_some()
    }

    fn bind_framebuffer(&self, target: GLenum, framebuffer: Option<&Self::Framebuffer>) {
        unsafe { self.context.bind_framebuffer(target, framebuffer.cloned()) }
    }

    fn delete_framebuffer(&self, framebuffer: Option<&Self::Framebuffer>) {
        if let Some(f) = framebuffer {
            unsafe { self.context.delete_framebuffer(*f) }
        }
    }

    fn framebuffer_texture2d(
        &self,
        target: GLenum,
        attachment: GLenum,
        textarget: GLenum,
        texture: Option<&Self::Texture>,
        level: i32,
    ) {
        unsafe {
            self.context.framebuffer_texture_2d(
                target,
                attachment,
                textarget,
                texture.cloned(),
                level,
            )
        }
    }

    fn create_renderbuffer(&self) -> Option<Self::Renderbuffer> {
        unsafe { self.context.create_renderbuffer().ok() }
    }

    fn is_renderbuffer(&self, buffer: Option<&Self::Renderbuffer>) -> bool {
        buffer.is_some()
    }

    fn delete_renderbuffer(&self, buffer: Option<&Self::Renderbuffer>) {
        if let Some(b) = buffer {
            unsafe { self.context.delete_renderbuffer(*b) }
        }
    }

    fn bind_renderbuffer(&self, buffer: Option<&Self::Renderbuffer>) {
        unsafe {
            self.context
                .bind_renderbuffer(Self::RENDERBUFFER, buffer.cloned())
        }
    }

    fn renderbuffer_storage(&self, internal_format: GLenum, width: i32, height: i32) {
        unsafe {
            self.context
                .renderbuffer_storage(Self::RENDERBUFFER, internal_format, width, height)
        }
    }

    fn framebuffer_renderbuffer(
        &self,
        attachment: GLenum,
        renderbuffer: Option<&Self::Renderbuffer>,
    ) {
        unsafe {
            self.context.framebuffer_renderbuffer(
                Self::FRAMEBUFFER,
                attachment,
                Self::RENDERBUFFER,
                renderbuffer.cloned(),
            )
        }
    }

    fn bind_texture(&self, target: GLenum, texture: Option<&Self::Texture>) {
        unsafe { self.context.bind_texture(target, texture.cloned()) }
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
        unsafe {
            self.context.tex_image_2d(
                target,
                level,
                internalformat,
                width,
                height,
                border,
                format,
                Self::UNSIGNED_BYTE,
                pixels,
            )
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
        unsafe {
            self.context.tex_image_2d(
                target,
                level,
                internalformat,
                width,
                height,
                border,
                format,
                Self::INT,
                pixels.map(|px| {
                    let len = px.len() * 4;
                    let ptr = px.as_ptr() as *const u8;
                    std::slice::from_raw_parts(ptr, len)
                }),
            )
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
        if let Some(pixels) = pixels {
            unsafe {
                self.context.tex_sub_image_2d(
                    target,
                    level,
                    xoffset,
                    yoffset,
                    width,
                    height,
                    format,
                    Self::UNSIGNED_BYTE,
                    glow::PixelUnpackData::Slice(pixels),
                )
            }
        }
    }

    fn tex_parameteri(&self, target: GLenum, pname: GLenum, param: i32) {
        unsafe { self.context.tex_parameter_i32(target, pname, param) }
    }

    fn is_texture(&self, texture: Option<&Self::Texture>) -> bool {
        if let Some(t) = texture {
            unsafe { self.context.is_texture(t.clone()) }
        } else {
            false
        }
    }

    fn create_texture(&self) -> Option<Self::Texture> {
        unsafe { self.context.create_texture().ok() }
    }

    fn delete_texture(&self, texture: Option<&Self::Texture>) {
        if let Some(t) = texture {
            unsafe { self.context.delete_texture(t.clone()) }
        }
    }

    fn active_texture(&self, texture: GLenum) {
        unsafe { self.context.active_texture(texture) }
    }

    fn enable(&self, cap: GLenum) {
        unsafe { self.context.enable(cap) }
    }

    fn disable(&self, cap: GLenum) {
        unsafe { self.context.disable(cap) }
    }

    fn draw_elements(&self, mode: GLenum, count: i32, type_: GLenum, offset: GLintptr) {
        unsafe {
            self.context
                .draw_elements(mode, count, type_, offset as i32)
        }
    }

    fn draw_arrays(&self, mode: GLenum, first: i32, count: i32) {
        unsafe { self.context.draw_arrays(mode, first, count) }
    }

    fn point_size(&self, _size: f32) {
        //        unsafe { self.context.point_size(size) }
    }

    fn line_width(&self, width: f32) {
        unsafe { self.context.line_width(width) }
    }

    fn clear(&self, mask: u32) {
        unsafe { self.context.clear(mask) }
    }

    fn clear_color(&self, r: f32, g: f32, b: f32, a: f32) {
        unsafe { self.context.clear_color(r, g, b, a) }
    }

    fn polygon_mode(&self, face: GLenum, mode: GLenum) -> bool {
        unsafe {
            self.context.polygon_mode(face, mode);
        }

        true
    }

    fn front_face(&self, mode: GLenum) {
        unsafe { self.context.front_face(mode) }
    }

    fn depth_func(&self, mode: GLenum) {
        unsafe { self.context.depth_func(mode) }
    }

    fn cull_face(&self, mode: GLenum) {
        unsafe { self.context.cull_face(mode) }
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
                self.context.read_pixels(
                    x,
                    y,
                    width,
                    height,
                    format,
                    Self::UNSIGNED_BYTE,
                    glow::PixelPackData::Slice(pixels),
                );
            }
        }
    }

    fn pixel_storei(&self, pname: GLenum, param: i32) {
        unsafe { self.context.pixel_store_i32(pname, param) }
    }

    fn blend_func_separate(
        &self,
        src_rgb: GLenum,
        dst_rgb: GLenum,
        src_alpha: GLenum,
        dst_alpha: GLenum,
    ) {
        unsafe {
            self.context
                .blend_func_separate(src_rgb, dst_rgb, src_alpha, dst_alpha)
        }
    }
}
