//! Materials definition and shader-related tools.

use std::cast;
use std::mem;
use std::ptr;
use std::str;
use std::util::NonCopyable;
use std::io::fs::File;
use std::io::Reader;
use gl;
use gl::types::*;
use camera::Camera;
use light::Light;
use object::ObjectData;
use resource::{GLPrimitive, Mesh, GPUVector};

#[path = "../error.rs"]
mod error;

/// Trait implemented by materials.
pub trait Material {
    // FIXME: add the number of the current pass?
    /// Makes the material active.
    fn render(&mut self,
              pass:   uint,
              camera: &mut Camera,    // FIXME: replace those two arguments by
              light:  &Light,        // a structure with all environment datas
              data:   &ObjectData,
              mesh:   &mut Mesh);
}


/// Structure encapsulating a shader program.
pub struct Shader {
    priv program: GLuint,
    priv vshader: GLuint,
    priv fshader: GLuint,
    priv nocpy:   NonCopyable
}

impl Shader {
    /// Creates a new shader program from two files containing the vertex and fragment shader.
    pub fn new(vshader_path: &Path, fshader_path: &Path) -> Option<Shader> {
        if !vshader_path.exists() || !fshader_path.exists() {
            None
        }
        else {
            let vshader = File::open(vshader_path).map(|mut v| v.read_to_end());
            let fshader = File::open(fshader_path).map(|mut f| f.read_to_end());

            if vshader.is_none() || fshader.is_none() {
                return None;
            }

            let vshader = str::from_utf8_owned(vshader.unwrap());
            let fshader = str::from_utf8_owned(fshader.unwrap());

            if vshader.is_none() || fshader.is_none() {
                return None;
            }

            Some(Shader::new_from_str(vshader.unwrap(), fshader.unwrap()))
        }
    }

    /// Creates a new shader program from strings of the vertex and fragment shader.
    pub fn new_from_str(vshader: &str, fshader: &str) -> Shader {
        let (program, vshader, fshader) = load_shader_program(vshader, fshader);

        Shader {
            program: program,
            vshader: vshader,
            fshader: fshader,
            nocpy:   NonCopyable
        }
    }

    /// Gets a uniform variable from the shader program.
    pub fn get_uniform<T: GLPrimitive>(&self, name: &str) -> Option<ShaderUniform<T>> {
        let location = unsafe { gl::GetUniformLocation(self.program, name.to_c_str().unwrap()) as GLuint };

        if gl::GetError() == 0 {
            Some(ShaderUniform { id: location })
        }
        else {
            None
        }
    }

    /// Gets an attribute from the shader program.
    pub fn get_attrib<T: GLPrimitive>(&self, name: &str) -> Option<ShaderAttribute<T>> {
        let location = unsafe { gl::GetAttribLocation(self.program, name.to_c_str().unwrap()) as GLuint };

        if gl::GetError() == 0 {
            Some(ShaderAttribute { id: location })
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
        gl::DeleteProgram(self.program);
        gl::DeleteShader(self.fshader);
        gl::DeleteShader(self.vshader);
    }
}

/// Structure encapsulating an uniform variable.
pub struct ShaderUniform<T> {
    priv id: GLuint
}

impl<T: GLPrimitive> ShaderUniform<T> {
    /// Upload a value to this variable.
    pub fn upload(&mut self, value: &T) {
        value.upload(self.id)
    }
}

/// Structure encapsulating an attribute.
pub struct ShaderAttribute<T> {
    priv id: GLuint
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
                        gl::FALSE as u8,
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
                        gl::FALSE as u8,
                        ((strides + 1) * mem::size_of::<T>()) as GLint,
                        cast::transmute(start_index * mem::size_of::<T>())));
        }
    }
}

/// Loads a shader program using the given source codes for the vertex and fragment shader.
///
/// Fails after displaying opengl compilation errors if the shaders are invalid.
fn load_shader_program(vertex_shader: &str, fragment_shader: &str) -> (GLuint, GLuint, GLuint) {
    // Create and compile the vertex shader
    let vshader = gl::CreateShader(gl::VERTEX_SHADER);
    unsafe {
        verify!(gl::ShaderSource(vshader, 1, &vertex_shader.to_c_str().unwrap(), ptr::null()));
        verify!(gl::CompileShader(vshader));
    }
    check_shader_error(vshader);

    // Create and compile the fragment shader
    let fshader = gl::CreateShader(gl::FRAGMENT_SHADER);
    unsafe {
        verify!(gl::ShaderSource(fshader, 1, &fragment_shader.to_c_str().unwrap(), ptr::null()));
        verify!(gl::CompileShader(fshader));
    }

    check_shader_error(fshader);

    // Link the vertex and fragment shader into a shader program
    let program = gl::CreateProgram();
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

                c_str.with_mut_ref(|c_str| {
                    gl::GetShaderInfoLog(shader, info_log_len, &mut chars_written, c_str);
                });

                let bytes = c_str.as_bytes();
                let bytes = bytes.slice_to(bytes.len() - 1);
                fail!("Shader compilation failed: " + str::from_utf8(bytes).unwrap());
            }
        }
    }
}
