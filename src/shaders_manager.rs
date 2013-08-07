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
    pub fn new() -> ShadersManager {
        let object_context = ShadersManager::load_object_shader();

        unsafe { glUseProgram(object_context.program) };

        ShadersManager {
            object_context: object_context,
            lines_context:  ShadersManager::load_lines_shader(),
            shader:         ObjectShader
        }
    }

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
                pos:        glGetAttribLocation(program, "position".as_c_str(|s| s)) as GLuint,
                normal:     glGetAttribLocation(program, "normal".as_c_str(|s| s)) as GLuint,
                tex_coord:  glGetAttribLocation(program, "tex_coord_v".as_c_str(|s| s)) as GLuint,
                light:      glGetUniformLocation(program, "light_position".as_c_str(|s| s)),
                color:      glGetUniformLocation(program, "color".as_c_str(|s| s)),
                transform:  glGetUniformLocation(program, "transform".as_c_str(|s| s)),
                scale:      glGetUniformLocation(program, "scale".as_c_str(|s| s)),
                ntransform: glGetUniformLocation(program, "ntransform".as_c_str(|s| s)),
                proj:       glGetUniformLocation(program, "projection".as_c_str(|s| s)),
                view:       glGetUniformLocation(program, "view".as_c_str(|s| s)),
                tex:        glGetUniformLocation(program, "tex".as_c_str(|s| s))
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
                pos:     glGetAttribLocation(program,  "position".as_c_str(|s| s)) as GLuint,
                color:   glGetAttribLocation(program, "color".as_c_str(|s| s)) as GLuint,
                proj:    glGetUniformLocation(program, "projection".as_c_str(|s| s)),
                view:    glGetUniformLocation(program, "view".as_c_str(|s| s)),
            };

            glEnableVertexAttribArray(res.pos);
            glEnableVertexAttribArray(res.color);

            res
        }
    }

    fn load_shader_program(vertex_shader:   &str,
                           fragment_shader: &str)
                           -> (GLuint, GLuint, GLuint) {
        // Create and compile the vertex shader
        let vshader = unsafe { glCreateShader(GL_VERTEX_SHADER) };
        unsafe {
            glShaderSource(vshader, 1, &vertex_shader.as_c_str(|s| s), ptr::null());
            glCompileShader(vshader);
        }

        check_shader_error(vshader);

        // Create and compile the fragment shader
        let fshader = unsafe { glCreateShader(GL_FRAGMENT_SHADER) };
        unsafe {
            glShaderSource(fshader, 1, &fragment_shader.as_c_str(|s| s), ptr::null());
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

fn check_shader_error(shader: GLuint) {
    let compiles: i32 = 0;
    unsafe{
        glGetShaderiv(shader, GL_COMPILE_STATUS, &compiles);

        if(compiles == 0) {
            let info_log_len = 0;

            glGetShaderiv(shader, GL_INFO_LOG_LENGTH, &info_log_len);

            if (info_log_len > 0) {
                // error check for fail to allocate memory omitted
                let chars_written = 0;
                let mut info_log = ~"";

                str::raw::set_len(&mut info_log, (info_log_len + 1) as uint);

                do info_log.as_c_str |c_str| {
                    glGetShaderInfoLog(shader, info_log_len, &chars_written, c_str)
                }
                fail!("Shader compilation failed: " + info_log);
            }
        }
    }
}

impl Drop for ShadersManager {
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
