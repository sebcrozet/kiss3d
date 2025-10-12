use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::context::{AbstractContext, AbstractContextConst, GLenum, GLintptr};
use crate::resource::GLPrimitive;
use na::{Matrix2, Matrix3, Matrix4};
use wgpu::util::DeviceExt;

thread_local! {
    static RENDER_STATE: RefCell<Option<RenderState>> = RefCell::new(None);
    static ACTIVE_RENDER_PASS: RefCell<Option<ActiveRenderPass>> = RefCell::new(None);
}

struct RenderState {
    encoder: Option<wgpu::CommandEncoder>,
    surface_texture: Option<wgpu::SurfaceTexture>,
    depth_texture: Option<wgpu::Texture>,
    depth_view: Option<wgpu::TextureView>,
}

impl RenderState {
    fn new() -> Self {
        Self {
            encoder: None,
            surface_texture: None,
            depth_texture: None,
            depth_view: None,
        }
    }
}

struct ActiveRenderPass {
    // We can't store the actual RenderPass here due to lifetimes,
    // but we can store the data needed to create it
    needs_new_pass: bool,
    pending_pipeline: Option<Arc<wgpu::RenderPipeline>>,
    pending_index_buffer: Option<Arc<wgpu::Buffer>>,
    pending_vertex_buffers: Vec<Arc<wgpu::Buffer>>,
    pending_bind_group: Option<Arc<wgpu::BindGroup>>,
}

/// A WebGPU context that emulates OpenGL-style stateful operations.
#[derive(Clone)]
pub struct WgpuContext {
    inner: Arc<Mutex<WgpuContextInner>>,
}

pub struct WgpuContextInner {
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,

    // Emulated OpenGL state
    bound_array_buffer: Option<WgpuBuffer>,
    bound_element_buffer: Option<WgpuBuffer>,
    last_bound_buffer: Option<WgpuBuffer>, // Track most recent binding
    bound_program: Option<WgpuProgram>,
    bound_framebuffer: Option<WgpuFramebuffer>,
    bound_texture_2d: Option<WgpuTexture>,
    active_texture_unit: u32,

    // Track most recently bound array buffers for vertex binding
    recent_array_buffers: Vec<WgpuBuffer>,

    // Default resources for rendering
    default_bind_group: Option<Arc<wgpu::BindGroup>>,
    default_texture: Option<wgpu::Texture>,
    default_sampler: Option<wgpu::Sampler>,
    default_uniform_buffer: Option<wgpu::Buffer>,

    // Current pipeline state
    depth_test_enabled: bool,
    cull_face_enabled: bool,
    blend_enabled: bool,
    scissor_test_enabled: bool,

    // Viewport
    viewport: (i32, i32, i32, i32),
    scissor: (i32, i32, i32, i32),
    clear_color: (f32, f32, f32, f32),

    // Resource tracking
    buffers: HashMap<u64, Arc<wgpu::Buffer>>,
    textures: HashMap<u64, Arc<wgpu::Texture>>,
    shaders: HashMap<u64, WgpuShader>,
    programs: HashMap<u64, WgpuProgramData>,

    next_buffer_id: u64,
    next_texture_id: u64,
    next_shader_id: u64,
    next_program_id: u64,
}

// Opaque handles that wrap u64 IDs
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WgpuBuffer(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WgpuTexture(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WgpuShaderHandle(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WgpuProgram(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WgpuFramebuffer(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WgpuRenderbuffer(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WgpuVertexArray(u64);

#[derive(Clone, Debug)]
pub struct WgpuUniformLocation {
    program: u64,
    name: String,
}

struct WgpuShader {
    source: String,
    shader_type: u32,
    compiled: bool,
    info_log: String,
}

struct WgpuProgramData {
    vertex_shader: Option<u64>,
    fragment_shader: Option<u64>,
    linked: bool,
    pipeline: Option<Arc<wgpu::RenderPipeline>>,
    bind_group_layout: Option<wgpu::BindGroupLayout>,
    uniform_buffer: Option<wgpu::Buffer>,
    uniforms: HashMap<String, UniformValue>,
}

#[derive(Clone, Debug)]
enum UniformValue {
    Float(f32),
    Float2(f32, f32),
    Float3(f32, f32, f32),
    Float4(f32, f32, f32, f32),
    Int(i32),
    Int2(i32, i32),
    Int3(i32, i32, i32),
    Matrix2(Matrix2<f32>),
    Matrix3(Matrix3<f32>),
    Matrix4(Matrix4<f32>),
}

impl WgpuContext {
    /// Creates a new WebGPU context from a device and queue.
    pub fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        Self {
            inner: Arc::new(Mutex::new(WgpuContextInner {
                device: Arc::new(device),
                queue: Arc::new(queue),
                bound_array_buffer: None,
                bound_element_buffer: None,
                last_bound_buffer: None,
                bound_program: None,
                bound_framebuffer: None,
                bound_texture_2d: None,
                active_texture_unit: 0,
                recent_array_buffers: Vec::new(),
                default_bind_group: None,
                default_texture: None,
                default_sampler: None,
                default_uniform_buffer: None,
                depth_test_enabled: false,
                cull_face_enabled: false,
                blend_enabled: false,
                scissor_test_enabled: false,
                viewport: (0, 0, 800, 600),
                scissor: (0, 0, 800, 600),
                clear_color: (0.0, 0.0, 0.0, 1.0),
                buffers: HashMap::new(),
                textures: HashMap::new(),
                shaders: HashMap::new(),
                programs: HashMap::new(),
                next_buffer_id: 1,
                next_texture_id: 1,
                next_shader_id: 1,
                next_program_id: 1,
            })),
        }
    }

    pub fn device(&self) -> Arc<wgpu::Device> {
        self.inner.lock().unwrap().device.clone()
    }

    pub fn queue(&self) -> Arc<wgpu::Queue> {
        self.inner.lock().unwrap().queue.clone()
    }

    /// Begin a new render pass
    pub fn begin_frame(&self, surface_texture: wgpu::SurfaceTexture) {
        RENDER_STATE.with(|rs| {
            let mut state = rs.borrow_mut();
            if state.is_none() {
                *state = Some(RenderState::new());
            }

            if let Some(ref mut render_state) = *state {
                // Create depth texture if needed
                let size = surface_texture.texture.size();
                if render_state.depth_texture.is_none() ||
                   render_state.depth_texture.as_ref().unwrap().size().width != size.width ||
                   render_state.depth_texture.as_ref().unwrap().size().height != size.height {

                    let depth_texture = self.device().create_texture(&wgpu::TextureDescriptor {
                        label: Some("Depth Texture"),
                        size,
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: wgpu::TextureFormat::Depth24Plus,
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                        view_formats: &[],
                    });

                    let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
                    render_state.depth_texture = Some(depth_texture);
                    render_state.depth_view = Some(depth_view);
                }

                // Create command encoder
                let encoder = self.device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });
                render_state.encoder = Some(encoder);
                render_state.surface_texture = Some(surface_texture);
            }
        });

        ACTIVE_RENDER_PASS.with(|arp| {
            *arp.borrow_mut() = Some(ActiveRenderPass {
                needs_new_pass: true,
                pending_pipeline: None,
                pending_index_buffer: None,
                pending_vertex_buffers: Vec::new(),
                pending_bind_group: None,
            });
        });
    }

    /// End the current render pass and submit
    pub fn end_frame(&self) {
        RENDER_STATE.with(|rs| {
            let mut state = rs.borrow_mut();
            if let Some(render_state) = state.take() {
                // Submit the command buffer
                if let Some(encoder) = render_state.encoder {
                    self.queue().submit(Some(encoder.finish()));
                }
                // Present the surface texture
                if let Some(texture) = render_state.surface_texture {
                    texture.present();
                }
            }
        });
    }

    /// Execute a function with access to the current render pass
    /// Pipeline and buffers should be set via the pending fields before calling this
    pub fn with_render_pass_exec<F>(&self, f: F)
    where
        F: for<'a> FnOnce(&mut wgpu::RenderPass<'a>),
    {
        // Get pending resources from thread-local
        let (pipeline, index_buffer, vertex_buffers, bind_group) = ACTIVE_RENDER_PASS.with(|arp| {
            let pass = arp.borrow();
            if let Some(ref active_pass) = *pass {
                (
                    active_pass.pending_pipeline.clone(),
                    active_pass.pending_index_buffer.clone(),
                    active_pass.pending_vertex_buffers.clone(),
                    active_pass.pending_bind_group.clone(),
                )
            } else {
                (None, None, Vec::new(), None)
            }
        });

        RENDER_STATE.with(|rs| {
            let mut state = rs.borrow_mut();
            if let Some(ref mut render_state) = *state {
                if let (Some(ref mut encoder), Some(ref surface_texture), Some(ref depth_view)) =
                    (&mut render_state.encoder, &render_state.surface_texture, &render_state.depth_view)
                {
                    let view = surface_texture
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());

                    let clear_color = self.inner.lock().unwrap().clear_color;

                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Main Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: clear_color.0 as f64,
                                    g: clear_color.1 as f64,
                                    b: clear_color.2 as f64,
                                    a: clear_color.3 as f64,
                                }),
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: depth_view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: true,
                            }),
                            stencil_ops: None,
                        }),
                    });

                    // Set pending resources
                    if let Some(ref pipeline) = pipeline {
                        render_pass.set_pipeline(pipeline);
                    }
                    if let Some(ref index_buf) = index_buffer {
                        render_pass.set_index_buffer(index_buf.slice(..), wgpu::IndexFormat::Uint16);
                    }
                    // Set all vertex buffers
                    for (slot, vertex_buf) in vertex_buffers.iter().enumerate() {
                        render_pass.set_vertex_buffer(slot as u32, vertex_buf.slice(..));
                    }

                    // Set bind group if available
                    if let Some(ref bg) = bind_group {
                        render_pass.set_bind_group(0, bg, &[]);
                    }

                    // Execute custom commands
                    f(&mut render_pass);
                }
            }
        });

        // Clear pending resources
        ACTIVE_RENDER_PASS.with(|arp| {
            if let Some(ref mut pass) = *arp.borrow_mut() {
                pass.pending_pipeline = None;
                pass.pending_index_buffer = None;
                pass.pending_vertex_buffers.clear();
                pass.pending_bind_group = None;
            }
        });
    }
}

// Implement constant mappings (these are just placeholders to maintain API compatibility)
impl AbstractContextConst for WgpuContext {
    const FLOAT: u32 = 0x1406; // GL_FLOAT
    const INT: u32 = 0x1404;
    const UNSIGNED_INT: u32 = 0x1405;
    const UNSIGNED_SHORT: u32 = 0x1403;
    const STATIC_DRAW: u32 = 0x88E4;
    const DYNAMIC_DRAW: u32 = 0x88E8;
    const STREAM_DRAW: u32 = 0x88E0;
    const ARRAY_BUFFER: u32 = 0x8892;
    const ELEMENT_ARRAY_BUFFER: u32 = 0x8893;
    const VERTEX_SHADER: u32 = 0x8B31;
    const FRAGMENT_SHADER: u32 = 0x8B30;
    const COMPILE_STATUS: u32 = 0x8B81;
    const FRAMEBUFFER: u32 = 0x8D40;
    const RENDERBUFFER: u32 = 0x8D41;
    const DEPTH_ATTACHMENT: u32 = 0x8D00;
    const COLOR_ATTACHMENT0: u32 = 0x8CE0;
    const TEXTURE_2D: u32 = 0x0DE1;
    const DEPTH_COMPONENT: u32 = 0x1902;
    const DEPTH_COMPONENT16: u32 = 0x81A5;
    const UNSIGNED_BYTE: u32 = 0x1401;
    const TEXTURE_WRAP_S: u32 = 0x2802;
    const TEXTURE_WRAP_T: u32 = 0x2803;
    const TEXTURE_MIN_FILTER: u32 = 0x2801;
    const TEXTURE_MAG_FILTER: u32 = 0x2800;
    const LINEAR: u32 = 0x2601;
    const NEAREST: u32 = 0x2600;
    const CLAMP_TO_EDGE: u32 = 0x812F;
    const RGB: u32 = 0x1907;
    const RGBA: u32 = 0x1908;
    const TEXTURE0: u32 = 0x84C0;
    const TEXTURE1: u32 = 0x84C1;
    const REPEAT: u32 = 0x2901;
    const MIRRORED_REPEAT: u32 = 0x8370;
    const LINEAR_MIPMAP_LINEAR: u32 = 0x2703;
    const TRIANGLES: u32 = 0x0004;
    const CULL_FACE: u32 = 0x0B44;
    const FRONT_AND_BACK: u32 = 0x0408;
    const LINES: u32 = 0x0001;
    const POINTS: u32 = 0x0000;
    const TRIANGLE_STRIP: u32 = 0x0005;
    const COLOR_BUFFER_BIT: u32 = 0x4000;
    const DEPTH_BUFFER_BIT: u32 = 0x0100;
    const CCW: u32 = 0x0901;
    const DEPTH_TEST: u32 = 0x0B71;
    const SCISSOR_TEST: u32 = 0x0C11;
    const LEQUAL: u32 = 0x0203;
    const BACK: u32 = 0x0405;
    const PACK_ALIGNMENT: u32 = 0x0D05;
    const PROGRAM_POINT_SIZE: u32 = 0x8642;
    const LINE: u32 = 0x1B01;
    const POINT: u32 = 0x1B00;
    const FILL: u32 = 0x1B02;
    const BLEND: u32 = 0x0BE2;
    const SRC_ALPHA: u32 = 0x0302;
    const ONE_MINUS_SRC_ALPHA: u32 = 0x0303;
    const ONE: u32 = 1;
    const UNPACK_ALIGNMENT: u32 = 0x0CF5;
    const ALPHA: u32 = 0x1906;
    const RED: u32 = 0x1903;
}

// Implement the AbstractContext trait with WebGPU-backed operations
impl AbstractContext for WgpuContext {
    type UniformLocation = WgpuUniformLocation;
    type Buffer = WgpuBuffer;
    type Shader = WgpuShaderHandle;
    type Program = WgpuProgram;
    type Framebuffer = WgpuFramebuffer;
    type Renderbuffer = WgpuRenderbuffer;
    type Texture = WgpuTexture;
    type VertexArray = WgpuVertexArray;

    fn get_error(&self) -> GLenum {
        // WebGPU doesn't have synchronous error checking
        0
    }

    fn uniform_matrix2fv(
        &self,
        location: Option<&Self::UniformLocation>,
        _transpose: bool,
        m: &Matrix2<f32>,
    ) {
        if let Some(loc) = location {
            let mut inner = self.inner.lock().unwrap();
            if let Some(program) = inner.programs.get_mut(&loc.program) {
                program.uniforms.insert(loc.name.clone(), UniformValue::Matrix2(*m));
            }
        }
    }

    fn uniform_matrix3fv(
        &self,
        location: Option<&Self::UniformLocation>,
        _transpose: bool,
        m: &Matrix3<f32>,
    ) {
        if let Some(loc) = location {
            let mut inner = self.inner.lock().unwrap();
            if let Some(program) = inner.programs.get_mut(&loc.program) {
                program.uniforms.insert(loc.name.clone(), UniformValue::Matrix3(*m));
            }
        }
    }

    fn uniform_matrix4fv(
        &self,
        location: Option<&Self::UniformLocation>,
        _transpose: bool,
        m: &Matrix4<f32>,
    ) {
        if let Some(loc) = location {
            let mut inner = self.inner.lock().unwrap();
            if let Some(program) = inner.programs.get_mut(&loc.program) {
                program.uniforms.insert(loc.name.clone(), UniformValue::Matrix4(*m));
            }
        }
    }

    fn uniform4f(&self, location: Option<&Self::UniformLocation>, x: f32, y: f32, z: f32, w: f32) {
        if let Some(loc) = location {
            let mut inner = self.inner.lock().unwrap();
            if let Some(program) = inner.programs.get_mut(&loc.program) {
                program.uniforms.insert(loc.name.clone(), UniformValue::Float4(x, y, z, w));
            }
        }
    }

    fn uniform3f(&self, location: Option<&Self::UniformLocation>, x: f32, y: f32, z: f32) {
        if let Some(loc) = location {
            let mut inner = self.inner.lock().unwrap();
            if let Some(program) = inner.programs.get_mut(&loc.program) {
                program.uniforms.insert(loc.name.clone(), UniformValue::Float3(x, y, z));
            }
        }
    }

    fn uniform2f(&self, location: Option<&Self::UniformLocation>, x: f32, y: f32) {
        if let Some(loc) = location {
            let mut inner = self.inner.lock().unwrap();
            if let Some(program) = inner.programs.get_mut(&loc.program) {
                program.uniforms.insert(loc.name.clone(), UniformValue::Float2(x, y));
            }
        }
    }

    fn uniform1f(&self, location: Option<&Self::UniformLocation>, x: f32) {
        if let Some(loc) = location {
            let mut inner = self.inner.lock().unwrap();
            if let Some(program) = inner.programs.get_mut(&loc.program) {
                program.uniforms.insert(loc.name.clone(), UniformValue::Float(x));
            }
        }
    }

    fn uniform3i(&self, location: Option<&Self::UniformLocation>, x: i32, y: i32, z: i32) {
        if let Some(loc) = location {
            let mut inner = self.inner.lock().unwrap();
            if let Some(program) = inner.programs.get_mut(&loc.program) {
                program.uniforms.insert(loc.name.clone(), UniformValue::Int3(x, y, z));
            }
        }
    }

    fn uniform2i(&self, location: Option<&Self::UniformLocation>, x: i32, y: i32) {
        if let Some(loc) = location {
            let mut inner = self.inner.lock().unwrap();
            if let Some(program) = inner.programs.get_mut(&loc.program) {
                program.uniforms.insert(loc.name.clone(), UniformValue::Int2(x, y));
            }
        }
    }

    fn uniform1i(&self, location: Option<&Self::UniformLocation>, x: i32) {
        if let Some(loc) = location {
            let mut inner = self.inner.lock().unwrap();
            if let Some(program) = inner.programs.get_mut(&loc.program) {
                program.uniforms.insert(loc.name.clone(), UniformValue::Int(x));
            }
        }
    }

    fn create_vertex_array(&self) -> Option<Self::VertexArray> {
        // WebGPU doesn't have vertex array objects; state is part of pipeline
        Some(WgpuVertexArray(0))
    }

    fn delete_vertex_array(&self, _vertex_array: Option<&Self::VertexArray>) {
        // No-op
    }

    fn bind_vertex_array(&self, _vertex_array: Option<&Self::VertexArray>) {
        // No-op - vertex state is part of pipeline in wgpu
    }

    fn create_buffer(&self) -> Option<Self::Buffer> {
        let mut inner = self.inner.lock().unwrap();
        let id = inner.next_buffer_id;
        inner.next_buffer_id += 1;
        Some(WgpuBuffer(id))
    }

    fn delete_buffer(&self, buffer: Option<&Self::Buffer>) {
        if let Some(buf) = buffer {
            let mut inner = self.inner.lock().unwrap();
            inner.buffers.remove(&buf.0);
        }
    }

    fn bind_buffer(&self, target: GLenum, buffer: Option<&Self::Buffer>) {
        let mut inner = self.inner.lock().unwrap();
        if target == Self::ARRAY_BUFFER {
            inner.bound_array_buffer = buffer.copied();
            inner.last_bound_buffer = buffer.copied(); // Track most recent
            // Track array buffer bindings for vertex buffer setup
            if let Some(buf) = buffer.copied() {
                // Keep last few bindings (for multiple vertex buffers)
                if !inner.recent_array_buffers.contains(&buf) {
                    inner.recent_array_buffers.push(buf);
                    // Keep only last 8 (enough for most vertex layouts)
                    if inner.recent_array_buffers.len() > 8 {
                        inner.recent_array_buffers.remove(0);
                    }
                }
            }
        } else if target == Self::ELEMENT_ARRAY_BUFFER {
            inner.bound_element_buffer = buffer.copied();
            inner.last_bound_buffer = buffer.copied(); // Track most recent
            if let Some(b) = buffer {
                // Index buffer bound
            }
        }
    }

    fn is_buffer(&self, buffer: Option<&Self::Buffer>) -> bool {
        if let Some(buf) = buffer {
            self.inner.lock().unwrap().buffers.contains_key(&buf.0)
        } else {
            false
        }
    }

    fn buffer_data_uninitialized(&self, _target: GLenum, len: usize, _usage: GLenum) {
        let inner = self.inner.lock().unwrap();
        let bound_buffer = inner.bound_array_buffer.or(inner.bound_element_buffer);

        if let Some(buf_handle) = bound_buffer {
            let buffer = inner.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Uninitialized Buffer"),
                size: len as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            drop(inner);
            self.inner.lock().unwrap().buffers.insert(buf_handle.0, Arc::new(buffer));
        }
    }

    fn buffer_data<T: GLPrimitive>(&self, _target: GLenum, data: &[T], _usage: GLenum) {
        let inner = self.inner.lock().unwrap();
        // Use the most recently bound buffer (regardless of type)
        let bound_buffer = inner.last_bound_buffer;

        if let Some(buf_handle) = bound_buffer {
            let byte_data = unsafe {
                std::slice::from_raw_parts(
                    data.as_ptr() as *const u8,
                    data.len() * std::mem::size_of::<T>(),
                )
            };

            let buffer = inner.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Data Buffer"),
                contents: byte_data,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            });

            drop(inner);
            self.inner.lock().unwrap().buffers.insert(buf_handle.0, Arc::new(buffer));
        }
    }

    fn buffer_sub_data<T: GLPrimitive>(&self, _target: GLenum, offset: u32, data: &[T]) {
        let inner = self.inner.lock().unwrap();
        let bound_buffer = inner.bound_array_buffer.or(inner.bound_element_buffer);

        if let Some(buf_handle) = bound_buffer {
            if let Some(buffer) = inner.buffers.get(&buf_handle.0) {
                let byte_data = unsafe {
                    std::slice::from_raw_parts(
                        data.as_ptr() as *const u8,
                        data.len() * std::mem::size_of::<T>(),
                    )
                };
                inner.queue.write_buffer(buffer, offset as u64, byte_data);
            }
        }
    }

    fn create_shader(&self, type_: GLenum) -> Option<Self::Shader> {
        let mut inner = self.inner.lock().unwrap();
        let id = inner.next_shader_id;
        inner.next_shader_id += 1;

        inner.shaders.insert(id, WgpuShader {
            source: String::new(),
            shader_type: type_,
            compiled: false,
            info_log: String::new(),
        });

        Some(WgpuShaderHandle(id))
    }

    fn create_program(&self) -> Option<Self::Program> {
        let mut inner = self.inner.lock().unwrap();
        let id = inner.next_program_id;
        inner.next_program_id += 1;

        inner.programs.insert(id, WgpuProgramData {
            vertex_shader: None,
            fragment_shader: None,
            linked: false,
            pipeline: None,
            bind_group_layout: None,
            uniform_buffer: None,
            uniforms: HashMap::new(),
        });

        Some(WgpuProgram(id))
    }

    fn delete_program(&self, program: Option<&Self::Program>) {
        if let Some(prog) = program {
            self.inner.lock().unwrap().programs.remove(&prog.0);
        }
    }

    fn delete_shader(&self, shader: Option<&Self::Shader>) {
        if let Some(shdr) = shader {
            self.inner.lock().unwrap().shaders.remove(&shdr.0);
        }
    }

    fn is_shader(&self, shader: Option<&Self::Shader>) -> bool {
        if let Some(shdr) = shader {
            self.inner.lock().unwrap().shaders.contains_key(&shdr.0)
        } else {
            false
        }
    }

    fn is_program(&self, program: Option<&Self::Program>) -> bool {
        if let Some(prog) = program {
            self.inner.lock().unwrap().programs.contains_key(&prog.0)
        } else {
            false
        }
    }

    fn shader_source(&self, shader: &Self::Shader, source: &str) {
        if let Some(shdr) = self.inner.lock().unwrap().shaders.get_mut(&shader.0) {
            shdr.source = source.to_string();
        }
    }

    fn compile_shader(&self, shader: &Self::Shader) {
        // In wgpu, shaders are compiled when creating the pipeline
        // For now, just mark as compiled
        if let Some(shdr) = self.inner.lock().unwrap().shaders.get_mut(&shader.0) {
            shdr.compiled = true;
            shdr.info_log = String::from("Shader will be compiled with pipeline");
        }
    }

    fn link_program(&self, program: &Self::Program) {
        let (vs_id, fs_id) = {
            let inner = self.inner.lock().unwrap();
            let prog_data = match inner.programs.get(&program.0) {
                Some(p) => p,
                None => return,
            };
            (prog_data.vertex_shader, prog_data.fragment_shader)
        };

        let (vs_source, fs_source) = {
            let inner = self.inner.lock().unwrap();
            let vs = vs_id.and_then(|id| inner.shaders.get(&id).map(|s| s.source.clone()));
            let fs = fs_id.and_then(|id| inner.shaders.get(&id).map(|s| s.source.clone()));
            (vs, fs)
        };

        if let (Some(vs), Some(fs)) = (vs_source, fs_source) {
            let device = self.device();

            // Check if this is GLSL (old shader format) - skip pipeline creation for now
            if vs.trim().starts_with("#version") || fs.trim().starts_with("#version") {
                // GLSL shaders not yet converted to WGSL - skip pipeline creation
                // This allows the program to run but these materials won't render
                eprintln!("Warning: GLSL shader detected. This material won't render until converted to WGSL.");
                return;
            }

            // Check if this is a unified WGSL shader (same source for both)
            let is_unified = vs == fs;

            // Create shader module(s)
            let shader_module = if is_unified {
                // Unified WGSL shader with both @vertex and @fragment entry points
                device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Unified Shader"),
                    source: wgpu::ShaderSource::Wgsl(vs.into()),
                })
            } else {
                // Separate shaders (shouldn't happen with WGSL, but handle it)
                device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Vertex Shader"),
                    source: wgpu::ShaderSource::Wgsl(vs.into()),
                })
            };

            // For unified shaders, use the same module for both
            let vs_module = &shader_module;
            let fs_module = &shader_module;

            // Create bind group layout for uniforms and textures
            let bind_group_layout =
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        label: Some("Bind Group Layout"),
                        entries: &[
                            // Uniform buffer binding
                            wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Buffer {
                                    ty: wgpu::BufferBindingType::Uniform,
                                    has_dynamic_offset: false,
                                    min_binding_size: None,
                                },
                                count: None,
                            },
                            // Texture sampler
                            wgpu::BindGroupLayoutEntry {
                                binding: 1,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                                count: None,
                            },
                            // Texture
                            wgpu::BindGroupLayoutEntry {
                                binding: 2,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Texture {
                                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                    view_dimension: wgpu::TextureViewDimension::D2,
                                    multisampled: false,
                                },
                                count: None,
                            },
                        ],
                    });

            let pipeline_layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("Pipeline Layout"),
                        bind_group_layouts: &[&bind_group_layout],
                        push_constant_ranges: &[],
                    });

            // Create render pipeline
            let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Render Pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &vs_module,
                        entry_point: "vs_main",
                        buffers: &[
                            // Vertex buffer layout (per-vertex data)
                            wgpu::VertexBufferLayout {
                                array_stride: 32, // 3 floats (pos) + 2 floats (uv) + 3 floats (normal)
                                step_mode: wgpu::VertexStepMode::Vertex,
                                attributes: &[
                                    // Position
                                    wgpu::VertexAttribute {
                                        offset: 0,
                                        shader_location: 0,
                                        format: wgpu::VertexFormat::Float32x3,
                                    },
                                    // Tex coords
                                    wgpu::VertexAttribute {
                                        offset: 12,
                                        shader_location: 1,
                                        format: wgpu::VertexFormat::Float32x2,
                                    },
                                    // Normal
                                    wgpu::VertexAttribute {
                                        offset: 20,
                                        shader_location: 2,
                                        format: wgpu::VertexFormat::Float32x3,
                                    },
                                ],
                            },
                            // Instance buffer layout (per-instance data)
                            wgpu::VertexBufferLayout {
                                array_stride: 60, // 3 + 4 + 3*3 floats
                                step_mode: wgpu::VertexStepMode::Instance,
                                attributes: &[
                                    // Instance translation
                                    wgpu::VertexAttribute {
                                        offset: 0,
                                        shader_location: 3,
                                        format: wgpu::VertexFormat::Float32x3,
                                    },
                                    // Instance color
                                    wgpu::VertexAttribute {
                                        offset: 12,
                                        shader_location: 4,
                                        format: wgpu::VertexFormat::Float32x4,
                                    },
                                    // Instance deformation row 0
                                    wgpu::VertexAttribute {
                                        offset: 28,
                                        shader_location: 5,
                                        format: wgpu::VertexFormat::Float32x3,
                                    },
                                    // Instance deformation row 1
                                    wgpu::VertexAttribute {
                                        offset: 40,
                                        shader_location: 6,
                                        format: wgpu::VertexFormat::Float32x3,
                                    },
                                    // Instance deformation row 2
                                    wgpu::VertexAttribute {
                                        offset: 52,
                                        shader_location: 7,
                                        format: wgpu::VertexFormat::Float32x3,
                                    },
                                ],
                            },
                        ],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &fs_module,
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: wgpu::TextureFormat::Bgra8UnormSrgb,
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: wgpu::TextureFormat::Depth24Plus,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::LessEqual,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                });

            // Create a default uniform buffer (320 bytes for shader Uniforms struct)
            // Uniforms: proj(64) + view(64) + transform(64) + ntransform(48) + scale(48) + light_position(16) + color(16) = 320
            let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Default Uniform Buffer"),
                size: 320,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            // Create default texture (1x1 white pixel)
            let default_texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Default White Texture"),
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });

            // Write white color to default texture
            self.queue().write_texture(
                wgpu::ImageCopyTexture {
                    texture: &default_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &[255u8, 255, 255, 255],
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4),
                    rows_per_image: Some(1),
                },
                wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
            );

            let default_texture_view = default_texture.create_view(&wgpu::TextureViewDescriptor::default());

            // Create default sampler
            let default_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("Default Sampler"),
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                address_mode_w: wgpu::AddressMode::Repeat,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            });

            // Create bind group
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Default Bind Group"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uniform_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&default_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&default_texture_view),
                    },
                ],
            });

            // Store the pipeline and default resources
            let mut inner = self.inner.lock().unwrap();
            if let Some(prog_data) = inner.programs.get_mut(&program.0) {
                prog_data.pipeline = Some(Arc::new(pipeline));
                prog_data.bind_group_layout = Some(bind_group_layout);
                prog_data.linked = true;
            }

            // Store default resources for later use
            inner.default_bind_group = Some(Arc::new(bind_group));
            inner.default_texture = Some(default_texture);
            inner.default_sampler = Some(default_sampler);
            inner.default_uniform_buffer = Some(uniform_buffer);
        }
    }

    fn use_program(&self, program: Option<&Self::Program>) {
        let mut inner = self.inner.lock().unwrap();
        inner.bound_program = program.copied();
    }

    fn attach_shader(&self, program: &Self::Program, shader: &Self::Shader) {
        let mut inner = self.inner.lock().unwrap();

        // First get the shader type
        let shader_type = inner.shaders.get(&shader.0).map(|s| s.shader_type);

        // Then mutate the program
        if let (Some(prog), Some(stype)) = (inner.programs.get_mut(&program.0), shader_type) {
            if stype == Self::VERTEX_SHADER {
                prog.vertex_shader = Some(shader.0);
            } else if stype == Self::FRAGMENT_SHADER {
                prog.fragment_shader = Some(shader.0);
            }
        }
    }

    fn get_shader_parameter_int(&self, shader: &Self::Shader, pname: GLenum) -> Option<i32> {
        if pname == Self::COMPILE_STATUS {
            self.inner.lock().unwrap().shaders.get(&shader.0).map(|s| if s.compiled { 1 } else { 0 })
        } else {
            None
        }
    }

    fn get_shader_info_log(&self, shader: &Self::Shader) -> Option<String> {
        self.inner.lock().unwrap().shaders.get(&shader.0).map(|s| s.info_log.clone())
    }

    fn vertex_attrib_pointer(
        &self,
        _index: u32,
        _size: i32,
        _type_: GLenum,
        _normalized: bool,
        _stride: i32,
        _offset: GLintptr,
    ) {
        // Vertex attributes are specified in the pipeline layout in wgpu
        // Store this for later pipeline creation
    }

    fn enable_vertex_attrib_array(&self, _index: u32) {
        // No-op in wgpu
    }

    fn disable_vertex_attrib_array(&self, _index: u32) {
        // No-op in wgpu
    }

    fn get_attrib_location(&self, _program: &Self::Program, _name: &str) -> i32 {
        // Return a dummy location - wgpu uses explicit layout locations
        0
    }

    fn get_uniform_location(
        &self,
        program: &Self::Program,
        name: &str,
    ) -> Option<Self::UniformLocation> {
        Some(WgpuUniformLocation {
            program: program.0,
            name: name.to_string(),
        })
    }

    fn vertex_attrib_divisor(&self, _id: u32, _divisor: u32) {
        // Instancing in wgpu is configured in the vertex buffer layout
    }

    fn viewport(&self, x: i32, y: i32, width: i32, height: i32) {
        self.inner.lock().unwrap().viewport = (x, y, width, height);
    }

    fn scissor(&self, x: i32, y: i32, width: i32, height: i32) {
        self.inner.lock().unwrap().scissor = (x, y, width, height);
    }

    fn create_framebuffer(&self) -> Option<Self::Framebuffer> {
        Some(WgpuFramebuffer(0))
    }

    fn is_framebuffer(&self, framebuffer: Option<&Self::Framebuffer>) -> bool {
        framebuffer.is_some()
    }

    fn bind_framebuffer(&self, _target: GLenum, framebuffer: Option<&Self::Framebuffer>) {
        self.inner.lock().unwrap().bound_framebuffer = framebuffer.copied();
    }

    fn delete_framebuffer(&self, _framebuffer: Option<&Self::Framebuffer>) {
        // No-op
    }

    fn framebuffer_texture2d(
        &self,
        _target: GLenum,
        _attachment: GLenum,
        _textarget: GLenum,
        _texture: Option<&Self::Texture>,
        _level: i32,
    ) {
        // Framebuffers in wgpu are configured differently
    }

    fn create_renderbuffer(&self) -> Option<Self::Renderbuffer> {
        Some(WgpuRenderbuffer(0))
    }

    fn is_renderbuffer(&self, buffer: Option<&Self::Renderbuffer>) -> bool {
        buffer.is_some()
    }

    fn delete_renderbuffer(&self, _buffer: Option<&Self::Renderbuffer>) {
        // No-op
    }

    fn bind_renderbuffer(&self, _buffer: Option<&Self::Renderbuffer>) {
        // No-op
    }

    fn renderbuffer_storage(&self, _internal_format: GLenum, _width: i32, _height: i32) {
        // No-op
    }

    fn framebuffer_renderbuffer(
        &self,
        _attachment: GLenum,
        _renderbuffer: Option<&Self::Renderbuffer>,
    ) {
        // No-op
    }

    fn bind_texture(&self, target: GLenum, texture: Option<&Self::Texture>) {
        if target == Self::TEXTURE_2D {
            self.inner.lock().unwrap().bound_texture_2d = texture.copied();
        }
    }

    fn tex_image2d(
        &self,
        _target: GLenum,
        _level: i32,
        _internalformat: i32,
        width: i32,
        height: i32,
        _border: i32,
        _format: GLenum,
        pixels: Option<&[u8]>,
    ) {
        let inner = self.inner.lock().unwrap();
        if let Some(tex_handle) = inner.bound_texture_2d {
            let size = wgpu::Extent3d {
                width: width as u32,
                height: height as u32,
                depth_or_array_layers: 1,
            };

            let texture = inner.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Texture2D"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });

            if let Some(data) = pixels {
                inner.queue.write_texture(
                    wgpu::ImageCopyTexture {
                        texture: &texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    data,
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(4 * width as u32),
                        rows_per_image: Some(height as u32),
                    },
                    size,
                );
            }

            drop(inner);
            self.inner.lock().unwrap().textures.insert(tex_handle.0, Arc::new(texture));
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
        // Convert i32 to u8 and call tex_image2d
        if let Some(data) = pixels {
            let byte_data = unsafe {
                std::slice::from_raw_parts(
                    data.as_ptr() as *const u8,
                    data.len() * 4,
                )
            };
            self.tex_image2d(target, level, internalformat, width, height, border, format, Some(byte_data));
        } else {
            self.tex_image2d(target, level, internalformat, width, height, border, format, None);
        }
    }

    fn tex_sub_image2d(
        &self,
        _target: GLenum,
        _level: i32,
        xoffset: i32,
        yoffset: i32,
        width: i32,
        height: i32,
        _format: GLenum,
        pixels: Option<&[u8]>,
    ) {
        let inner = self.inner.lock().unwrap();
        if let Some(tex_handle) = inner.bound_texture_2d {
            if let Some(texture) = inner.textures.get(&tex_handle.0) {
                if let Some(data) = pixels {
                    inner.queue.write_texture(
                        wgpu::ImageCopyTexture {
                            texture,
                            mip_level: 0,
                            origin: wgpu::Origin3d {
                                x: xoffset as u32,
                                y: yoffset as u32,
                                z: 0,
                            },
                            aspect: wgpu::TextureAspect::All,
                        },
                        data,
                        wgpu::ImageDataLayout {
                            offset: 0,
                            bytes_per_row: Some(4 * width as u32),
                            rows_per_image: Some(height as u32),
                        },
                        wgpu::Extent3d {
                            width: width as u32,
                            height: height as u32,
                            depth_or_array_layers: 1,
                        },
                    );
                }
            }
        }
    }

    fn tex_parameteri(&self, _target: GLenum, _pname: GLenum, _param: i32) {
        // Texture parameters in wgpu are set via samplers
    }

    fn is_texture(&self, texture: Option<&Self::Texture>) -> bool {
        if let Some(tex) = texture {
            self.inner.lock().unwrap().textures.contains_key(&tex.0)
        } else {
            false
        }
    }

    fn create_texture(&self) -> Option<Self::Texture> {
        let mut inner = self.inner.lock().unwrap();
        let id = inner.next_texture_id;
        inner.next_texture_id += 1;
        Some(WgpuTexture(id))
    }

    fn delete_texture(&self, texture: Option<&Self::Texture>) {
        if let Some(tex) = texture {
            self.inner.lock().unwrap().textures.remove(&tex.0);
        }
    }

    fn active_texture(&self, texture: GLenum) {
        self.inner.lock().unwrap().active_texture_unit = texture - Self::TEXTURE0;
    }

    fn enable(&self, cap: GLenum) {
        let mut inner = self.inner.lock().unwrap();
        match cap {
            Self::DEPTH_TEST => inner.depth_test_enabled = true,
            Self::CULL_FACE => inner.cull_face_enabled = true,
            Self::BLEND => inner.blend_enabled = true,
            Self::SCISSOR_TEST => inner.scissor_test_enabled = true,
            _ => {}
        }
    }

    fn disable(&self, cap: GLenum) {
        let mut inner = self.inner.lock().unwrap();
        match cap {
            Self::DEPTH_TEST => inner.depth_test_enabled = false,
            Self::CULL_FACE => inner.cull_face_enabled = false,
            Self::BLEND => inner.blend_enabled = false,
            Self::SCISSOR_TEST => inner.scissor_test_enabled = false,
            _ => {}
        }
    }

    fn draw_elements(&self, _mode: GLenum, count: i32, _type_: GLenum, _offset: GLintptr) {
        // Collect resources including vertex buffers and bind group
        let (pipeline, index_buffer, vertex_buffers, bind_group) = {
            let inner = self.inner.lock().unwrap();

            let pipeline = inner.bound_program
                .and_then(|prog| inner.programs.get(&prog.0))
                .and_then(|p| p.pipeline.as_ref().map(|p| Arc::clone(p)));
            let index_buffer = inner.bound_element_buffer
                .and_then(|b| inner.buffers.get(&b.0).cloned());

            // Collect recent array buffers as vertex buffers
            let vertex_buffers: Vec<Arc<wgpu::Buffer>> = inner.recent_array_buffers
                .iter()
                .filter_map(|b| inner.buffers.get(&b.0).cloned())
                .collect();

            let bind_group = inner.default_bind_group.clone();

            (pipeline, index_buffer, vertex_buffers, bind_group)
        };

        // Set pending resources
        ACTIVE_RENDER_PASS.with(|arp| {
            if let Some(ref mut pass) = *arp.borrow_mut() {
                pass.pending_pipeline = pipeline;
                pass.pending_index_buffer = index_buffer;
                pass.pending_vertex_buffers = vertex_buffers;
                pass.pending_bind_group = bind_group;
            }
        });

        // Execute draw
        self.with_render_pass_exec(move |render_pass| {
            render_pass.draw_indexed(0..count as u32, 0, 0..1);
        });
    }

    fn draw_elements_instanced(
        &self,
        _mode: GLenum,
        count: i32,
        _type_: GLenum,
        _offset: GLintptr,
        instance_count: i32,
    ) {
        // Collect resources including vertex buffers and bind group
        let (pipeline, index_buffer, vertex_buffers, bind_group) = {
            let inner = self.inner.lock().unwrap();

            let pipeline = inner.bound_program
                .and_then(|prog| inner.programs.get(&prog.0))
                .and_then(|p| p.pipeline.as_ref().map(|p| Arc::clone(p)));
            let index_buffer = inner.bound_element_buffer
                .and_then(|b| inner.buffers.get(&b.0).cloned());

            // Collect recent array buffers as vertex buffers
            let vertex_buffers: Vec<Arc<wgpu::Buffer>> = inner.recent_array_buffers
                .iter()
                .filter_map(|b| inner.buffers.get(&b.0).cloned())
                .collect();

            let bind_group = inner.default_bind_group.clone();

            (pipeline, index_buffer, vertex_buffers, bind_group)
        };

        // Set pending resources
        ACTIVE_RENDER_PASS.with(|arp| {
            if let Some(ref mut pass) = *arp.borrow_mut() {
                pass.pending_pipeline = pipeline;
                pass.pending_index_buffer = index_buffer;
                pass.pending_vertex_buffers = vertex_buffers;
                pass.pending_bind_group = bind_group;
            }
        });

        // Execute draw
        self.with_render_pass_exec(move |render_pass| {
            render_pass.draw_indexed(0..count as u32, 0, 0..instance_count as u32);
        });
    }

    fn draw_arrays(&self, _mode: GLenum, first: i32, count: i32) {
        // Collect pipeline
        let pipeline = {
            let inner = self.inner.lock().unwrap();
            inner.bound_program
                .and_then(|prog| inner.programs.get(&prog.0))
                .and_then(|p| p.pipeline.as_ref().map(|p| Arc::clone(p)))
        };

        // Set pending pipeline
        ACTIVE_RENDER_PASS.with(|arp| {
            if let Some(ref mut pass) = *arp.borrow_mut() {
                pass.pending_pipeline = pipeline;
            }
        });

        // Execute draw
        self.with_render_pass_exec(move |render_pass| {
            render_pass.draw(first as u32..(first + count) as u32, 0..1);
        });
    }

    fn point_size(&self, _size: f32) {
        // Point size in wgpu is set via pipeline
    }

    fn line_width(&self, _width: f32) {
        // Line width is not supported in WebGPU core spec
    }

    fn clear(&self, _mask: u32) {
        // Clear is handled automatically by the render pass LoadOp::Clear
        // The clear color set via clear_color() is used when the render pass begins
        // This is intentionally a no-op as wgpu uses a different clearing model
    }

    fn clear_color(&self, r: f32, g: f32, b: f32, a: f32) {
        self.inner.lock().unwrap().clear_color = (r, g, b, a);
    }

    fn polygon_mode(&self, _face: GLenum, _mode: GLenum) -> bool {
        // Polygon mode (wireframe) requires native-only feature
        false
    }

    fn front_face(&self, _mode: GLenum) {
        // Front face winding is set in pipeline
    }

    fn depth_func(&self, _mode: GLenum) {
        // Depth function is set in pipeline
    }

    fn cull_face(&self, _mode: GLenum) {
        // Cull mode is set in pipeline
    }

    fn read_pixels(
        &self,
        _x: i32,
        _y: i32,
        _width: i32,
        _height: i32,
        _format: GLenum,
        _pixels: Option<&mut [u8]>,
    ) {
        // Reading pixels in wgpu requires creating a staging buffer and mapping it
    }

    fn pixel_storei(&self, _pname: GLenum, _param: i32) {
        // No direct equivalent in wgpu
    }

    fn blend_func_separate(
        &self,
        _src_rgb: GLenum,
        _dst_rgb: GLenum,
        _src_alpha: GLenum,
        _dst_alpha: GLenum,
    ) {
        // Blend state is configured in the pipeline
    }
}
