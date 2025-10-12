use crate::camera::Camera;
use crate::context::Context;
use crate::light::Light;
use crate::resource::Material;
use crate::resource::{Effect, Mesh, ShaderAttribute, ShaderUniform};
use crate::scene::{InstancesBuffer, ObjectData};
use crate::verify;
use na::{Isometry3, Matrix3, Matrix4, Point2, Point3, Vector3};

/// A material that draws normals of an object.
pub struct UvsMaterial {
    shader: Effect,
    position: ShaderAttribute<Point3<f32>>,
    uvs: ShaderAttribute<Point2<f32>>,
    view: ShaderUniform<Matrix4<f32>>,
    proj: ShaderUniform<Matrix4<f32>>,
    transform: ShaderUniform<Matrix4<f32>>,
    scale: ShaderUniform<Matrix3<f32>>,
}

impl Default for UvsMaterial {
    fn default() -> Self {
        Self::new()
    }
}

impl UvsMaterial {
    /// Creates a new UvsMaterial.
    pub fn new() -> UvsMaterial {
        let mut shader = Effect::new_from_str(UVS_VERTEX_SRC, UVS_FRAGMENT_SRC);

        shader.use_program();

        UvsMaterial {
            position: shader.get_attrib("position").unwrap(),
            uvs: shader.get_attrib("uvs").unwrap(),
            transform: shader.get_uniform("transform").unwrap(),
            scale: shader.get_uniform("scale").unwrap(),
            view: shader.get_uniform("view").unwrap(),
            proj: shader.get_uniform("proj").unwrap(),
            shader,
        }
    }
}

impl Material for UvsMaterial {
    fn render(
        &mut self,
        pass: usize,
        transform: &Isometry3<f32>,
        scale: &Vector3<f32>,
        camera: &mut dyn Camera,
        _: &Light,
        data: &ObjectData,
        _instances: &mut InstancesBuffer,
        mesh: &mut Mesh,
    ) {
        if !data.surface_rendering_active() {
            return;
        }

        let ctxt = Context::get();
        // enable/disable culling.
        if data.backface_culling_enabled() {
            verify!(ctxt.enable(Context::CULL_FACE));
        } else {
            verify!(ctxt.disable(Context::CULL_FACE));
        }

        self.shader.use_program();
        self.position.enable();
        self.uvs.enable();

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
        let formatted_transform = transform.to_homogeneous();
        let formatted_scale = Matrix3::from_diagonal(&Vector3::new(scale.x, scale.y, scale.z));

        self.transform.upload(&formatted_transform);
        self.scale.upload(&formatted_scale);

        mesh.bind_coords(&mut self.position);
        mesh.bind_uvs(&mut self.uvs);
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
        self.uvs.disable();
    }
}

/// WGSL shader for UVs visualization
pub static UVS_WGSL_SRC: &str = include_str!("uvs.wgsl");

/// A vertex shader for coloring each point of an object depending on its texture coordinates.
pub static UVS_VERTEX_SRC: &str = UVS_WGSL_SRC;

/// A fragment shader for coloring each point of an object depending on its texture coordinates.
pub static UVS_FRAGMENT_SRC: &str = UVS_WGSL_SRC;
