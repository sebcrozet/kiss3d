use crate::camera::Camera;
use crate::context::Context;
use crate::light::Light;
use na::{Isometry3, Matrix3, Matrix4, Point3, Vector3};
use crate::resource::Material;
use crate::resource::{Effect, Mesh, ShaderAttribute, ShaderUniform, Cubemap};
use crate::scene::ObjectData;
use std::rc::Rc;

#[path = "../error.rs"]
mod error;

/// A material that draws skybox
pub struct CubemapMaterial {
    shader: Effect,
    position: ShaderAttribute<Point3<f32>>,
    view: ShaderUniform<Matrix4<f32>>,
    proj: ShaderUniform<Matrix4<f32>>,
    transform: ShaderUniform<Matrix4<f32>>,
    scale: ShaderUniform<Matrix3<f32>>,
    cubemap: Rc<Cubemap>
}

impl CubemapMaterial {
    pub fn new(c: Rc<Cubemap>) -> CubemapMaterial {
        let mut shader = Effect::new_from_str(NORMAL_VERTEX_SRC, NORMAL_FRAGMENT_SRC);

        shader.use_program();

        CubemapMaterial {
            position: shader.get_attrib("position").unwrap(),
            transform: shader.get_uniform("transform").unwrap(),
            scale: shader.get_uniform("scale").unwrap(),
            view: shader.get_uniform("view").unwrap(),
            proj: shader.get_uniform("proj").unwrap(),
            shader: shader,
            cubemap: c
        }
    }
}

impl Material for CubemapMaterial {
    fn render(
        &mut self,
        pass: usize,
        transform: &Isometry3<f32>,
        scale: &Vector3<f32>,
        camera: &mut dyn Camera,
        _: &Light,
        _data: &ObjectData,
        mesh: &mut Mesh,
    ) {
        self.shader.use_program();
        self.position.enable();

        /*
         * Setup camera and light.
         */
        camera.upload(pass, &mut self.proj, &mut self.view);

        /*
         * Setup object-related stuffs.
         */
        let formated_transform = transform.to_homogeneous();
        let formated_scale = Matrix3::from_diagonal(&Vector3::new(scale.x, scale.y, scale.z));

        self.transform.upload(&formated_transform);
        self.scale.upload(&formated_scale);

        mesh.bind_coords(&mut self.position);
        mesh.bind_faces();

        let ctxt = Context::get();
        verify!(ctxt.active_texture(Context::TEXTURE0));
        verify!(ctxt.bind_cubemap(Context::TEXTURE_CUBE_MAP, Some(&*self.cubemap)));

        ctxt.cull_face(Context::FRONT);

        ctxt.draw_elements(
            Context::TRIANGLES,
            mesh.num_pts() as i32,
            Context::UNSIGNED_SHORT,
            0,
        );

        mesh.unbind();

        self.position.disable();
    }
}

static NORMAL_VERTEX_SRC: &'static str = "#version 150
attribute vec3 position;
uniform mat4 view;
uniform mat4 proj;
uniform mat4 transform;
uniform mat3 scale;
uniform samplerCube skybox;
varying vec3 tex_coords;

void main() {
    tex_coords  = position;
    gl_Position = proj * view * transform * mat4(scale) * vec4(position, 1.0);
}
";

static NORMAL_FRAGMENT_SRC: &'static str = "#version 150
#ifdef GL_FRAGMENT_PRECISION_HIGH
   precision highp float;
#else
   precision mediump float;
#endif
varying vec3 tex_coords;
uniform samplerCube skybox;

void main() {
    gl_FragColor = texture(skybox, tex_coords);
}
";
