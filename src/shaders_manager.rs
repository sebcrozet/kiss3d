use std::ptr;
use std::str;
use gl;
use gl::types::*;
use shaders;

#[path = "error.rs"]
mod error;

pub enum Shader {
    ObjectShader,
    LinesShader,
    Other // FIXME: improve the manager to handler user-defined shaders properly
}

impl Eq for Shader {
    fn eq(&self, other: &Shader) -> bool {
        match (*self, *other) {
            (ObjectShader, ObjectShader) => true,
            (LinesShader, LinesShader)   => true,
            _ => false // FIXME: this is really suboptimal
        }
    }
}

pub struct ObjectShaderContext {
    program:    GLuint,
    vshader:    GLuint,
    fshader:    GLuint,
    pos:        GLuint,
    normal:     GLuint,
    tex_coord:  GLuint,
    light:      GLint,
    color:      GLint,
    transform:  GLint,
    scale:      GLint,
    ntransform: GLint,
    proj:       GLint,
    view:       GLint,
    tex:        GLint
}

pub struct LinesShaderContext {
    program:   GLuint,
    vshader:   GLuint,
    fshader:   GLuint,
    pos:       GLuint,
    color:     GLuint,
    proj:      GLint,
    view:      GLint
}

pub struct ShadersManager {
    priv object_context: ObjectShaderContext,
    priv lines_context:  LinesShaderContext,
    priv shader:         Shader
}

impl ShadersManager {
    pub fn new() -> ShadersManager {
        let object_context = ShadersManager::load_object_shader();

        verify!(gl::UseProgram(object_context.program));

        ShadersManager {
            object_context: object_context,
            lines_context:  ShadersManager::load_lines_shader(),
            shader:         ObjectShader
        }
    }

    pub fn select(&mut self, shader: Shader) {
        if true { // FIXME: shader != self.shader {
            self.shader = shader;

            match self.shader {
                ObjectShader => { verify!(gl::UseProgram(self.object_context.program)); },
                LinesShader  => { verify!(gl::UseProgram(self.lines_context.program)); }
                _ => { }
            }
        }
    }

    pub fn object_context<'r>(&'r self) -> &'r ObjectShaderContext {
        &'r self.object_context
    }

    pub fn lines_context<'r>(&'r self) -> &'r LinesShaderContext {
        &'r self.lines_context
    }

    fn load_object_shader() -> ObjectShaderContext {
        unsafe {
            // load the shader
            let (program, vshader, fshader) =
                ShadersManager::load_shader_program(shaders::OBJECT_VERTEX_SRC,
                                                    shaders::OBJECT_FRAGMENT_SRC);

            verify!(gl::UseProgram(program));

            // get the variables locations
            ObjectShaderContext {
                program:    program,
                vshader:    vshader,
                fshader:    fshader,
                pos:        gl::GetAttribLocation(program, "position".to_c_str().unwrap()) as GLuint,
                normal:     gl::GetAttribLocation(program, "normal".to_c_str().unwrap()) as GLuint,
                tex_coord:  gl::GetAttribLocation(program, "tex_coord_v".to_c_str().unwrap()) as GLuint,
                light:      gl::GetUniformLocation(program, "light_position".to_c_str().unwrap()),
                color:      gl::GetUniformLocation(program, "color".to_c_str().unwrap()),
                transform:  gl::GetUniformLocation(program, "transform".to_c_str().unwrap()),
                scale:      gl::GetUniformLocation(program, "scale".to_c_str().unwrap()),
                ntransform: gl::GetUniformLocation(program, "ntransform".to_c_str().unwrap()),
                proj:       gl::GetUniformLocation(program, "projection".to_c_str().unwrap()),
                view:       gl::GetUniformLocation(program, "view".to_c_str().unwrap()),
                tex:        gl::GetUniformLocation(program, "tex".to_c_str().unwrap())
            }
        }
    }

    fn load_lines_shader() -> LinesShaderContext {
        unsafe {
            // load the shader
            let (program, vshader, fshader) =
                ShadersManager::load_shader_program(
                    shaders::LINES_VERTEX_SRC,
                    shaders::LINES_FRAGMENT_SRC);

            let res = LinesShaderContext {
                program: program,
                vshader: vshader,
                fshader: fshader,
                pos:     gl::GetAttribLocation(program,  "position".to_c_str().unwrap()) as GLuint,
                color:   gl::GetAttribLocation(program,  "color".to_c_str().unwrap()) as GLuint,
                proj:    gl::GetUniformLocation(program, "projection".to_c_str().unwrap()),
                view:    gl::GetUniformLocation(program, "view".to_c_str().unwrap()),
            };

            verify!(gl::EnableVertexAttribArray(res.pos));
            verify!(gl::EnableVertexAttribArray(res.color));

            res
        }
    }

    pub fn load_shader_program(vertex_shader:   &str,
                               fragment_shader: &str)
                               -> (GLuint, GLuint, GLuint) {
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
}

fn check_shader_error(shader: GLuint) {
    let compiles: i32 = 0;
    unsafe{
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &compiles);

        if(compiles == 0) {
            println("Shader compilation failed.");
            let info_log_len = 0;

            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &info_log_len);

            if (info_log_len > 0) {
                // error check for fail to allocate memory omitted
                let chars_written = 0;
                let info_log = " ".repeat(info_log_len as uint);

                let c_str = info_log.to_c_str();

                do c_str.with_ref |c_str| {
                    gl::GetShaderInfoLog(shader, info_log_len, &chars_written, c_str);
                }

                let bytes = c_str.as_bytes();
                let bytes = bytes.slice_to(bytes.len() - 1);
                fail!("Shader compilation failed: " + str::from_bytes(bytes));
            }
        }
    }
}

impl Drop for ShadersManager {
    fn drop(&self) {
        gl::DeleteProgram(self.object_context.program);
        gl::DeleteShader(self.object_context.fshader);
        gl::DeleteShader(self.object_context.vshader);

        gl::DeleteProgram(self.lines_context.program);
        gl::DeleteShader(self.lines_context.fshader);
        gl::DeleteShader(self.lines_context.vshader);
    }
}
