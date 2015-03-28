use std::marker::PhantomData;
use std::ffi::CString;
use std::mem;
use std::ptr;
use std::str;
use std::iter::repeat;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use gl;
use gl::types::*;
use camera::Camera;
use resource::{GLPrimitive, GPUVector};

#[path = "../error.rs"]
mod error;

/// Structure encapsulating a shader program.
pub struct Shader {
    program: GLuint,
    vshader: GLuint,
    fshader: GLuint
}

impl Shader {
    /// Creates a new shader program from two files containing the vertex and fragment shader.
    pub fn new(vshader_path: &Path, fshader_path: &Path) -> Option<Shader> {
        let mut vshader = String::new();
        let mut fshader = String::new();

        if File::open(vshader_path).map(|mut v| v.read_to_string(&mut vshader)).is_err() {
            return None;
        }

        if File::open(fshader_path).map(|mut f| f.read_to_string(&mut fshader)).is_err() {
            return None;
        }

        Some(Shader::new_from_str(&vshader[..], &fshader[..]))
    }

    /// Creates a new shader program from strings of the vertex and fragment shader.
    pub fn new_from_str(vshader: &str, fshader: &str) -> Shader {
        let (program, vshader, fshader) = load_shader_program(vshader, fshader);

        Shader {
            program: program,
            vshader: vshader,
            fshader: fshader
        }
    }

    /// Gets a uniform variable from the shader program.
    pub fn get_uniform<T: GLPrimitive>(&self, name: &str) -> Option<ShaderUniform<T>> {
        let c_str = CString::new(name.as_bytes()).unwrap();
        let location = unsafe { gl::GetUniformLocation(self.program, c_str.as_ptr()) };

        if unsafe { gl::GetError() } == 0 && location != -1 {
            Some(ShaderUniform { id: location as GLuint, data_type: PhantomData })
        }
        else {
            None
        }
    }

    /// Gets an attribute from the shader program.
    pub fn get_attrib<T: GLPrimitive>(&self, name: &str) -> Option<ShaderAttribute<T>> {
        let c_str = CString::new(name.as_bytes()).unwrap();
        let location = unsafe { gl::GetAttribLocation(self.program, c_str.as_ptr()) };

        if unsafe { gl::GetError() } == 0 && location != -1 {
            Some(ShaderAttribute { id: location as GLuint, data_type: PhantomData })
        }
        else {
            None
        }
    }

    /// Make this program active.
    pub fn use_program(&mut self) {
        verify!(gl::UseProgram(self.program));
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        verify!(gl::DeleteProgram(self.program));
        verify!(gl::DeleteShader(self.fshader));
        verify!(gl::DeleteShader(self.vshader));
    }
}

/// Structure encapsulating an uniform variable.
pub struct ShaderUniform<T> {
    id:        GLuint,
    data_type: PhantomData<T>
}

impl<T: GLPrimitive> ShaderUniform<T> {
    /// Upload a value to this variable.
    pub fn upload(&mut self, value: &T) {
        value.upload(self.id)
    }
}

/// Structure encapsulating an attribute.
pub struct ShaderAttribute<T> {
    id:        GLuint,
    data_type: PhantomData<T>
}

impl<T: GLPrimitive> ShaderAttribute<T> {
    /// Disable this attribute.
    pub fn disable(&mut self) {
        verify!(gl::DisableVertexAttribArray(self.id));
    }

    /// Enable this attribute.
    pub fn enable(&mut self) {
        verify!(gl::EnableVertexAttribArray(self.id));
    }

    /// Binds this attribute to a gpu vector.
    pub fn bind(&mut self, vector: &mut GPUVector<T>) {
        vector.bind();

        unsafe {
            verify!(gl::VertexAttribPointer(
                        self.id,
                        GLPrimitive::size(None::<T>) as i32,
                        GLPrimitive::gl_type(None::<T>),
                        gl::FALSE,
                        0,
                        ptr::null()));
        }
    }

    /// Binds this attribute to non contiguous parts of a gpu vector.
    pub fn bind_sub_buffer(&mut self, vector: &mut GPUVector<T>, strides: usize, start_index: usize) {
        vector.bind();

        unsafe {
            verify!(gl::VertexAttribPointer(
                        self.id,
                        GLPrimitive::size(None::<T>) as i32,
                        GLPrimitive::gl_type(None::<T>),
                        gl::FALSE,
                        ((strides + 1) * mem::size_of::<T>()) as GLint,
                        mem::transmute(start_index * mem::size_of::<T>())));
        }
    }
}

/// Loads a shader program using the given source codes for the vertex and fragment shader.
///
/// Fails after displaying opengl compilation errors if the shaders are invalid.
fn load_shader_program(vertex_shader: &str, fragment_shader: &str) -> (GLuint, GLuint, GLuint) {
    // Create and compile the vertex shader
    let vshader = verify!(gl::CreateShader(gl::VERTEX_SHADER));
    let vertex_shader = CString::new(vertex_shader.as_bytes()).unwrap();
    let fragment_shader = CString::new(fragment_shader.as_bytes()).unwrap();

    unsafe {
        verify!(gl::ShaderSource(vshader, 1, &vertex_shader.as_ptr(), ptr::null()));
        verify!(gl::CompileShader(vshader));
    }
    check_shader_error(vshader);

    // Create and compile the fragment shader
    let fshader = verify!(gl::CreateShader(gl::FRAGMENT_SHADER));
    unsafe {
        verify!(gl::ShaderSource(fshader, 1, &fragment_shader.as_ptr(), ptr::null()));
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
fn check_shader_error(shader: GLuint) {
    let mut compiles: i32 = 0;

    unsafe{
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
                gl::GetShaderInfoLog(shader, info_log_len, &mut chars_written, c_str.as_ptr() as *mut i8);

                let bytes = c_str.as_bytes();
                let bytes = &bytes[.. bytes.len() - 1];
                panic!("Shader compilation failed: {}", str::from_utf8(bytes).unwrap());
            }
        }
    }
}
