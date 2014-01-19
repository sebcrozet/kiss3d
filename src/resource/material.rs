//! Materials definition and shader-related tools.

use std::ptr;
use std::str;
use gl;
use gl::types::*;
use camera::Camera;
use light::Light;
use object::ObjectData;
use resource::mesh::Mesh;

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

/// Loads a shader program using the given source codes for the vertex and fragment shader.
///
/// Fails after displaying opengl compilation errors if the shaders are invalid.
pub fn load_shader_program(vertex_shader: &str, fragment_shader: &str) -> (GLuint, GLuint, GLuint) {
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
pub fn check_shader_error(shader: GLuint) {
    let mut compiles: i32 = 0;

    unsafe{
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut compiles);

        if(compiles == 0) {
            println!("Shader compilation failed.");
            let mut info_log_len = 0;

            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut info_log_len);

            if (info_log_len > 0) {
                // error check for fail to allocate memory omitted
                let mut chars_written = 0;
                let info_log = " ".repeat(info_log_len as uint);

                let mut c_str = info_log.to_c_str();

                c_str.with_mut_ref(|c_str| {
                    gl::GetShaderInfoLog(shader, info_log_len, &mut chars_written, c_str);
                });

                let bytes = c_str.as_bytes();
                let bytes = bytes.slice_to(bytes.len() - 1);
                fail!("Shader compilation failed: " + str::from_utf8(bytes));
            }
        }
    }
}
