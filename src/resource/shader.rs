use std::mem;
use std::ptr;
use std::str;
use std::kinds::marker::NoCopy;
use std::io::fs::File;
use std::io::fs::PathExtensions;
use std::io::Reader;
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
    fshader: GLuint,
    nocpy:   NoCopy
}

impl Shader {
    /// Creates a new shader program from two files containing the vertex and fragment shader.
    pub fn new(vshader_path: &Path, fshader_path: &Path) -> Option<Shader> {
        if !vshader_path.exists() || !fshader_path.exists() {
            None
        }
        else {
            let vshader = File::open(vshader_path).map(|mut v| v.read_to_string());
            let fshader = File::open(fshader_path).map(|mut f| f.read_to_string());

            if vshader.is_err() || fshader.is_err() {
                return None;
            }

            let vshader = vshader.unwrap();
            let fshader = fshader.unwrap();

            if vshader.is_err() || fshader.is_err() {
                return None;
            }

            Some(Shader::new_from_str(vshader.unwrap().as_slice(), fshader.unwrap().as_slice()))
        }
    }

    /// Creates a new shader program from strings of the vertex and fragment shader.
    pub fn new_from_str(vshader: &str, fshader: &str) -> Shader {
        let (program, vshader, fshader) = load_shader_program(vshader, fshader);

        Shader {
            program: program,
            vshader: vshader,
            fshader: fshader,
            nocpy:   NoCopy
        }
    }

    /// Gets a uniform variable from the shader program.
    pub fn get_uniform<T: GLPrimitive>(&self, name: &str) -> Option<ShaderUniform<T>> {
        let location = unsafe { gl::GetUniformLocation(self.program, name.to_c_str().unwrap()) };

        if unsafe { gl::GetError() } == 0 && location != -1 {
            Some(ShaderUniform { id: location as GLuint })
        }
        else {
            None
        }
    }

    /// Gets an attribute from the shader program.
    pub fn get_attrib<T: GLPrimitive>(&self, name: &str) -> Option<ShaderAttribute<T>> {
        let location = unsafe { gl::GetAttribLocation(self.program, name.to_c_str().unwrap()) };

        if unsafe { gl::GetError() } == 0 && location != -1 {
            Some(ShaderAttribute { id: location as GLuint })
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
    id: GLuint
}

impl<T: GLPrimitive> ShaderUniform<T> {
    /// Upload a value to this variable.
    pub fn upload(&mut self, value: &T) {
        value.upload(self.id)
    }
}

/// Structure encapsulating an attribute.
pub struct ShaderAttribute<T> {
    id: GLuint
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
    pub fn bind_sub_buffer(&mut self, vector: &mut GPUVector<T>, strides: uint, start_index: uint) {
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
    unsafe {
        verify!(gl::ShaderSource(vshader, 1, &vertex_shader.to_c_str().unwrap(), ptr::null()));
        verify!(gl::CompileShader(vshader));
    }
    check_shader_error(vshader);

    // Create and compile the fragment shader
    let fshader = verify!(gl::CreateShader(gl::FRAGMENT_SHADER));
    unsafe {
        verify!(gl::ShaderSource(fshader, 1, &fragment_shader.to_c_str().unwrap(), ptr::null()));
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
                let info_log = " ".repeat(info_log_len as uint);

                let mut c_str = info_log.to_c_str();

                gl::GetShaderInfoLog(shader, info_log_len, &mut chars_written, c_str.as_mut_ptr());

                let bytes = c_str.as_bytes();
                let bytes = bytes.slice_to(bytes.len() - 1);
                panic!("Shader compilation failed: {}", str::from_utf8(bytes).unwrap());
            }
        }
    }
}
