use camera::Camera;
use context::Context;
use light::Light;
use na::{Isometry3, Matrix3, Matrix4, Point3, Vector3};
use resource::Material;
use resource::{Effect, Mesh, ShaderAttribute, ShaderUniform};
use scene::ObjectData;

#[path = "../error.rs"]
mod error;

/// A material that draws normals of an object.
pub struct NormalsMaterial {
    shader: Effect,
    position: ShaderAttribute<Point3<f32>>,
    normal: ShaderAttribute<Vector3<f32>>,
    proj: ShaderUniform<Matrix4<f32>>,
    view: ShaderUniform<Matrix4<f32>>,
    transform: ShaderUniform<Matrix4<f32>>,
    scale: ShaderUniform<Matrix3<f32>>,
}

impl NormalsMaterial {
    /// Creates a new NormalsMaterial.
    pub fn new() -> NormalsMaterial {
        let mut shader = Effect::new_from_str(NORMAL_VERTEX_SRC, NORMAL_FRAGMENT_SRC);

        shader.use_program();

        NormalsMaterial {
            position: shader.get_attrib("position").unwrap(),
            normal: shader.get_attrib("normal").unwrap(),
            transform: shader.get_uniform("transform").unwrap(),
            scale: shader.get_uniform("scale").unwrap(),
            view: shader.get_uniform("view").unwrap(),
            proj: shader.get_uniform("proj").unwrap(),
            shader: shader,
        }
    }
}

impl Material for NormalsMaterial {
    fn render(
        &mut self,
        pass: usize,
        transform: &Isometry3<f32>,
        scale: &Vector3<f32>,
        camera: &mut Camera,
        _: &Light,
        data: &ObjectData,
        mesh: &mut Mesh,
    ) {
        let ctxt = Context::get();
        if !data.surface_rendering_active() {
            return;
        }
        // enable/disable culling.
        if data.backface_culling_enabled() {
            verify!(ctxt.enable(Context::CULL_FACE));
        } else {
            verify!(ctxt.disable(Context::CULL_FACE));
        }

        self.shader.use_program();
        self.position.enable();
        self.normal.enable();

        /*
         *
         * Setup camera and light.
         *
         */
        camera.upload(pass, &mut self.view, &mut self.proj);

        /*
         *
         * Setup object-related stuffs.
         *
         */
        let formated_transform = transform.to_homogeneous();
        let formated_scale = Matrix3::from_diagonal(&Vector3::new(scale.x, scale.y, scale.z));

        self.transform.upload(&formated_transform);
        self.scale.upload(&formated_scale);

        mesh.bind_coords(&mut self.position);
        mesh.bind_normals(&mut self.normal);
        mesh.bind_faces();

        unsafe {
            ctxt.draw_elements(
                Context::TRIANGLES,
                mesh.num_pts() as i32,
                Context::UNSIGNED_INT,
                0,
            );
        }

        mesh.unbind();

        self.position.disable();
        self.normal.disable();
    }
}

/// A vertex shader for coloring each point of an object depending on its normal.
pub static NORMAL_VERTEX_SRC: &'static str = A_VERY_LONG_STRING;

/// A fragment shader for coloring each point of an object depending on its normal.
pub static NORMAL_FRAGMENT_SRC: &'static str = ANOTHER_VERY_LONG_STRING;

const A_VERY_LONG_STRING: &'static str = "#version 100
attribute vec3 position;
attribute vec3 normal;
uniform mat4 proj;
uniform mat4 view;
uniform mat4 transform;
uniform mat3 scale;
varying vec3 ls_normal;

void main() {
    ls_normal   = normal;
    gl_Position = proj * view * transform * mat4(scale) * vec4(position, 1.0);
}
";

const ANOTHER_VERY_LONG_STRING: &'static str = "#version 100
#ifdef GL_FRAGMENT_PRECISION_HIGH
   precision highp float;
#else
   precision mediump float;
#endif

varying vec3 ls_normal;

void main() {
    gl_FragColor = vec4((ls_normal + 1.0) / 2.0, 1.0);
}
";
