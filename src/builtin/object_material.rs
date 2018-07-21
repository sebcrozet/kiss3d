use camera::Camera;
use context::Context;
use light::Light;
use na::{Isometry3, Matrix3, Matrix4, Point2, Point3, Vector3};
use resource::Material;
use resource::{Effect, Mesh, ShaderAttribute, ShaderUniform};
use scene::ObjectData;

#[path = "../error.rs"]
mod error;

/// The default material used to draw objects.
pub struct ObjectMaterial {
    effect: Effect,
    pos: ShaderAttribute<Point3<f32>>,
    normal: ShaderAttribute<Vector3<f32>>,
    tex_coord: ShaderAttribute<Point2<f32>>,
    light: ShaderUniform<Point3<f32>>,
    color: ShaderUniform<Point3<f32>>,
    transform: ShaderUniform<Matrix4<f32>>,
    scale: ShaderUniform<Matrix3<f32>>,
    ntransform: ShaderUniform<Matrix3<f32>>,
    proj: ShaderUniform<Matrix4<f32>>,
    view: ShaderUniform<Matrix4<f32>>,
}

impl ObjectMaterial {
    /// Creates a new `ObjectMaterial`.
    pub fn new() -> ObjectMaterial {
        // load the effect
        let mut effect = Effect::new_from_str(OBJECT_VERTEX_SRC, OBJECT_FRAGMENT_SRC);

        effect.use_program();

        // get the variables locations
        ObjectMaterial {
            pos: effect.get_attrib("position").unwrap(),
            normal: effect.get_attrib("normal").unwrap(),
            tex_coord: effect.get_attrib("tex_coord").unwrap(),
            light: effect.get_uniform("light_position").unwrap(),
            color: effect.get_uniform("color").unwrap(),
            transform: effect.get_uniform("transform").unwrap(),
            scale: effect.get_uniform("scale").unwrap(),
            ntransform: effect.get_uniform("ntransform").unwrap(),
            view: effect.get_uniform("view").unwrap(),
            proj: effect.get_uniform("proj").unwrap(),
            effect: effect,
        }
    }

    fn activate(&mut self) {
        self.effect.use_program();
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
    fn render(
        &mut self,
        pass: usize,
        transform: &Isometry3<f32>,
        scale: &Vector3<f32>,
        camera: &mut Camera,
        light: &Light,
        data: &ObjectData,
        mesh: &mut Mesh,
    ) {
        let ctxt = Context::get();
        self.activate();

        /*
         *
         * Setup camera and light.
         *
         */
        camera.upload(pass, &mut self.proj, &mut self.view);

        let pos = match *light {
            Light::Absolute(ref p) => p.clone(),
            Light::StickToCamera => camera.eye(),
        };

        self.light.upload(&pos);

        /*
         *
         * Setup object-related stuffs.
         *
         */
        let formated_transform = transform.to_homogeneous();
        let formated_ntransform = transform.rotation.to_rotation_matrix().unwrap();
        let formated_scale = Matrix3::from_diagonal(&Vector3::new(scale.x, scale.y, scale.z));

        unsafe {
            self.transform.upload(&formated_transform);
            self.ntransform.upload(&formated_ntransform);
            self.scale.upload(&formated_scale);
            self.color.upload(data.color());

            mesh.bind(&mut self.pos, &mut self.normal, &mut self.tex_coord);

            verify!(ctxt.active_texture(Context::TEXTURE0));
            verify!(ctxt.bind_texture(Context::TEXTURE_2D, Some(&*data.texture())));

            if data.surface_rendering_active() {
                if data.backface_culling_enabled() {
                    verify!(ctxt.enable(Context::CULL_FACE));
                } else {
                    verify!(ctxt.disable(Context::CULL_FACE));
                }

                let _ = verify!(ctxt.polygon_mode(Context::FRONT_AND_BACK, Context::FILL));
                verify!(ctxt.draw_elements(
                    Context::TRIANGLES,
                    mesh.num_pts() as i32,
                    Context::UNSIGNED_SHORT,
                    0
                ));
            }

            if data.lines_width() != 0.0 {
                verify!(ctxt.disable(Context::CULL_FACE));
                ignore!(ctxt.line_width(data.lines_width()));

                if verify!(ctxt.polygon_mode(Context::FRONT_AND_BACK, Context::LINE)) {
                    verify!(ctxt.draw_elements(
                        Context::TRIANGLES,
                        mesh.num_pts() as i32,
                        Context::UNSIGNED_SHORT,
                        0
                    ));
                } else {
                    mesh.bind_edges();
                    verify!(ctxt.draw_elements(
                        Context::LINES,
                        mesh.num_pts() as i32 * 2,
                        Context::UNSIGNED_SHORT,
                        0
                    ));
                }
                ctxt.line_width(1.0);
            }

            if data.points_size() != 0.0 {
                verify!(ctxt.disable(Context::CULL_FACE));
                ctxt.point_size(data.points_size());
                if verify!(ctxt.polygon_mode(Context::FRONT_AND_BACK, Context::POINT)) {
                    verify!(ctxt.draw_elements(
                        Context::TRIANGLES,
                        mesh.num_pts() as i32,
                        Context::UNSIGNED_SHORT,
                        0
                    ));
                } else {
                    verify!(ctxt.draw_elements(
                        Context::POINTS,
                        mesh.num_pts() as i32,
                        Context::UNSIGNED_SHORT,
                        0
                    ));
                }
                ctxt.point_size(1.0);
            }
        }

        mesh.unbind();
        self.deactivate();
    }
}

/// Vertex shader of the default object material.
pub static OBJECT_VERTEX_SRC: &'static str = A_VERY_LONG_STRING;
/// Fragment shader of the default object material.
pub static OBJECT_FRAGMENT_SRC: &'static str = ANOTHER_VERY_LONG_STRING;

const A_VERY_LONG_STRING: &'static str = "#version 100
attribute vec3 position;
attribute vec2 tex_coord;
attribute vec3 normal;

uniform mat3 ntransform, scale;
uniform mat4 proj, view, transform;
uniform vec3 light_position;

varying vec3 local_light_position;
varying vec2 tex_coord_v;
varying vec3 normalInterp;
varying vec3 vertPos;

void main(){
    gl_Position = proj * view * transform * vec4(scale * position, 1.0);
    vec4 vertPos4 = view * transform * vec4(scale * position, 1.0);
    vertPos = vec3(vertPos4) / vertPos4.w;
    normalInterp = mat3(view) * ntransform * normal;
    tex_coord_v = tex_coord;
    local_light_position = (view * vec4(light_position, 1.0)).xyz;
}";

// phong-like lighting (heavily) inspired
// http://www.mathematik.uni-marburg.de/~thormae/lectures/graphics1/code/WebGLShaderLightMat/ShaderLightMat.html
const ANOTHER_VERY_LONG_STRING: &'static str = "#version 100
#ifdef GL_FRAGMENT_PRECISION_HIGH
   precision highp float;
#else
   precision mediump float;
#endif

varying vec3 local_light_position;
varying vec2 tex_coord_v;
varying vec3 normalInterp;
varying vec3 vertPos;

uniform vec3 color;
uniform sampler2D tex;
const vec3 specColor = vec3(0.4, 0.4, 0.4);

void main() {
  vec3 normal = normalize(normalInterp);
  vec3 lightDir = normalize(local_light_position - vertPos);

  float lambertian = max(dot(lightDir, normal), 0.0);
  float specular = 0.0;

  if(lambertian > 0.0) {
    vec3 viewDir = normalize(-vertPos);
    vec3 halfDir = normalize(lightDir + viewDir);
    float specAngle = max(dot(halfDir, normal), 0.0);
    specular = pow(specAngle, 30.0);
  }

  vec4 tex_color = texture2D(tex, tex_coord_v);
  gl_FragColor = tex_color * vec4(color / 3.0 +
                                  lambertian * color / 3.0 +
                                  specular * specColor / 3.0, 1.0);
}";
