extern crate gl;
extern crate kiss3d;
extern crate nalgebra as na;

use std::ptr;
use std::rc::Rc;
use std::cell::RefCell;
use gl::types::GLint;
use na::{Pnt3, Vec3, Mat3, Mat4, Iso3};
use kiss3d::window::Window;
use kiss3d::scene::ObjectData;
use kiss3d::camera::Camera;
use kiss3d::light::Light;
use kiss3d::resource::{Shader, ShaderAttribute, ShaderUniform, Material, Mesh};

fn main() {
    let mut window = Window::new("Kiss3d: custom_material");
    let mut c      = window.add_sphere(1.0);
    let material   = Rc::new(RefCell::new(Box::new(NormalMaterial::new()) as Box<Material + 'static>));

    c.set_material(material);

    while window.render() {
        c.prepend_to_local_rotation(&Vec3::new(0.0f32, 0.014, 0.0));
    }
}

// A material that draws normals
pub struct NormalMaterial {
    shader:    Shader,
    position:  ShaderAttribute<Pnt3<f32>>,
    normal:    ShaderAttribute<Vec3<f32>>,
    view:      ShaderUniform<Mat4<f32>>,
    transform: ShaderUniform<Mat4<f32>>,
    scale:     ShaderUniform<Mat3<f32>>
}

impl NormalMaterial {
    pub fn new() -> NormalMaterial {
        let mut shader = Shader::new_from_str(NORMAL_VERTEX_SRC, NORMAL_FRAGMENT_SRC);

        shader.use_program();

        NormalMaterial {
            position:  shader.get_attrib("position").unwrap(),
            normal:    shader.get_attrib("normal").unwrap(),
            transform: shader.get_uniform("transform").unwrap(),
            scale:     shader.get_uniform("scale").unwrap(),
            view:      shader.get_uniform("view").unwrap(),
            shader:    shader
        }
    }
}

impl Material for NormalMaterial {
    fn render(&mut self,
              pass:      usize,
              transform: &Iso3<f32>,
              scale:     &Vec3<f32>,
              camera:    &mut Camera,
              _:         &Light,
              _:         &ObjectData,
              mesh:      &mut Mesh) {
        self.shader.use_program();
        self.position.enable();
        self.normal.enable();

        /*
         *
         * Setup camera and light.
         *
         */
        camera.upload(pass, &mut self.view);

        /*
         *
         * Setup object-related stuffs.
         *
         */
        let formated_transform: Mat4<f32> = na::to_homogeneous(transform);
        // FIXME: add a function `na::diagonal(scale)` to nalgebra.
        let formated_scale:     Mat3<f32> = Mat3::new(scale.x, 0.0, 0.0, 0.0, scale.y, 0.0, 0.0, 0.0, scale.z);

        self.transform.upload(&formated_transform);
        self.scale.upload(&formated_scale);

        mesh.bind_coords(&mut self.position);
        mesh.bind_normals(&mut self.normal);
        mesh.bind_faces();

        unsafe {
            gl::DrawElements(gl::TRIANGLES,
                             mesh.num_pts() as GLint,
                             gl::UNSIGNED_INT,
                             ptr::null());
        }

        mesh.unbind();

        self.position.disable();
        self.normal.disable();
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
