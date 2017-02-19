use std::ptr;
use gl;
use gl::types::*;
use na::{Point3, Point2, Vector3, Matrix3, Matrix4, Isometry3};
use resource::Material;
use scene::ObjectData;
use light::Light;
use camera::Camera;
use resource::{Mesh, Shader, ShaderAttribute, ShaderUniform};

#[path = "../error.rs"]
mod error;


/// A material that draws normals of an object.
pub struct UvsMaterial {
    shader:    Shader,
    position:  ShaderAttribute<Point3<f32>>,
    uvs:       ShaderAttribute<Point2<f32>>,
    view:      ShaderUniform<Matrix4<f32>>,
    transform: ShaderUniform<Matrix4<f32>>,
    scale:     ShaderUniform<Matrix3<f32>>
}

impl UvsMaterial {
    /// Creates a new UvsMaterial.
    pub fn new() -> UvsMaterial {
        let mut shader = Shader::new_from_str(UVS_VERTEX_SRC, UVS_FRAGMENT_SRC);

        shader.use_program();

        UvsMaterial {
            position:  shader.get_attrib("position").unwrap(),
            uvs:       shader.get_attrib("uvs").unwrap(),
            transform: shader.get_uniform("transform").unwrap(),
            scale:     shader.get_uniform("scale").unwrap(),
            view:      shader.get_uniform("view").unwrap(),
            shader:    shader
        }
    }
}

impl Material for UvsMaterial {
    fn render(&mut self,
              pass:      usize,
              transform: &Isometry3<f32>,
              scale:     &Vector3<f32>,
              camera:    &mut Camera,
              _:         &Light,
              data:      &ObjectData,
              mesh:      &mut Mesh) {
        if !data.surface_rendering_active() {
            return
        }
        // enable/disable culling.
        if data.backface_culling_enabled() {
            verify!(gl::Enable(gl::CULL_FACE));
        }
        else {
            verify!(gl::Disable(gl::CULL_FACE));
        }


        self.shader.use_program();
        self.position.enable();
        self.uvs.enable();

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
        let formated_transform = transform.to_homogeneous();
        let formated_scale     = Matrix3::from_diagonal(&Vector3::new(scale.x, scale.y, scale.z));

        self.transform.upload(&formated_transform);
        self.scale.upload(&formated_scale);

        mesh.bind_coords(&mut self.position);
        mesh.bind_uvs(&mut self.uvs);
        mesh.bind_faces();

        unsafe {
            gl::DrawElements(gl::TRIANGLES, mesh.num_pts() as GLint, gl::UNSIGNED_INT, ptr::null());
        }

        mesh.unbind();

        self.position.disable();
        self.uvs.disable();
    }
}

/// A vertex shader for coloring each point of an object depending on its texture coordinates.
pub static UVS_VERTEX_SRC: &'static str = A_VERY_LONG_STRING;

/// A fragment shader for coloring each point of an object depending on its texture coordinates.
pub static UVS_FRAGMENT_SRC: &'static str = ANOTHER_VERY_LONG_STRING;

const A_VERY_LONG_STRING: &'static str =
"#version 120
attribute vec3 position;
attribute vec3 uvs;
uniform mat4 view;
uniform mat4 transform;
uniform mat3 scale;
varying vec3 uv_as_a_color;

void main() {
    uv_as_a_color  = vec3(uvs.xy, 0.0);
    gl_Position = view * transform * mat4(scale) * vec4(position, 1.0);
}
";

const ANOTHER_VERY_LONG_STRING: &'static str =
"#version 120
varying vec3 uv_as_a_color;

void main() {
    gl_FragColor = vec4(uv_as_a_color, 1.0);
}
";

