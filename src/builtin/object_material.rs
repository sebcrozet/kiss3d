use std::cast;
use std::ptr;
use gl;
use gl::types::*;
use nalgebra::na::{Mat3, Mat4};
use nalgebra::na;
use resource::Material;
use resource;
use object::ObjectData;
use light::{Light, Absolute, StickToCamera};
use camera::Camera;
use resource::Mesh;

#[path = "../error.rs"]
mod error;

/// The default material used to draw objects.
pub struct ObjectMaterial {
    priv program:    GLuint,
    priv vshader:    GLuint,
    priv fshader:    GLuint,
    priv pos:        GLuint,
    priv normal:     GLuint,
    priv tex_coord:  GLuint,
    priv light:      GLint,
    priv color:      GLint,
    priv transform:  GLint,
    priv scale:      GLint,
    priv ntransform: GLint,
    priv view:       GLint,
    priv tex:        GLint
}

impl ObjectMaterial {
    /// Creates a new `ObjectMaterial`.
    pub fn new() -> ObjectMaterial {
        unsafe {
            // load the shader
            let (program, vshader, fshader) =
                resource::load_shader_program(OBJECT_VERTEX_SRC, OBJECT_FRAGMENT_SRC);

            verify!(gl::UseProgram(program));

            // get the variables locations
            ObjectMaterial {
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
                view:       gl::GetUniformLocation(program, "view".to_c_str().unwrap()),
                tex:        gl::GetUniformLocation(program, "tex".to_c_str().unwrap())
            }
        }
    }

    fn activate(&mut self) {
        verify!(gl::UseProgram(self.program));
        verify!(gl::EnableVertexAttribArray(self.pos));
        verify!(gl::EnableVertexAttribArray(self.normal));
        verify!(gl::EnableVertexAttribArray(self.tex_coord));
    }

    fn deactivate(&mut self) {
        verify!(gl::DisableVertexAttribArray(self.pos));
        verify!(gl::DisableVertexAttribArray(self.normal));
        verify!(gl::DisableVertexAttribArray(self.tex_coord));
    }
}

impl Material for ObjectMaterial {
    fn render(&mut self,
              pass:   uint,
              camera: &mut Camera,
              light:  &Light,
              data:   &ObjectData,
              mesh:   &mut Mesh) {
        self.activate();


        /*
         *
         * Setup camera and light.
         *
         */
        camera.upload(pass, self.view);

        let pos = match *light {
            Absolute(ref p) => p.clone(),
            StickToCamera   => camera.eye()
        };
        verify!(gl::Uniform3f(self.light, pos.x, pos.y, pos.z));

        /*
         *
         * Setup object-related stuffs.
         *
         */
        let formated_transform:  Mat4<f32> = na::to_homogeneous(data.transform());
        let formated_ntransform: Mat3<f32> = *data.transform().rotation.submat();

        unsafe {
            verify!(gl::UniformMatrix4fv(self.transform,
                                         1,
                                         gl::FALSE as u8,
                                         cast::transmute(&formated_transform)));

            verify!(gl::UniformMatrix3fv(self.ntransform,
                                         1,
                                         gl::FALSE as u8,
                                         cast::transmute(&formated_ntransform)));

            verify!(gl::UniformMatrix3fv(self.scale, 1, gl::FALSE as u8, cast::transmute(data.scale())));

            verify!(gl::Uniform3f(self.color, data.color().x, data.color().y, data.color().z));

            mesh.bind(self.pos, self.normal, self.tex_coord);

            verify!(gl::ActiveTexture(gl::TEXTURE0));
            verify!(gl::BindTexture(gl::TEXTURE_2D, data.texture().borrow().id()));

            verify!(gl::DrawElements(gl::TRIANGLES,
                    mesh.num_pts() as GLint,
                    gl::UNSIGNED_INT,
                    ptr::null()));
        }

        mesh.unbind();
        self.deactivate();
    }
}

impl Drop for ObjectMaterial {
    fn drop(&mut self) {
        gl::DeleteProgram(self.program);
        gl::DeleteShader(self.fshader);
        gl::DeleteShader(self.vshader);
    }
}

/// Vertex shader of the default object material.
pub static OBJECT_VERTEX_SRC:   &'static str = A_VERY_LONG_STRING;
/// Fragment shader of the default object material.
pub static OBJECT_FRAGMENT_SRC: &'static str = ANOTHER_VERY_LONG_STRING;

static A_VERY_LONG_STRING: &'static str =
   "#version 120
    attribute vec3 position;
    attribute vec3 normal;
    attribute vec3 color;
    attribute vec2 tex_coord_v;
    varying vec3 ws_normal;
    varying vec3 ws_position;
    varying vec2 tex_coord;
    uniform mat4 view;
    uniform mat4 transform;
    uniform mat3 scale;
    uniform mat3 ntransform;
    void main() {
        mat4 scale4 = mat4(scale);
        vec4 pos4   = transform * scale4 * vec4(position, 1.0);
        tex_coord   = tex_coord_v;
        ws_position = pos4.xyz;
        gl_Position = view * pos4;
        ws_normal   = normalize(ntransform * scale * normal);
    }";

// phong lighting (heavily) inspired
// by http://www.opengl.org/sdk/docs/tutorials/ClockworkCoders/lighting.php
static ANOTHER_VERY_LONG_STRING: &'static str =
   "#version 120
    uniform vec3      color;
    uniform vec3      light_position;
    uniform sampler2D tex;
    varying vec2      tex_coord;
    varying vec3      ws_normal;
    varying vec3      ws_position;
    void main() {
      vec3 L = normalize(light_position - ws_position);
      vec3 E = normalize(-ws_position);

      //calculate Ambient Term:
      vec4 Iamb = vec4(1.0, 1.0, 1.0, 1.0);

      //calculate Diffuse Term:
      vec4 Idiff1 = vec4(1.0, 1.0, 1.0, 1.0) * max(dot(ws_normal,L), 0.0);
      Idiff1 = clamp(Idiff1, 0.0, 1.0);

      // double sided lighting:
      vec4 Idiff2 = vec4(1.0, 1.0, 1.0, 1.0) * max(dot(-ws_normal,L), 0.0);
      Idiff2 = clamp(Idiff2, 0.0, 1.0);

      vec4 tex_color = texture2D(tex, tex_coord);
      gl_FragColor   = tex_color * (vec4(color, 1.0) + Iamb + (Idiff1 + Idiff2) / 2) / 3;
    }";
