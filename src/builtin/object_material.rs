use std::ptr;
use gl;
use gl::types::*;
use na::{Point2, Point3, Vector3, Matrix3, Matrix4, Isometry3};
use resource::Material;
use scene::ObjectData;
use light::Light;
use camera::Camera;
use resource::{Mesh, Shader, ShaderAttribute, ShaderUniform};

#[path = "../error.rs"]
mod error;

/// The default material used to draw objects.
pub struct ObjectMaterial {
    shader:     Shader,
    pos:        ShaderAttribute<Point3<f32>>,
    normal:     ShaderAttribute<Vector3<f32>>,
    tex_coord:  ShaderAttribute<Point2<f32>>,
    light:      ShaderUniform<Point3<f32>>,
    color:      ShaderUniform<Point3<f32>>,
    transform:  ShaderUniform<Matrix4<f32>>,
    scale:      ShaderUniform<Matrix3<f32>>,
    ntransform: ShaderUniform<Matrix3<f32>>,
    view:       ShaderUniform<Matrix4<f32>>
}

impl ObjectMaterial {
    /// Creates a new `ObjectMaterial`.
    pub fn new() -> ObjectMaterial {
        // load the shader
        let mut shader = Shader::new_from_str(OBJECT_VERTEX_SRC, OBJECT_FRAGMENT_SRC);

        shader.use_program();

        // get the variables locations
        ObjectMaterial {
            pos:        shader.get_attrib("position").unwrap(),
            normal:     shader.get_attrib("normal").unwrap(),
            tex_coord:  shader.get_attrib("tex_coord_v").unwrap(),
            light:      shader.get_uniform("light_position").unwrap(),
            color:      shader.get_uniform("color").unwrap(),
            transform:  shader.get_uniform("transform").unwrap(),
            scale:      shader.get_uniform("scale").unwrap(),
            ntransform: shader.get_uniform("ntransform").unwrap(),
            view:       shader.get_uniform("view").unwrap(),
            shader:     shader
        }
    }

    fn activate(&mut self) {
        self.shader.use_program();
        self.pos.enable();
        self.normal.enable();
        self.tex_coord.enable();
    }

    fn deactivate(&mut self) {
        self.pos.disable();
        self.normal.disable();
        self.tex_coord.disable();
    }
}

impl Material for ObjectMaterial {
    fn render(&mut self,
              pass:      usize,
              transform: &Isometry3<f32>,
              scale:     &Vector3<f32>,
              camera:    &mut Camera,
              light:     &Light,
              data:      &ObjectData,
              mesh:      &mut Mesh) {
        self.activate();


        /*
         *
         * Setup camera and light.
         *
         */
        camera.upload(pass, &mut self.view);

        let pos = match *light {
            Light::Absolute(ref p) => p.clone(),
            Light::StickToCamera   => camera.eye()
        };

        self.light.upload(&pos);

        /*
         *
         * Setup object-related stuffs.
         *
         */
        let formated_transform  = transform.to_homogeneous();
        let formated_ntransform = transform.rotation.to_rotation_matrix().unwrap();
        let formated_scale      = Matrix3::from_diagonal(&Vector3::new(scale.x, scale.y, scale.z));

        unsafe {
            self.transform.upload(&formated_transform);
            self.ntransform.upload(&formated_ntransform);
            self.scale.upload(&formated_scale);
            self.color.upload(data.color());

            mesh.bind(&mut self.pos, &mut self.normal, &mut self.tex_coord);

            verify!(gl::ActiveTexture(gl::TEXTURE0));
            verify!(gl::BindTexture(gl::TEXTURE_2D, data.texture().id()));

            if data.surface_rendering_active() {
                if data.backface_culling_enabled() {
                    verify!(gl::Enable(gl::CULL_FACE));
                }
                else {
                    verify!(gl::Disable(gl::CULL_FACE));
                }

                verify!(gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL));
                verify!(gl::DrawElements(
                            gl::TRIANGLES,
                            mesh.num_pts() as GLint,
                            gl::UNSIGNED_INT,
                            ptr::null()));
            }

            if data.lines_width() != 0.0 {
                verify!(gl::Disable(gl::CULL_FACE));
                verify!(gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE));
                gl::LineWidth(data.lines_width());
                verify!(gl::DrawElements(
                            gl::TRIANGLES,
                            mesh.num_pts() as GLint,
                            gl::UNSIGNED_INT,
                            ptr::null()));
                gl::LineWidth(1.0);
            }

            if data.points_size() != 0.0 {
                verify!(gl::Disable(gl::CULL_FACE));
                verify!(gl::PolygonMode(gl::FRONT_AND_BACK, gl::POINT));
                gl::PointSize(data.points_size());
                verify!(gl::DrawElements(
                            gl::TRIANGLES,
                            mesh.num_pts() as GLint,
                            gl::UNSIGNED_INT,
                            ptr::null()));
                gl::PointSize(1.0);
            }
        }

        mesh.unbind();
        self.deactivate();
    }
}

/// Vertex shader of the default object material.
pub static OBJECT_VERTEX_SRC:   &'static str = A_VERY_LONG_STRING;
/// Fragment shader of the default object material.
pub static OBJECT_FRAGMENT_SRC: &'static str = ANOTHER_VERY_LONG_STRING;

const A_VERY_LONG_STRING: &'static str =
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

// phong-like lighting (heavily) inspired
// by http://www.opengl.org/sdk/docs/tutorials/ClockworkCoders/lighting.php
const ANOTHER_VERY_LONG_STRING: &'static str =
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
      vec4 Iamb = vec4(color, 1.0);

      //calculate Diffuse Term:
      vec4 Idiff1 = vec4(1.0, 1.0, 1.0, 1.0) * max(dot(ws_normal,L), 0.0);
      Idiff1 = clamp(Idiff1, 0.0, 1.0);

      // double sided lighting:
      vec4 Idiff2 = vec4(1.0, 1.0, 1.0, 1.0) * max(dot(-ws_normal,L), 0.0);
      Idiff2 = clamp(Idiff2, 0.0, 1.0);

      vec4 tex_color = texture2D(tex, tex_coord);
      gl_FragColor   = tex_color * (Iamb + (Idiff1 + Idiff2) / 2) / 2;
    }";
