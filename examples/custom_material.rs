
extern mod gl;
extern mod kiss3d;
extern mod nalgebra;

use std::ptr;
use std::cast;
use std::rc::Rc;
use std::cell::RefCell;
use gl::types::{GLuint, GLint};
use nalgebra::na::{Vec3, Mat4, Rotation};
use nalgebra::na;
use kiss3d::window;
use kiss3d::object::ObjectData;
use kiss3d::camera::Camera;
use kiss3d::light::Light;
use kiss3d::resource::{Material, Mesh};
use kiss3d::resource;

fn main() {
    do window::Window::spawn("Kiss3d: cube") |window| {
        let mut c        = window.add_sphere(1.0);
        let     material = Rc::new(RefCell::new(~NormalMaterial::new() as ~Material));

        c.set_material(material);

        window.render_loop(|_| {
            c.prepend_rotation(&Vec3::new(0.0f32, 0.014, 0.0))
        })
    }
}

// A material that draws normals
struct NormalMaterial {
    priv program:    GLuint,
    priv vshader:    GLuint,
    priv fshader:    GLuint,
    priv position:   GLuint,
    priv normal:     GLuint,
    priv view:       GLint,
    priv transform:  GLint,
    priv scale:      GLint,
}

impl NormalMaterial {
    pub fn new() -> NormalMaterial {
        unsafe {
            let (program, vshader, fshader) =
                resource::load_shader_program(NORMAL_VERTEX_SRC, NORMAL_FRAGMENT_SRC);

            gl::UseProgram(program);

            NormalMaterial {
                program:   program,
                vshader:   vshader,
                fshader:   fshader,
                position:  gl::GetAttribLocation(program, "position".to_c_str().unwrap()) as GLuint,
                normal:    gl::GetAttribLocation(program, "normal".to_c_str().unwrap()) as GLuint,
                transform: gl::GetUniformLocation(program, "transform".to_c_str().unwrap()),
                scale:     gl::GetUniformLocation(program, "scale".to_c_str().unwrap()),
                view:      gl::GetUniformLocation(program, "view".to_c_str().unwrap()),
            }
        }
    }
}

impl Material for NormalMaterial {
    fn render(&mut self,
              pass:   uint,
              camera: &mut Camera,
              _:      &Light,
              data:   &ObjectData,
              mesh:   &mut Mesh) {
        gl::UseProgram(self.program);
        gl::EnableVertexAttribArray(self.position);
        gl::EnableVertexAttribArray(self.normal);

        /*
         *
         * Setup camera and light.
         *
         */
        camera.upload(pass, self.view);

        /*
         *
         * Setup object-related stuffs.
         *
         */
        let formated_transform:  Mat4<f32> = na::to_homogeneous(data.transform());

        unsafe {
            gl::UniformMatrix4fv(self.transform,
                                 1,
                                 gl::FALSE as u8,
                                 cast::transmute(&formated_transform));

            gl::UniformMatrix3fv(self.scale, 1, gl::FALSE as u8, cast::transmute(data.scale()));

            mesh.bind_coords(self.position);
            mesh.bind_normals(self.normal);
            mesh.bind_faces();

            gl::DrawElements(gl::TRIANGLES,
                             mesh.num_pts() as GLint,
                             gl::UNSIGNED_INT,
                             ptr::null());
        }

        mesh.unbind();

        gl::DisableVertexAttribArray(self.position);
        gl::DisableVertexAttribArray(self.normal);
    }
}

impl Drop for NormalMaterial {
    fn drop(&mut self) {
        gl::DeleteProgram(self.program);
        gl::DeleteShader(self.fshader);
        gl::DeleteShader(self.vshader);
    }
}


static NORMAL_VERTEX_SRC: &'static str =
"#version 120
attribute vec3 position;
attribute vec3 normal;
uniform mat4 view;
uniform mat4 transform;
uniform mat3 scale;
varying vec3 ls_normal;

void main() {
    ls_normal   = normal;
    gl_Position = view * transform * mat4(scale) * vec4(position, 1.0);
}
";

static NORMAL_FRAGMENT_SRC: &'static str =
"#version 120
varying vec3 ls_normal;

void main() {
    gl_FragColor = vec4((ls_normal + 1.0) / 2.0, 1.0);
}
";
