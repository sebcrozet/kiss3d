//! Data structure of a scene node.

use crate::planar_camera::PlanarCamera;
use crate::resource::vertex_index::VertexIndex;
use crate::resource::{AllocationType, BufferType, GPUVec, PlanarMaterial, PlanarMesh, Texture, TextureManager};
use na::{Isometry2, Matrix2, Point2, Point3, Vector2};
use std::any::Any;
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

/// Set of data identifying a scene node.
pub struct PlanarObjectData {
    material: Rc<RefCell<Box<dyn PlanarMaterial + 'static>>>,
    texture: Rc<Texture>,
    color: Point3<f32>,
    lines_color: Option<Point3<f32>>,
    wlines: f32,
    wpoints: f32,
    draw_surface: bool,
    cull: bool,
    user_data: Box<dyn Any + 'static>,
}

impl PlanarObjectData {
    /// The texture of this object.
    #[inline]
    pub fn texture(&self) -> &Rc<Texture> {
        &self.texture
    }

    /// The color of this object.
    #[inline]
    pub fn color(&self) -> &Point3<f32> {
        &self.color
    }

    /// The width of the lines draw for this object.
    #[inline]
    pub fn lines_width(&self) -> f32 {
        self.wlines
    }

    /// The color of the lines draw for this object.
    #[inline]
    pub fn lines_color(&self) -> Option<&Point3<f32>> {
        self.lines_color.as_ref()
    }

    /// The size of the points draw for this object.
    #[inline]
    pub fn points_size(&self) -> f32 {
        self.wpoints
    }

    /// Whether this object has its surface rendered or not.
    #[inline]
    pub fn surface_rendering_active(&self) -> bool {
        self.draw_surface
    }

    /// Whether this object uses backface culling or not.
    #[inline]
    pub fn backface_culling_enabled(&self) -> bool {
        self.cull
    }

    /// An user-defined data.
    ///
    /// Use dynamic typing capabilities of the `Any` type to recover the actual data.
    #[inline]
    pub fn user_data(&self) -> &dyn Any {
        &*self.user_data
    }
}

pub struct PlanarInstanceData {
    pub position: Point2<f32>,
    pub deformation: Matrix2<f32>,
    pub color: [f32; 4],
}

impl Default for PlanarInstanceData {
    fn default() -> Self {
        Self {
            position: Point2::origin(),
            deformation: Matrix2::identity(),
            color: [1.0; 4],
        }
    }
}

pub struct PlanarInstancesBuffers {
    pub positions: GPUVec<Point2<f32>>,
    pub deformations: GPUVec<Matrix2<f32>>,
    pub colors: GPUVec<[f32; 4]>,
    // TODO: add other properties we want compatible with instancing.
    //       (like rotations, color, or a full 4x4 matrix).
}

impl Default for PlanarInstancesBuffers {
    fn default() -> Self {
        PlanarInstancesBuffers {
            positions: GPUVec::new(vec![Point2::origin()], BufferType::Array, AllocationType::StreamDraw),
            deformations: GPUVec::new(vec![Matrix2::identity()], BufferType::Array, AllocationType::StreamDraw),
            colors: GPUVec::new(vec![[1.0; 4]], BufferType::Array, AllocationType::StreamDraw),
        }
    }
}

impl PlanarInstancesBuffers {
    pub fn len(&self) -> usize {
        self.positions.len()
    }
}

/// A 3d objects on the scene.
///
/// This is the only interface to manipulate the object position, color, vertices and texture.
pub struct PlanarObject {
    // FIXME: should PlanarMesh and PlanarObject be merged?
    // (thus removing the need of PlanarObjectData at all.)
    data: PlanarObjectData,
    instances: Rc<RefCell<PlanarInstancesBuffers>>,
    mesh: Rc<RefCell<PlanarMesh>>,
}

impl PlanarObject {
    #[doc(hidden)]
    pub fn new(
        mesh: Rc<RefCell<PlanarMesh>>,
        r: f32,
        g: f32,
        b: f32,
        texture: Rc<Texture>,
        material: Rc<RefCell<Box<dyn PlanarMaterial + 'static>>>,
    ) -> PlanarObject {
        let user_data = ();
        let data = PlanarObjectData {
            color: Point3::new(r, g, b),
            lines_color: None,
            texture,
            wlines: 0.0,
            wpoints: 0.0,
            draw_surface: true,
            cull: true,
            material,
            user_data: Box::new(user_data),
        };
        let instances = Rc::new(RefCell::new(PlanarInstancesBuffers::default()));

        PlanarObject { data, instances, mesh }
    }

    #[doc(hidden)]
    pub fn render(
        &self,
        transform: &Isometry2<f32>,
        scale: &Vector2<f32>,
        camera: &mut dyn PlanarCamera,
    ) {
        self.data.material.borrow_mut().render(
            transform,
            scale,
            camera,
            &self.data,
            &mut *self.instances.borrow_mut(),
            &mut *self.mesh.borrow_mut(),
        );
    }

    /// Gets the data of this object.
    #[inline]
    pub fn data(&self) -> &PlanarObjectData {
        &self.data
    }

    /// Gets the data of this object.
    #[inline]
    pub fn data_mut(&mut self) -> &mut PlanarObjectData {
        &mut self.data
    }

    /// Gets the instances of this object.
    #[inline]
    pub fn instances(&self) -> &Rc<RefCell<PlanarInstancesBuffers>> {
        &self.instances
    }

    pub fn set_instances(&mut self, instances: &[PlanarInstanceData]) {
        let mut pos_data: Vec<_> = self.instances.borrow_mut().positions.data_mut().take().unwrap_or_default();
        let mut col_data: Vec<_> = self.instances.borrow_mut().colors.data_mut().take().unwrap_or_default();
        let mut def_data: Vec<_> = self.instances.borrow_mut().deformations.data_mut().take().unwrap_or_default();

        pos_data.clear();
        col_data.clear();
        def_data.clear();

        pos_data.extend(instances.iter().map(|i| i.position));
        col_data.extend(instances.iter().map(|i| i.color));
        def_data.extend(instances.iter().map(|i| i.deformation));


        *self.instances.borrow_mut().positions.data_mut() = Some(pos_data);
        *self.instances.borrow_mut().colors.data_mut() = Some(col_data);
        *self.instances.borrow_mut().deformations.data_mut() = Some(def_data);
    }

    /// Enables or disables backface culling for this object.
    #[inline]
    pub fn enable_backface_culling(&mut self, active: bool) {
        self.data.cull = active;
    }

    /// Attaches user-defined data to this object.
    #[inline]
    pub fn set_user_data(&mut self, user_data: Box<dyn Any + 'static>) {
        self.data.user_data = user_data;
    }

    /// Gets the material of this object.
    #[inline]
    pub fn material(&self) -> Rc<RefCell<Box<dyn PlanarMaterial + 'static>>> {
        self.data.material.clone()
    }

    /// Sets the material of this object.
    #[inline]
    pub fn set_material(&mut self, material: Rc<RefCell<Box<dyn PlanarMaterial + 'static>>>) {
        self.data.material = material;
    }

    /// Sets the width of the lines drawn for this object.
    #[inline]
    pub fn set_lines_width(&mut self, width: f32) {
        self.data.wlines = width
    }

    /// Returns the width of the lines drawn for this object.
    #[inline]
    pub fn lines_width(&self) -> f32 {
        self.data.wlines
    }

    /// Sets the color of the lines drawn for this object.
    #[inline]
    pub fn set_lines_color(&mut self, color: Option<Point3<f32>>) {
        self.data.lines_color = color
    }

    /// Returns the color of the lines drawn for this object.
    #[inline]
    pub fn lines_color(&self) -> Option<&Point3<f32>> {
        self.data.lines_color()
    }

    /// Sets the size of the points drawn for this object.
    #[inline]
    pub fn set_points_size(&mut self, size: f32) {
        self.data.wpoints = size
    }

    /// Returns the size of the points drawn for this object.
    #[inline]
    pub fn points_size(&self) -> f32 {
        self.data.wpoints
    }

    /// Activate or deactivate the rendering of this object surface.
    #[inline]
    pub fn set_surface_rendering_activation(&mut self, active: bool) {
        self.data.draw_surface = active
    }

    /// Activate or deactivate the rendering of this object surface.
    #[inline]
    pub fn surface_rendering_activation(&self) -> bool {
        self.data.draw_surface
    }

    /// This object's mesh.
    #[inline]
    pub fn mesh(&self) -> &Rc<RefCell<PlanarMesh>> {
        &self.mesh
    }

    /// Mutably access the object's vertices.
    #[inline(always)]
    pub fn modify_vertices<F: FnMut(&mut Vec<Point2<f32>>)>(&mut self, f: &mut F) {
        let bmesh = self.mesh.borrow_mut();
        let _ = bmesh
            .coords()
            .write()
            .unwrap()
            .data_mut()
            .as_mut()
            .map(|coords| f(coords));
    }

    /// Access the object's vertices.
    #[inline(always)]
    pub fn read_vertices<F: FnMut(&[Point2<f32>])>(&self, f: &mut F) {
        let bmesh = self.mesh.borrow();
        let _ = bmesh
            .coords()
            .read()
            .unwrap()
            .data()
            .as_ref()
            .map(|coords| f(&coords[..]));
    }

    /// Mutably access the object's faces.
    #[inline(always)]
    pub fn modify_faces<F: FnMut(&mut Vec<Point3<VertexIndex>>)>(&mut self, f: &mut F) {
        let bmesh = self.mesh.borrow_mut();
        let _ = bmesh
            .faces()
            .write()
            .unwrap()
            .data_mut()
            .as_mut()
            .map(|faces| f(faces));
    }

    /// Access the object's faces.
    #[inline(always)]
    pub fn read_faces<F: FnMut(&[Point3<VertexIndex>])>(&self, f: &mut F) {
        let bmesh = self.mesh.borrow();
        let _ = bmesh
            .faces()
            .read()
            .unwrap()
            .data()
            .as_ref()
            .map(|faces| f(&faces[..]));
    }

    /// Mutably access the object's texture coordinates.
    #[inline(always)]
    pub fn modify_uvs<F: FnMut(&mut Vec<Point2<f32>>)>(&mut self, f: &mut F) {
        let bmesh = self.mesh.borrow_mut();
        let _ = bmesh
            .uvs()
            .write()
            .unwrap()
            .data_mut()
            .as_mut()
            .map(|uvs| f(uvs));
    }

    /// Access the object's texture coordinates.
    #[inline(always)]
    pub fn read_uvs<F: FnMut(&[Point2<f32>])>(&self, f: &mut F) {
        let bmesh = self.mesh.borrow();
        let _ = bmesh
            .uvs()
            .read()
            .unwrap()
            .data()
            .as_ref()
            .map(|uvs| f(&uvs[..]));
    }

    /// Sets the color of the object.
    ///
    /// Colors components must be on the range `[0.0, 1.0]`.
    #[inline]
    pub fn set_color(&mut self, r: f32, g: f32, b: f32) {
        self.data.color.x = r;
        self.data.color.y = g;
        self.data.color.z = b;
    }

    /// Sets the texture of the object.
    ///
    /// The texture is loaded from a file and registered by the global `TextureManager`.
    ///
    /// # Arguments
    ///   * `path` - relative path of the texture on the disk
    #[inline]
    pub fn set_texture_from_file(&mut self, path: &Path, name: &str) {
        let texture = TextureManager::get_global_manager(|tm| tm.add(path, name));

        self.set_texture(texture)
    }

    /// Sets the texture of the object.
    ///
    /// The texture must already have been registered as `name`.
    #[inline]
    pub fn set_texture_with_name(&mut self, name: &str) {
        let texture = TextureManager::get_global_manager(|tm| {
            tm.get(name).unwrap_or_else(|| {
                panic!("Invalid attempt to use the unregistered texture: {}", name)
            })
        });

        self.set_texture(texture)
    }

    /// Sets the texture of the object.
    #[inline]
    pub fn set_texture(&mut self, texture: Rc<Texture>) {
        self.data.texture = texture
    }
}
