use crate::context::Context;
use crate::planar_camera::PlanarCamera;
use crate::resource::vertex_index::VERTEX_INDEX_TYPE;
use crate::resource::PlanarMaterial;
use crate::resource::{Effect, PlanarMesh, ShaderAttribute, ShaderUniform};
use crate::scene::{PlanarInstancesBuffers, PlanarObjectData};
use crate::{ignore, verify};
use na::{Isometry2, Matrix2, Matrix3, Point2, Point3, Vector2};

/// The default material used to draw objects.
pub struct PlanarObjectMaterial {
    effect: Effect,
    pos: ShaderAttribute<Point2<f32>>,
    tex_coord: ShaderAttribute<Point2<f32>>,
    inst_tra: ShaderAttribute<Point2<f32>>,
    inst_color: ShaderAttribute<[f32; 4]>,
    inst_deformation: ShaderAttribute<Matrix2<f32>>,
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
            inst_tra: effect.get_attrib("inst_tra").unwrap(),
            inst_color: effect.get_attrib("inst_color").unwrap(),
            inst_deformation: effect.get_attrib("inst_deformation").unwrap(),
            color: effect.get_uniform("color").unwrap(),
            scale: effect.get_uniform("scale").unwrap(),
            model: effect.get_uniform("model").unwrap(),
            view: effect.get_uniform("view").unwrap(),
            proj: effect.get_uniform("proj").unwrap(),
            effect,
        }
    }

    fn activate(&mut self) {
        self.effect.use_program();
        self.pos.enable();
        self.tex_coord.enable();
        self.inst_tra.enable();
        self.inst_color.enable();
        self.inst_deformation.enable();
    }

    fn deactivate(&mut self) {
        self.pos.disable();
        self.tex_coord.disable();
        self.inst_tra.disable();
        self.inst_color.disable();
        self.inst_deformation.disable();
    }
}

impl PlanarMaterial for PlanarObjectMaterial {
    fn render(
        &mut self,
        model: &Isometry2<f32>,
        scale: &Vector2<f32>,
        camera: &mut dyn PlanarCamera,
        data: &PlanarObjectData,
        instances: &mut PlanarInstancesBuffers,
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
        let instance_count = instances.len() as i32;

        unsafe {
            self.model.upload(&formated_transform);
            self.scale.upload(&formated_scale);

            mesh.bind(&mut self.pos, &mut self.tex_coord);

            // NOTE: attrib divisors different than 1 are very slow to render. So we
            //       require all instanced attributes to be provided for each instance.
            self.inst_tra.bind(&mut instances.positions);
            verify!(ctxt.vertex_attrib_divisor(self.inst_tra.id(), 1));
            self.inst_color.bind(&mut instances.colors);
            verify!(ctxt.vertex_attrib_divisor(self.inst_color.id(), 1));
            self.inst_deformation.bind(&mut instances.deformations);
            verify!(ctxt.vertex_attrib_divisor(self.inst_deformation.id(), 1));

            verify!(ctxt.active_texture(Context::TEXTURE0));
            verify!(ctxt.bind_texture(Context::TEXTURE_2D, Some(&*data.texture())));
            verify!(ctxt.disable(Context::CULL_FACE));

            if data.surface_rendering_active() {
                self.color.upload(data.color());

                let _ = verify!(ctxt.polygon_mode(Context::FRONT_AND_BACK, Context::FILL));
                verify!(ctxt.draw_elements_instanced(
                    Context::TRIANGLES,
                    mesh.num_pts() as i32,
                    VERTEX_INDEX_TYPE,
                    0,
                    instance_count,
                ));
            }

            if data.lines_width() != 0.0 {
                self.color
                    .upload(data.lines_color().unwrap_or(data.color()));

                verify!(ctxt.disable(Context::CULL_FACE));
                ignore!(ctxt.line_width(data.lines_width()));

                if verify!(ctxt.polygon_mode(Context::FRONT_AND_BACK, Context::LINE)) {
                    verify!(ctxt.draw_elements_instanced(
                        Context::TRIANGLES,
                        mesh.num_pts() as i32,
                        VERTEX_INDEX_TYPE,
                        0,
                        instance_count
                    ));
                } else {
                    mesh.bind_edges();
                    verify!(ctxt.draw_elements_instanced(
                        Context::LINES,
                        mesh.num_pts() as i32 * 2,
                        VERTEX_INDEX_TYPE,
                        0,
                        instance_count
                    ));
                }

                ctxt.line_width(1.0);
            }

            if data.points_size() != 0.0 {
                self.color.upload(data.color());

                verify!(ctxt.disable(Context::CULL_FACE));
                ctxt.point_size(data.points_size());
                if verify!(ctxt.polygon_mode(Context::FRONT_AND_BACK, Context::POINT)) {
                    verify!(ctxt.draw_elements_instanced(
                        Context::TRIANGLES,
                        mesh.num_pts() as i32,
                        VERTEX_INDEX_TYPE,
                        0,
                        instance_count
                    ));
                } else {
                    verify!(ctxt.draw_elements_instanced(
                        Context::POINTS,
                        mesh.num_pts() as i32,
                        VERTEX_INDEX_TYPE,
                        0,
                        instance_count
                    ));
                }
                ctxt.point_size(1.0);
            }
        }

        // Reset attrib divisors so they don’t affect other shaders.
        verify!(ctxt.vertex_attrib_divisor(self.inst_tra.id(), 0));
        verify!(ctxt.vertex_attrib_divisor(self.inst_color.id(), 0));
        verify!(ctxt.vertex_attrib_divisor(self.inst_deformation.id(), 0));

        mesh.unbind();
        self.deactivate();
    }
}

/// Vertex shader of the default object material.
static OBJECT_VERTEX_SRC: &str = A_VERY_LONG_STRING;
/// Fragment shader of the default object material.
static OBJECT_FRAGMENT_SRC: &str = ANOTHER_VERY_LONG_STRING;

const A_VERY_LONG_STRING: &str = "#version 100
attribute vec2 position;
attribute vec2 tex_coord;
attribute vec2 inst_tra;
attribute vec4 inst_color;
attribute vec4 inst_deformation;

uniform mat2 scale;
uniform mat3 proj, view, model;

varying vec2 tex_coord_v;
varying vec4 vert_color;

void main(){
    mat2 def = mat2(
        vec2(inst_deformation[0], inst_deformation[1]),
        vec2(inst_deformation[2], inst_deformation[3])
    );
    vec3 projected_pos = proj * view * (vec3(inst_tra, 0.0) + model * vec3(def * scale * position, 1.0));
    projected_pos.z = 0.0;

    gl_Position = vec4(projected_pos, 1.0);
    tex_coord_v = tex_coord;
    vert_color = inst_color;
}";

const ANOTHER_VERY_LONG_STRING: &str = "#version 100
#ifdef GL_FRAGMENT_PRECISION_HIGH
   precision highp float;
#else
   precision mediump float;
#endif

varying vec2 tex_coord_v;
varying vec4 vert_color;

uniform sampler2D tex;
uniform vec3 color;

void main() {
  vec4 tex_color = texture2D(tex, tex_coord_v);
  gl_FragColor = tex_color * (vec4(color, 1.0) * vert_color);
}";
