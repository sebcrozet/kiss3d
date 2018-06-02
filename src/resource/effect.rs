use std::fs::File;
use std::io::Read;
use std::marker::PhantomData;
use std::mem;
use std::path::Path;
use std::str;

use context::{Context, GLintptr, Program, Shader, UniformLocation};
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
        verify!(Context::get().enable_vertex_attrib_array(self.id));
    }

    /// Binds this attribute to a gpu vector.
    pub fn bind(&mut self, vector: &mut GPUVec<T>) {
        vector.bind();

        verify!(Context::get().vertex_attrib_pointer(
            self.id,
            T::size() as i32,
            T::gl_type(),
            false,
            0,
            0
        ));
    }

    /// Binds this attribute to non contiguous parts of a gpu vector.
    pub fn bind_sub_buffer(&mut self, vector: &mut GPUVec<T>, strides: usize, start_index: usize) {
        vector.bind();

        verify!(Context::get().vertex_attrib_pointer(
            self.id,
            T::size() as i32,
            T::gl_type(),
            false,
            ((strides + 1) * mem::size_of::<T>()) as i32,
            (start_index * mem::size_of::<T>()) as GLintptr
        ));
    }
}

/// Loads a shader program using the given source codes for the vertex and fragment shader.
///
/// Fails after displaying opengl compilation errors if the shaders are invalid.
fn load_shader_program(vertex_shader: &str, fragment_shader: &str) -> (Program, Shader, Shader) {
    // Create and compile the vertex shader
    let ctxt = Context::get();
    let vshader = verify!(
        ctxt.create_shader(Context::VERTEX_SHADER)
            .expect("Could not create vertex shader.")
    );

    verify!(ctxt.shader_source(&vshader, vertex_shader));
    verify!(ctxt.compile_shader(&vshader));
    check_shader_error(&vshader);

    // Create and compile the fragment shader
    let fshader = verify!(
        ctxt.create_shader(Context::FRAGMENT_SHADER)
            .expect("Could not create fragment shader.")
    );
    verify!(ctxt.shader_source(&fshader, fragment_shader));
    verify!(ctxt.compile_shader(&fshader));
    check_shader_error(&fshader);

    // Link the vertex and fragment shader into a shader program
    let program = verify!(ctxt.create_program().expect("Could not create program."));
    verify!(ctxt.attach_shader(&program, &vshader));
    verify!(ctxt.attach_shader(&program, &fshader));
    verify!(ctxt.link_program(&program));
    (program, vshader, fshader)
}

/// Checks if a shader handle is valid.
///
/// If it is not valid, it fails with a descriptive error message.
fn check_shader_error(shader: &Shader) {
    let ctxt = Context::get();
    let compiles = ctxt.get_shader_parameter_int(shader, Context::COMPILE_STATUS);

    if compiles == Some(0) {
        if let Some(log) = ctxt.get_shader_info_log(shader) {
            panic!("Shader compilation failed: {}", log);
        } else {
            println!("Shader compilation failed.");
        }
    }
}
