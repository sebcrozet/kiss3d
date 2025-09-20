use crate::camera::Camera;
use crate::context::Context;
use crate::light::Light;
use crate::resource::vertex_index::VERTEX_INDEX_TYPE;
use crate::resource::Material;
use crate::resource::{Effect, Mesh, ShaderAttribute, ShaderUniform};
use crate::scene::{InstancesBuffer, ObjectData};
use crate::{ignore, verify};
use na::{Isometry3, Matrix3, Matrix4, Point2, Point3, Vector3};

/// The default material used to draw objects.
pub struct ObjectMaterial {
    effect: Effect,
    pos: ShaderAttribute<Point3<f32>>,
    normal: ShaderAttribute<Vector3<f32>>,
    tex_coord: ShaderAttribute<Point2<f32>>,
    inst_tra: ShaderAttribute<Point3<f32>>,
    inst_color: ShaderAttribute<[f32; 4]>,
    inst_def0: ShaderAttribute<Vector3<f32>>,
    inst_def1: ShaderAttribute<Vector3<f32>>,
    inst_def2: ShaderAttribute<Vector3<f32>>,
    light: ShaderUniform<Point3<f32>>,
    color: ShaderUniform<Point3<f32>>,
    transform: ShaderUniform<Matrix4<f32>>,
    scale: ShaderUniform<Matrix3<f32>>,
    ntransform: ShaderUniform<Matrix3<f32>>,
    proj: ShaderUniform<Matrix4<f32>>,
    view: ShaderUniform<Matrix4<f32>>,
}

impl Default for ObjectMaterial {
    fn default() -> Self {
        Self::new()
    }
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
            inst_tra: effect.get_attrib("inst_tra").unwrap(),
            inst_color: effect.get_attrib("inst_color").unwrap(),
            inst_def0: effect.get_attrib("inst_def_0").unwrap(),
            inst_def1: effect.get_attrib("inst_def_1").unwrap(),
            inst_def2: effect.get_attrib("inst_def_2").unwrap(),
            light: effect.get_uniform("light_position").unwrap(),
            color: effect.get_uniform("color").unwrap(),
            transform: effect.get_uniform("transform").unwrap(),
            scale: effect.get_uniform("scale").unwrap(),
            ntransform: effect.get_uniform("ntransform").unwrap(),
            view: effect.get_uniform("view").unwrap(),
            proj: effect.get_uniform("proj").unwrap(),
            effect,
        }
    }

    fn activate(&mut self) {
        self.effect.use_program();
        self.pos.enable();
        self.normal.enable();
        self.tex_coord.enable();
        self.inst_tra.enable();
        self.inst_color.enable();
        self.inst_def0.enable();
        self.inst_def1.enable();
        self.inst_def2.enable();
    }

    fn deactivate(&mut self) {
        self.pos.disable();
        self.normal.disable();
        self.tex_coord.disable();
        self.inst_tra.disable();
        self.inst_color.disable();
        self.inst_def0.disable();
        self.inst_def1.disable();
        self.inst_def2.disable();
    }
}

impl Material for ObjectMaterial {
    fn render(
        &mut self,
        pass: usize,
        transform: &Isometry3<f32>,
        scale: &Vector3<f32>,
        camera: &mut dyn Camera,
        light: &Light,
        data: &ObjectData,
        instances: &mut InstancesBuffer,
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
            Light::Absolute(ref p) => *p,
            Light::StickToCamera => camera.eye(),
        };

        self.light.upload(&pos);

        /*
         *
         * Setup object-related stuffs.
         *
         */
        let formatted_transform = transform.to_homogeneous();
        let formatted_ntransform = transform.rotation.to_rotation_matrix().into_inner();
        let formatted_scale = Matrix3::from_diagonal(&Vector3::new(scale.x, scale.y, scale.z));
        let instance_count = instances.len() as i32;

        unsafe {
            self.transform.upload(&formatted_transform);
            self.ntransform.upload(&formatted_ntransform);
            self.scale.upload(&formatted_scale);

            mesh.bind(&mut self.pos, &mut self.normal, &mut self.tex_coord);

            // NOTE: attrib divisors different than 1 are very slow to render. So we
            //       require all instanced attributes to be provided for each instance.
            self.inst_tra.bind(&mut instances.positions);
            verify!(ctxt.vertex_attrib_divisor(self.inst_tra.id(), 1));
            self.inst_color.bind(&mut instances.colors);
            verify!(ctxt.vertex_attrib_divisor(self.inst_color.id(), 1));
            self.inst_def0
                .bind_sub_buffer(&mut instances.deformations, 2, 0);
            verify!(ctxt.vertex_attrib_divisor(self.inst_def0.id(), 1));
            self.inst_def1
                .bind_sub_buffer(&mut instances.deformations, 2, 1);
            verify!(ctxt.vertex_attrib_divisor(self.inst_def1.id(), 1));
            self.inst_def2
                .bind_sub_buffer(&mut instances.deformations, 2, 2);
            verify!(ctxt.vertex_attrib_divisor(self.inst_def2.id(), 1));

            verify!(ctxt.active_texture(Context::TEXTURE0));
            verify!(ctxt.bind_texture(Context::TEXTURE_2D, Some(&**data.texture())));

            if data.surface_rendering_active() {
                self.color.upload(data.color());

                if data.backface_culling_enabled() {
                    verify!(ctxt.enable(Context::CULL_FACE));
                } else {
                    verify!(ctxt.disable(Context::CULL_FACE));
                }

                let _ = verify!(ctxt.polygon_mode(Context::FRONT_AND_BACK, Context::FILL));
                verify!(ctxt.draw_elements_instanced(
                    Context::TRIANGLES,
                    mesh.num_pts() as i32,
                    VERTEX_INDEX_TYPE,
                    0,
                    instance_count
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
        verify!(ctxt.vertex_attrib_divisor(self.inst_def0.id(), 0));
        verify!(ctxt.vertex_attrib_divisor(self.inst_def1.id(), 0));
        verify!(ctxt.vertex_attrib_divisor(self.inst_def2.id(), 0));

        mesh.unbind();
        self.deactivate();
    }
}

/// Vertex shader of the default object material.
pub static OBJECT_VERTEX_SRC: &str = A_VERY_LONG_STRING;
/// Fragment shader of the default object material.
pub static OBJECT_FRAGMENT_SRC: &str = ANOTHER_VERY_LONG_STRING;

const A_VERY_LONG_STRING: &str = include_str!("default.vert");

// phong-like lighting (heavily) inspired
// http://www.mathematik.uni-marburg.de/~thormae/lectures/graphics1/code/WebGLShaderLightMat/ShaderLightMat.html
const ANOTHER_VERY_LONG_STRING: &str = include_str!("default.frag");
