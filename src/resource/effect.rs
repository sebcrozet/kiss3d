use std::ffi::CString;
use std::fs::File;
use std::io::Read;
use std::iter::repeat;
use std::marker::PhantomData;
use std::mem;
use std::path::Path;
use std::ptr;
use std::str;

use context::{Context, Program, Shader, UniformLocation};
use resource::{GLPrimitive, GPUVec};

#[path = "../error.rs"]
mod error;

/// Structure encapsulating a program.
pub struct Effect {
    program: Program,
    vshader: Shader,
    fshader: Shader,
}

impl Effect {
    /// Creates a new shader program from two files containing the vertex and fragment shader.
    pub fn new(vshader_path: &Path, fshader_path: &Path) -> Option<Effect> {
        let mut vshader = String::new();
        let mut fshader = String::new();

        if File::open(vshader_path)
            .map(|mut v| v.read_to_string(&mut vshader))
            .is_err()
        {
            return None;
        }

        if File::open(fshader_path)
            .map(|mut f| f.read_to_string(&mut fshader))
            .is_err()
        {
            return None;
        }

        Some(Effect::new_from_str(&vshader[..], &fshader[..]))
    }

    /// Creates a new shader program from strings of the vertex and fragment shader.
    pub fn new_from_str(vshader: &str, fshader: &str) -> Effect {
        let (program, vshader, fshader) = load_shader_program(vshader, fshader);

        Effect {
            program,
            vshader,
            fshader,
        }
    }

    /// Gets a uniform variable from the shader program.
    pub fn get_uniform<T: GLPrimitive>(&self, name: &str) -> Option<ShaderUniform<T>> {
        let ctxt = Context::get();
        let location = ctxt.get_uniform_location(&self.program, name);

        if ctxt.get_error() == 0 {
            if let Some(id) = location {
                let data_type = PhantomData;
                return Some(ShaderUniform { id, data_type });
            }
        }

        return None;
    }

    /// Gets an attribute from the shader program.
    pub fn get_attrib<T: GLPrimitive>(&self, name: &str) -> Option<ShaderAttribute<T>> {
        let ctxt = Context::get();
        let location = ctxt.get_attrib_location(&self.program, name);

        if ctxt.get_error() == 0 && location != -1 {
            let id = location as u32;
            let data_type = PhantomData;
            return Some(ShaderAttribute { id, data_type });
        }

        return None;
    }

    /// Make this program active.
    pub fn use_program(&mut self) {
        verify!(Context::get().use_program(Some(&self.program)));
    }
}

impl Drop for Effect {
    fn drop(&mut self) {
        let ctxt = Context::get();
        if ctxt.is_program(Some(&self.program)) {
            verify!(ctxt.delete_program(Some(&self.program)));
        }
        if ctxt.is_shader(Some(&self.fshader)) {
            verify!(ctxt.delete_shader(Some(&self.fshader)));
        }
        if ctxt.is_shader(Some(&self.vshader)) {
            verify!(ctxt.delete_shader(Some(&self.vshader)));
        }
    }
}

/// Structure encapsulating an uniform variable.
pub struct ShaderUniform<T> {
    id: UniformLocation,
    data_type: PhantomData<T>,
}

impl<T: GLPrimitive> ShaderUniform<T> {
    /// Upload a value to this variable.
    pub fn upload(&mut self, value: &T) {
        value.upload(&self.id)
    }
}

/// Structure encapsulating an attribute.
pub struct ShaderAttribute<T> {
    id: u32,
    data_type: PhantomData<T>,
}

impl<T: GLPrimitive> ShaderAttribute<T> {
    /// Disable this attribute.
    pub fn disable(&mut self) {
        verify!(Context::get().disable_vertex_attrib_array(self.id));
    }

    /// Enable this attribute.
    pub fn enable(&mut self) {
        verify!(gl::EnableVertexAttribArray(self.id));
    }

    /// Binds this attribute to a gpu vector.
    pub fn bind(&mut self, vector: &mut GPUVec<T>) {
        vector.bind();

        unsafe {
            verify!(gl::VertexAttribPointer(
                self.id,
                GLPrimitive::size() as i32,
                GLPrimitive::gl_type(),
                gl::FALSE,
                0,
                ptr::null()
            ));
        }
    }

    /// Binds this attribute to non contiguous parts of a gpu vector.
    pub fn bind_sub_buffer(&mut self, vector: &mut GPUVec<T>, strides: usize, start_index: usize) {
        vector.bind();

        unsafe {
            verify!(gl::VertexAttribPointer(
                self.id,
                GLPrimitive::size() as i32,
                GLPrimitive::gl_type(),
                gl::FALSE,
                ((strides + 1) * mem::size_of::<T>()) as i32,
                mem::transmute(start_index * mem::size_of::<T>())
            ));
        }
    }
}

/// Loads a shader program using the given source codes for the vertex and fragment shader.
///
/// Fails after displaying opengl compilation errors if the shaders are invalid.
fn load_shader_program(vertex_shader: &str, fragment_shader: &str) -> (Program, Shader, Shader) {
    // Create and compile the vertex shader
    let ctxt = Context::get();
    let vshader = verify!(ctxt.create_shader(Context::VERTEX_SHADER));
    let vertex_shader = CString::new(vertex_shader.as_bytes()).unwrap();
    let fragment_shader = CString::new(fragment_shader.as_bytes()).unwrap();

    unsafe {
        verify!(gl::ShaderSource(
            vshader,
            1,
            &vertex_shader.as_ptr(),
            ptr::null()
        ));
        verify!(gl::CompileShader(vshader));
    }
    check_shader_error(vshader);

    // Create and compile the fragment shader
    let fshader = verify!(gl::CreateShader(gl::FRAGMENT_SHADER));
    unsafe {
        verify!(gl::ShaderSource(
            fshader,
            1,
            &fragment_shader.as_ptr(),
            ptr::null()
        ));
        verify!(gl::CompileShader(fshader));
    }

    check_shader_error(fshader);

    // Link the vertex and fragment shader into a shader program
    let program = verify!(gl::CreateProgram());
    verify!(gl::AttachShader(program, vshader));
    verify!(gl::AttachShader(program, fshader));
    verify!(gl::LinkProgram(program));

    (program, vshader, fshader)
}

/// Checks if a shader handle is valid.
///
/// If it is not valid, it fails with a descriptive error message.
fn check_shader_error(shader: u32) {
    let mut compiles: i32 = 0;

    unsafe {
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut compiles);

        if compiles == 0 {
            println!("Shader compilation failed.");
            let mut info_log_len = 0;

            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut info_log_len);

            if info_log_len > 0 {
                // error check for fail to allocate memory omitted
                let mut chars_written = 0;
                let info_log: String = repeat(' ').take(info_log_len as usize).collect();

                let c_str = CString::new(info_log.as_bytes()).unwrap();
                gl::GetShaderInfoLog(
                    shader,
                    info_log_len,
                    &mut chars_written,
                    c_str.as_ptr() as *mut _,
                );

                let bytes = c_str.as_bytes();
                let bytes = &bytes[..bytes.len() - 1];
                panic!(
                    "Shader compilation failed: {}",
                    str::from_utf8(bytes).unwrap()
                );
            }
        }
    }
}
