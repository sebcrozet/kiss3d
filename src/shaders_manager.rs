use std::ptr;
use std::str;
use glcore::consts::GL_VERSION_2_0::*;
use glcore::functions::GL_VERSION_2_0::*;
use glcore::types::GL_VERSION_1_0::*;
use shaders;

#[deriving(Eq)]
pub enum Shader {
    ObjectShader,
    LinesShader
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
    #[fixed_stack_segment] #[inline(never)]
    pub fn new() -> ShadersManager {
        let object_context = ShadersManager::load_object_shader();

        unsafe { glUseProgram(object_context.program) };

        ShadersManager {
            object_context: object_context,
            lines_context:  ShadersManager::load_lines_shader(),
            shader:         ObjectShader
        }
    }

    #[fixed_stack_segment] #[inline(never)]
    pub fn select(&mut self, shader: Shader) {
        if shader != self.shader {
            self.shader = shader;

            unsafe {
                match self.shader {
                    ObjectShader => glUseProgram(self.object_context.program),
                    LinesShader  => glUseProgram(self.lines_context.program)
                }
            }
        }
    }

    pub fn object_context<'r>(&'r self) -> &'r ObjectShaderContext {
        &'r self.object_context
    }

    pub fn lines_context<'r>(&'r self) -> &'r LinesShaderContext {
        &'r self.lines_context
    }

    #[fixed_stack_segment] #[inline(never)]
    fn load_object_shader() -> ObjectShaderContext {
        unsafe {
            // load the shader
            let (program, vshader, fshader) =
                ShadersManager::load_shader_program(shaders::OBJECT_VERTEX_SRC,
                                                    shaders::OBJECT_FRAGMENT_SRC);

            glUseProgram(program);

            // get the variables locations
            ObjectShaderContext {
                program:    program,
                vshader:    vshader,
                fshader:    fshader,
                pos:        glGetAttribLocation(program, "position".to_c_str().unwrap()) as GLuint,
                normal:     glGetAttribLocation(program, "normal".to_c_str().unwrap()) as GLuint,
                tex_coord:  glGetAttribLocation(program, "tex_coord_v".to_c_str().unwrap()) as GLuint,
                light:      glGetUniformLocation(program, "light_position".to_c_str().unwrap()),
                color:      glGetUniformLocation(program, "color".to_c_str().unwrap()),
                transform:  glGetUniformLocation(program, "transform".to_c_str().unwrap()),
                scale:      glGetUniformLocation(program, "scale".to_c_str().unwrap()),
                ntransform: glGetUniformLocation(program, "ntransform".to_c_str().unwrap()),
                proj:       glGetUniformLocation(program, "projection".to_c_str().unwrap()),
                view:       glGetUniformLocation(program, "view".to_c_str().unwrap()),
                tex:        glGetUniformLocation(program, "tex".to_c_str().unwrap())
            }
        }
    }

    #[fixed_stack_segment] #[inline(never)]
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
                pos:     glGetAttribLocation(program,  "position".to_c_str().unwrap()) as GLuint,
                color:   glGetAttribLocation(program, "color".to_c_str().unwrap()) as GLuint,
                proj:    glGetUniformLocation(program, "projection".to_c_str().unwrap()),
                view:    glGetUniformLocation(program, "view".to_c_str().unwrap()),
            };

            glEnableVertexAttribArray(res.pos);
            glEnableVertexAttribArray(res.color);

            res
        }
    }

    #[fixed_stack_segment] #[inline(never)]
    fn load_shader_program(vertex_shader:   &str,
                           fragment_shader: &str)
                           -> (GLuint, GLuint, GLuint) {
        // Create and compile the vertex shader
        let vshader = unsafe { glCreateShader(GL_VERTEX_SHADER) };
        unsafe {
            glShaderSource(vshader, 1, &vertex_shader.to_c_str().unwrap(), ptr::null());
            glCompileShader(vshader);
        }
        check_shader_error(vshader);

        // Create and compile the fragment shader
        let fshader = unsafe { glCreateShader(GL_FRAGMENT_SHADER) };
        unsafe {
            glShaderSource(fshader, 1, &fragment_shader.to_c_str().unwrap(), ptr::null());
            glCompileShader(fshader);
        }

        check_shader_error(fshader);

        // Link the vertex and fragment shader into a shader program
        let program = unsafe { glCreateProgram() };
        unsafe {
            glAttachShader(program, vshader);
            glAttachShader(program, fshader);
            glLinkProgram(program);
        }

        (program, vshader, fshader)
    }
}

#[fixed_stack_segment] #[inline(never)]
fn check_shader_error(shader: GLuint) {
    let compiles: i32 = 0;
    unsafe{
        glGetShaderiv(shader, GL_COMPILE_STATUS, &compiles);

        if(compiles == 0) {
            println("Shader compilation failed.");
            let info_log_len = 0;

            glGetShaderiv(shader, GL_INFO_LOG_LENGTH, &info_log_len);

            if (info_log_len > 0) {
                // error check for fail to allocate memory omitted
                let chars_written = 0;
                let info_log = " ".repeat(info_log_len as uint);

                let c_str = info_log.to_c_str();

                do c_str.with_ref |c_str| {
                    glGetShaderInfoLog(shader, info_log_len, &chars_written, c_str)
                }

                let bytes = c_str.as_bytes();
                let bytes = bytes.slice_to(bytes.len() - 1);
                fail!("Shader compilation failed: " + str::from_bytes(bytes));
            }
        }
    }
}

impl Drop for ShadersManager {
    #[fixed_stack_segment] #[inline(never)]
    fn drop(&self) {
        unsafe {
            glDeleteProgram(self.object_context.program);
            glDeleteShader(self.object_context.fshader);
            glDeleteShader(self.object_context.vshader);

            glDeleteProgram(self.lines_context.program);
            glDeleteShader(self.lines_context.fshader);
            glDeleteShader(self.lines_context.vshader);
        }
    }
}
