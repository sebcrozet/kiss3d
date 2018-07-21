use context::Context;
use na::{Isometry2, Matrix2, Matrix3, Point2, Point3, Vector2};
use planar_camera::PlanarCamera;
use resource::PlanarMaterial;
use resource::{Effect, PlanarMesh, ShaderAttribute, ShaderUniform};
use scene::PlanarObjectData;

#[path = "../error.rs"]
mod error;

/// The default material used to draw objects.
pub struct PlanarObjectMaterial {
    effect: Effect,
    pos: ShaderAttribute<Point2<f32>>,
    tex_coord: ShaderAttribute<Point2<f32>>,
    color: ShaderUniform<Point3<f32>>,
    scale: ShaderUniform<Matrix2<f32>>,
    model: ShaderUniform<Matrix3<f32>>,
    view: ShaderUniform<Matrix3<f32>>,
    proj: ShaderUniform<Matrix3<f32>>,
}

impl PlanarObjectMaterial {
    /// Creates a new `PlanarObjectMaterial`.
    pub fn new() -> PlanarObjectMaterial {
        // load the effect
        let mut effect = Effect::new_from_str(OBJECT_VERTEX_SRC, OBJECT_FRAGMENT_SRC);

        effect.use_program();

        // get the variables locations
        PlanarObjectMaterial {
            pos: effect.get_attrib("position").unwrap(),
            tex_coord: effect.get_attrib("tex_coord").unwrap(),
            color: effect.get_uniform("color").unwrap(),
            scale: effect.get_uniform("scale").unwrap(),
            model: effect.get_uniform("model").unwrap(),
            view: effect.get_uniform("view").unwrap(),
            proj: effect.get_uniform("proj").unwrap(),
            effect: effect,
        }
    }

    fn activate(&mut self) {
        self.effect.use_program();
        self.pos.enable();
        self.tex_coord.enable();
    }

    fn deactivate(&mut self) {
        self.pos.disable();
        self.tex_coord.disable();
    }
}

impl PlanarMaterial for PlanarObjectMaterial {
    fn render(
        &mut self,
        model: &Isometry2<f32>,
        scale: &Vector2<f32>,
        camera: &mut PlanarCamera,
        data: &PlanarObjectData,
        mesh: &mut PlanarMesh,
    ) {
        let ctxt = Context::get();
        self.activate();

        /*
         *
         * Setup camera and light.
         *
         */
        camera.upload(&mut self.proj, &mut self.view);

        /*
         *
         * Setup object-related stuffs.
         *
         */
        let formated_transform = model.to_homogeneous();
        let formated_scale = Matrix2::from_diagonal(&Vector2::new(scale.x, scale.y));

        unsafe {
            self.model.upload(&formated_transform);
            self.scale.upload(&formated_scale);
            self.color.upload(data.color());

            mesh.bind(&mut self.pos, &mut self.tex_coord);

            verify!(ctxt.active_texture(Context::TEXTURE0));
            verify!(ctxt.bind_texture(Context::TEXTURE_2D, Some(&*data.texture())));
            verify!(ctxt.disable(Context::CULL_FACE));

            if data.surface_rendering_active() {
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
static OBJECT_VERTEX_SRC: &'static str = A_VERY_LONG_STRING;
/// Fragment shader of the default object material.
static OBJECT_FRAGMENT_SRC: &'static str = ANOTHER_VERY_LONG_STRING;

const A_VERY_LONG_STRING: &'static str = "#version 100
attribute vec2 position;
attribute vec2 tex_coord;

uniform mat2 scale;
uniform mat3 proj, view, model;

varying vec2 tex_coord_v;

void main(){
    vec3 projected_pos = proj * view * model * vec3(scale * position, 1.0);
    projected_pos.z = 0.0;

    gl_Position = vec4(projected_pos, 1.0);
    tex_coord_v = tex_coord;
}";

// phong-like lighting (heavily) inspired
// http://www.mathematik.uni-marburg.de/~thormae/lectures/graphics1/code/WebGLShaderLightMat/ShaderLightMat.html
const ANOTHER_VERY_LONG_STRING: &'static str = "#version 100
#ifdef GL_FRAGMENT_PRECISION_HIGH
   precision highp float;
#else
   precision mediump float;
#endif

varying vec2 tex_coord_v;

uniform sampler2D tex;
uniform vec3 color;

void main() {
  vec4 tex_color = texture2D(tex, tex_coord_v);
  gl_FragColor = tex_color * vec4(color, 1.0);
}";
