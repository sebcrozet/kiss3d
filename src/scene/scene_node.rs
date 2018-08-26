use camera::Camera;
use light::Light;
use na;
use na::{Isometry3, Point2, Point3, Translation3, UnitQuaternion, Vector3};
use ncollide3d::procedural;
use ncollide3d::procedural::TriMesh;
use resource::{Material, MaterialManager, Mesh, MeshManager, Texture, TextureManager};
use scene::Object;
use std::cell::{Ref, RefCell, RefMut};
use std::mem;
use std::path::{Path, PathBuf};
use std::rc::Rc;

// XXX: once something like `fn foo(self: Rc<RefCell<SceneNode>>)` is allowed, this extra struct
// will not be needed any more.
/// The datas contained by a `SceneNode`.
pub struct SceneNodeData {
    local_scale: Vector3<f32>,
    local_transform: Isometry3<f32>,
    world_scale: Vector3<f32>,
    world_transform: Isometry3<f32>,
    visible: bool,
    up_to_date: bool,
    children: Vec<SceneNode>,
    object: Option<Object>,
    // FIXME: use Weak pointers instead of the raw pointer.
    parent: Option<*const RefCell<SceneNodeData>>,
}

/// A node of the scene graph.
///
/// This may represent a group of other nodes, and/or contain an object that can be rendered.
#[derive(Clone)]
pub struct SceneNode {
    data: Rc<RefCell<SceneNodeData>>,
}

impl SceneNodeData {
    // XXX: Because `node.borrow_mut().parent = Some(self.data.downgrade())`
    // causes a weird compiler error:
    //
    // ```
    // error: mismatched types: expected `&std::cell::RefCell<scene::scene_node::SceneNodeData>`
    // but found
    // `std::option::Option<std::rc::Weak<std::cell::RefCell<scene::scene_node::SceneNodeData>>>`
    // (expe cted &-ptr but found enum std::option::Option)
    // ```
    fn set_parent(&mut self, parent: *const RefCell<SceneNodeData>) {
        self.parent = Some(parent);
    }

    // XXX: this exists because of a similar bug as `set_parent`.
    fn remove_from_parent(&mut self, to_remove: &SceneNode) {
        let _ = self.parent.as_ref().map(|p| unsafe {
            let mut bp = (**p).borrow_mut();
            bp.remove(to_remove)
        });
    }

    fn remove(&mut self, o: &SceneNode) {
        match self.children.iter().rposition(|e| {
            &*o.data as *const RefCell<SceneNodeData> as usize
                == &*e.data as *const RefCell<SceneNodeData> as usize
        }) {
            Some(i) => {
                let _ = self.children.swap_remove(i);
            }
            None => {}
        }
    }

    /// Whether this node contains an `Object`.
    #[inline]
    pub fn has_object(&self) -> bool {
        self.object.is_some()
    }

    /// Whether this node has no parent.
    #[inline]
    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }

    /// Render the scene graph rooted by this node.
    pub fn render(&mut self, pass: usize, camera: &mut Camera, light: &Light) {
        if self.visible {
            self.do_render(&na::one(), &Vector3::from_element(1.0), pass, camera, light)
        }
    }

    fn do_render(
        &mut self,
        transform: &Isometry3<f32>,
        scale: &Vector3<f32>,
        pass: usize,
        camera: &mut Camera,
        light: &Light,
    ) {
        if !self.up_to_date {
            self.up_to_date = true;
            self.world_transform = *transform * self.local_transform;
            self.world_scale = scale.component_mul(&self.local_scale);
        }

        match self.object {
            Some(ref o) => o.render(
                &self.world_transform,
                &self.world_scale,
                pass,
                camera,
                light,
            ),
            None => {}
        }

        for c in self.children.iter_mut() {
            let mut bc = c.data_mut();
            if bc.visible {
                bc.do_render(
                    &self.world_transform,
                    &self.world_scale,
                    pass,
                    camera,
                    light,
                )
            }
        }
    }

    /// A reference to the object possibly contained by this node.
    #[inline]
    pub fn object(&self) -> Option<&Object> {
        self.object.as_ref()
    }

    /// A mutable reference to the object possibly contained by this node.
    #[inline]
    pub fn object_mut(&mut self) -> Option<&mut Object> {
        self.object.as_mut()
    }

    /// A reference to the object possibly contained by this node.
    ///
    /// # Failure
    /// Fails of this node does not contains an object.
    #[inline]
    pub fn get_object(&self) -> &Object {
        self.object()
            .expect("This scene node does not contain an Object.")
    }

    /// A mutable reference to the object possibly contained by this node.
    ///
    /// # Failure
    /// Fails of this node does not contains an object.
    #[inline]
    pub fn get_object_mut(&mut self) -> &mut Object {
        self.object_mut()
            .expect("This scene node does not contain an Object.")
    }

    ///////////////////~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~ HERE
    /* FIXME: the ~Any is kind of problematic here…
    /// Attaches user-defined data to the objects contained by this node and its children.
    #[inline]
    pub fn set_user_data(&mut self, user_data: ~Any) {
        self.apply_to_objects_mut(&mut |o| o.set_user_data(user_data))
    }
    */

    // FIXME: for all those set_stuff, would it be more per formant to add a special case for when
    // we are on a leaf? (to avoid the call to a closure required by the apply_to_*).
    /// Sets the material of the objects contained by this node and its children.
    #[inline]
    pub fn set_material(&mut self, material: Rc<RefCell<Box<Material + 'static>>>) {
        self.apply_to_objects_mut(&mut |o| o.set_material(material.clone()))
    }

    /// Sets the material of the objects contained by this node and its children.
    ///
    /// The material must already have been registered as `name`.
    #[inline]
    pub fn set_material_with_name(&mut self, name: &str) {
        let material = MaterialManager::get_global_manager(|tm| {
            tm.get(name).unwrap_or_else(|| {
                panic!("Invalid attempt to use the unregistered material: {}", name)
            })
        });

        self.set_material(material)
    }

    /// Sets the width of the lines drawn for the objects contained by this node and its children.
    #[inline]
    pub fn set_lines_width(&mut self, width: f32) {
        self.apply_to_objects_mut(&mut |o| o.set_lines_width(width))
    }

    /// Sets the size of the points drawn for the objects contained by this node and its children.
    #[inline]
    pub fn set_points_size(&mut self, size: f32) {
        self.apply_to_objects_mut(&mut |o| o.set_points_size(size))
    }

    /// Activates or deactivates the rendering of the surfaces of the objects contained by this node and its
    /// children.
    #[inline]
    pub fn set_surface_rendering_activation(&mut self, active: bool) {
        self.apply_to_objects_mut(&mut |o| o.set_surface_rendering_activation(active))
    }

    /// Activates or deactivates backface culling for the objects contained by this node and its
    /// children.
    #[inline]
    pub fn enable_backface_culling(&mut self, active: bool) {
        self.apply_to_objects_mut(&mut |o| o.enable_backface_culling(active))
    }

    /// Mutably accesses the vertices of the objects contained by this node and its children.
    ///
    /// The provided closure is called once per object.
    #[inline(always)]
    pub fn modify_vertices<F: FnMut(&mut Vec<Point3<f32>>)>(&mut self, f: &mut F) {
        self.apply_to_objects_mut(&mut |o| o.modify_vertices(f))
    }

    /// Accesses the vertices of the objects contained by this node and its children.
    ///
    /// The provided closure is called once per object.
    #[inline(always)]
    pub fn read_vertices<F: FnMut(&[Point3<f32>])>(&self, f: &mut F) {
        self.apply_to_objects(&mut |o| o.read_vertices(f))
    }

    /// Recomputes the normals of the meshes of the objects contained by this node and its
    /// children.
    #[inline]
    pub fn recompute_normals(&mut self) {
        self.apply_to_objects_mut(&mut |o| o.recompute_normals())
    }

    /// Mutably accesses the normals of the objects contained by this node and its children.
    ///
    /// The provided closure is called once per object.
    #[inline(always)]
    pub fn modify_normals<F: FnMut(&mut Vec<Vector3<f32>>)>(&mut self, f: &mut F) {
        self.apply_to_objects_mut(&mut |o| o.modify_normals(f))
    }

    /// Accesses the normals of the objects contained by this node and its children.
    ///
    /// The provided closure is called once per object.
    #[inline(always)]
    pub fn read_normals<F: FnMut(&[Vector3<f32>])>(&self, f: &mut F) {
        self.apply_to_objects(&mut |o| o.read_normals(f))
    }

    /// Mutably accesses the faces of the objects contained by this node and its children.
    ///
    /// The provided closure is called once per object.
    #[inline(always)]
    pub fn modify_faces<F: FnMut(&mut Vec<Point3<u16>>)>(&mut self, f: &mut F) {
        self.apply_to_objects_mut(&mut |o| o.modify_faces(f))
    }

    /// Accesses the faces of the objects contained by this node and its children.
    ///
    /// The provided closure is called once per object.
    #[inline(always)]
    pub fn read_faces<F: FnMut(&[Point3<u16>])>(&self, f: &mut F) {
        self.apply_to_objects(&mut |o| o.read_faces(f))
    }

    /// Mutably accesses the texture coordinates of the objects contained by this node and its
    /// children.
    ///
    /// The provided closure is called once per object.
    #[inline(always)]
    pub fn modify_uvs<F: FnMut(&mut Vec<Point2<f32>>)>(&mut self, f: &mut F) {
        self.apply_to_objects_mut(&mut |o| o.modify_uvs(f))
    }

    /// Accesses the texture coordinates of the objects contained by this node and its children.
    ///
    /// The provided closure is called once per object.
    #[inline(always)]
    pub fn read_uvs<F: FnMut(&[Point2<f32>])>(&self, f: &mut F) {
        self.apply_to_objects(&mut |o| o.read_uvs(f))
    }

    /// Get the visibility status of node.
    #[inline]
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Sets the visibility of this node.
    ///
    /// The node and its children are not rendered if it is not visible.
    #[inline]
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Sets the color of the objects contained by this node and its children.
    ///
    /// Colors components must be on the range `[0.0, 1.0]`.
    #[inline]
    pub fn set_color(&mut self, r: f32, g: f32, b: f32) {
        self.apply_to_objects_mut(&mut |o| o.set_color(r, g, b))
    }

    /// Sets the texture of the objects contained by this node and its children.
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

    /// Sets the texture of the objects contained by this node and its children.
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

    /// Sets the texture of the objects contained by this node and its children.
    pub fn set_texture(&mut self, texture: Rc<Texture>) {
        self.apply_to_objects_mut(&mut |o| o.set_texture(texture.clone()))
    }

    /// Applies a closure to each object contained by this node and its children.
    #[inline]
    pub fn apply_to_objects_mut<F: FnMut(&mut Object)>(&mut self, f: &mut F) {
        match self.object {
            Some(ref mut o) => f(o),
            None => {}
        }

        for c in self.children.iter_mut() {
            c.data_mut().apply_to_objects_mut(f)
        }
    }

    /// Applies a closure to each object contained by this node and its children.
    #[inline]
    pub fn apply_to_objects<F: FnMut(&Object)>(&self, f: &mut F) {
        match self.object {
            Some(ref o) => f(o),
            None => {}
        }

        for c in self.children.iter() {
            c.data().apply_to_objects(f)
        }
    }

    // FIXME: add folding?

    /// Sets the local scaling factors of the object.
    #[inline]
    pub fn set_local_scale(&mut self, sx: f32, sy: f32, sz: f32) {
        self.invalidate();
        self.local_scale = Vector3::new(sx, sy, sz)
    }

    /// Returns the scaling factors of the object.
    #[inline]
    pub fn local_scale(&self) -> Vector3<f32> {
        self.local_scale
    }

    /// Move and orient the object such that it is placed at the point `eye` and have its `z` axis
    /// oriented toward `at`.
    #[inline]
    pub fn reorient(&mut self, eye: &Point3<f32>, at: &Point3<f32>, up: &Vector3<f32>) {
        self.invalidate();
        // FIXME: multiply by the parent's world transform?
        self.local_transform = Isometry3::new_observer_frame(eye, at, up)
    }

    /// This node local transformation.
    #[inline]
    pub fn local_transformation(&self) -> Isometry3<f32> {
        self.local_transform.clone()
    }

    /// Inverse of this node local transformation.
    #[inline]
    pub fn inverse_local_transformation(&self) -> Isometry3<f32> {
        self.local_transform.inverse()
    }

    /// This node world transformation.
    ///
    /// This will force an update of the world transformation of its parents if they have been
    /// invalidated.
    #[inline]
    #[allow(mutable_transmutes)]
    pub fn world_transformation(&self) -> Isometry3<f32> {
        // NOTE: this is to have some kind of laziness without a `&mut self`.
        unsafe {
            let mself: &mut SceneNodeData = mem::transmute(self);
            mself.update();
        }
        self.world_transform.clone()
    }

    /// The inverse of this node world transformation.
    ///
    /// This will force an update of the world transformation of its parents if they have been
    /// invalidated.
    #[inline]
    #[allow(mutable_transmutes)]
    pub fn inverse_world_transformation(&self) -> Isometry3<f32> {
        // NOTE: this is to have some kind of laziness without a `&mut self`.
        unsafe {
            let mself: &mut SceneNodeData = mem::transmute(self);
            mself.update();
        }
        self.local_transform.inverse()
    }

    /// Appends a transformation to this node local transformation.
    #[inline]
    pub fn append_transformation(&mut self, t: &Isometry3<f32>) {
        self.invalidate();
        self.local_transform = t * self.local_transform
    }

    /// Prepends a transformation to this node local transformation.
    #[inline]
    pub fn prepend_to_local_transformation(&mut self, t: &Isometry3<f32>) {
        self.invalidate();
        self.local_transform *= t;
    }

    /// Set this node local transformation.
    #[inline]
    pub fn set_local_transformation(&mut self, t: Isometry3<f32>) {
        self.invalidate();
        self.local_transform = t
    }

    /// This node local translation.
    #[inline]
    pub fn local_translation(&self) -> Translation3<f32> {
        self.local_transform.translation
    }

    /// The inverse of this node local translation.
    #[inline]
    pub fn inverse_local_translation(&self) -> Translation3<f32> {
        self.local_transform.translation.inverse()
    }

    /// Appends a translation to this node local transformation.
    #[inline]
    pub fn append_translation(&mut self, t: &Translation3<f32>) {
        self.invalidate();
        self.local_transform = t * self.local_transform
    }

    /// Prepends a translation to this node local transformation.
    #[inline]
    pub fn prepend_to_local_translation(&mut self, t: &Translation3<f32>) {
        self.invalidate();
        self.local_transform *= t
    }

    /// Sets the local translation of this node.
    #[inline]
    pub fn set_local_translation(&mut self, t: Translation3<f32>) {
        self.invalidate();
        self.local_transform.translation = t
    }

    /// This node local rotation.
    #[inline]
    pub fn local_rotation(&self) -> UnitQuaternion<f32> {
        self.local_transform.rotation
    }

    /// The inverse of this node local rotation.
    #[inline]
    pub fn inverse_local_rotation(&self) -> UnitQuaternion<f32> {
        self.local_transform.rotation.inverse()
    }

    /// Appends a rotation to this node local transformation.
    #[inline]
    pub fn append_rotation(&mut self, r: &UnitQuaternion<f32>) {
        self.invalidate();
        self.local_transform = r * self.local_transform
    }

    /// Appends a rotation to this node local transformation.
    #[inline]
    pub fn append_rotation_wrt_center(&mut self, r: &UnitQuaternion<f32>) {
        self.invalidate();
        self.local_transform.append_rotation_wrt_center_mut(r)
    }

    /// Prepends a rotation to this node local transformation.
    #[inline]
    pub fn prepend_to_local_rotation(&mut self, r: &UnitQuaternion<f32>) {
        self.invalidate();
        self.local_transform *= r
    }

    /// Sets the local rotation of this node.
    #[inline]
    pub fn set_local_rotation(&mut self, r: UnitQuaternion<f32>) {
        self.invalidate();
        self.local_transform.rotation = r
    }

    fn invalidate(&mut self) {
        self.up_to_date = false;

        for c in self.children.iter_mut() {
            let mut dm = c.data_mut();

            if dm.up_to_date {
                dm.invalidate()
            }
        }
    }

    // FIXME: make this public?
    fn update(&mut self) {
        // NOTE: makin this test
        if !self.up_to_date {
            match self.parent {
                Some(ref mut p) => unsafe {
                    let mut dp = (**p).borrow_mut();

                    dp.update();
                    self.world_transform = self.local_transform * dp.world_transform;
                    self.world_scale = self.local_scale.component_mul(&dp.local_scale);
                    self.up_to_date = true;
                    return;
                },
                None => {}
            }

            // no parent
            self.world_transform = self.local_transform;
            self.world_scale = self.local_scale;
            self.up_to_date = true;
        }
    }
}

impl SceneNode {
    /// Creates a new scene node that is not rooted.
    pub fn new(
        local_scale: Vector3<f32>,
        local_transform: Isometry3<f32>,
        object: Option<Object>,
    ) -> SceneNode {
        let data = SceneNodeData {
            local_scale: local_scale,
            local_transform: local_transform,
            world_transform: local_transform,
            world_scale: local_scale,
            visible: true,
            up_to_date: false,
            children: Vec::new(),
            object: object,
            parent: None,
        };

        SceneNode {
            data: Rc::new(RefCell::new(data)),
        }
    }

    /// Creates a new empty, not rooted, node with identity transformations.
    pub fn new_empty() -> SceneNode {
        SceneNode::new(Vector3::from_element(1.0), na::one(), None)
    }

    /// Removes this node from its parent.
    pub fn unlink(&mut self) {
        let self_self = self.clone();
        self.data_mut().remove_from_parent(&self_self);
        self.data_mut().parent = None
    }

    /// The data of this scene node.
    pub fn data(&self) -> Ref<SceneNodeData> {
        self.data.borrow()
    }

    /// The data of this scene node.
    pub fn data_mut(&mut self) -> RefMut<SceneNodeData> {
        self.data.borrow_mut()
    }

    /*
     *
     * Methods to add objects.
     *
     */
    /// Adds a node without object to this node children.
    pub fn add_group(&mut self) -> SceneNode {
        let node = SceneNode::new_empty();

        self.add_child(node.clone());

        node
    }

    /// Adds a node as a child of `parent`.
    ///
    /// # Failures:
    /// Fails if `node` already has a parent.
    pub fn add_child(&mut self, node: SceneNode) {
        assert!(
            node.data().is_root(),
            "The added node must not have a parent yet."
        );

        let mut node = node;
        node.data_mut().set_parent(&*self.data);
        self.data_mut().children.push(node)
    }

    /// Adds a node containing an object to this node children.
    pub fn add_object(
        &mut self,
        local_scale: Vector3<f32>,
        local_transform: Isometry3<f32>,
        object: Object,
    ) -> SceneNode {
        let node = SceneNode::new(local_scale, local_transform, Some(object));

        self.add_child(node.clone());

        node
    }

    /// Adds a cube as a children of this node. The cube is initially axis-aligned and centered
    /// at (0, 0, 0).
    ///
    /// # Arguments
    /// * `wx` - the cube extent along the x axis
    /// * `wy` - the cube extent along the y axis
    /// * `wz` - the cube extent along the z axis
    pub fn add_cube(&mut self, wx: f32, wy: f32, wz: f32) -> SceneNode {
        let res = self.add_geom_with_name("cube", Vector3::new(wx, wy, wz));

        res.expect("Unable to load the default cube geometry.")
    }

    /// Adds a sphere as a children of this node. The sphere is initially centered at (0, 0, 0).
    ///
    /// # Arguments
    /// * `r` - the sphere radius
    pub fn add_sphere(&mut self, r: f32) -> SceneNode {
        let res = self.add_geom_with_name("sphere", Vector3::new(r * 2.0, r * 2.0, r * 2.0));

        res.expect("Unable to load the default sphere geometry.")
    }

    /// Adds a cone to the scene. The cone is initially centered at (0, 0, 0) and points toward the
    /// positive `y` axis.
    ///
    /// # Arguments
    /// * `h` - the cone height
    /// * `r` - the cone base radius
    pub fn add_cone(&mut self, r: f32, h: f32) -> SceneNode {
        let res = self.add_geom_with_name("cone", Vector3::new(r * 2.0, h, r * 2.0));

        res.expect("Unable to load the default cone geometry.")
    }

    /// Adds a cylinder to this node children. The cylinder is initially centered at (0, 0, 0)
    /// and has its principal axis aligned with the `y` axis.
    ///
    /// # Arguments
    /// * `h` - the cylinder height
    /// * `r` - the cylinder base radius
    pub fn add_cylinder(&mut self, r: f32, h: f32) -> SceneNode {
        let res = self.add_geom_with_name("cylinder", Vector3::new(r * 2.0, h, r * 2.0));

        res.expect("Unable to load the default cylinder geometry.")
    }

    /// Adds a capsule to this node children. The capsule is initially centered at (0, 0, 0) and
    /// has its principal axis aligned with the `y` axis.
    ///
    /// # Arguments
    /// * `h` - the capsule height
    /// * `r` - the capsule caps radius
    pub fn add_capsule(&mut self, r: f32, h: f32) -> SceneNode {
        self.add_trimesh(
            procedural::capsule(&(r * 2.0), &h, 50, 50),
            Vector3::from_element(1.0),
        )
    }

    /// Adds a double-sided quad to this node children. The quad is initially centered at (0, 0,
    /// 0). The quad itself is composed of a user-defined number of triangles regularly spaced on a
    /// grid. This is the main way to draw height maps.
    ///
    /// # Arguments
    /// * `w` - the quad width.
    /// * `h` - the quad height.
    /// * `wsubdivs` - number of horizontal subdivisions. This correspond to the number of squares
    /// which will be placed horizontally on each line. Must not be `0`.
    /// * `hsubdivs` - number of vertical subdivisions. This correspond to the number of squares
    /// which will be placed vertically on each line. Must not be `0`.
    /// update.
    pub fn add_quad(&mut self, w: f32, h: f32, usubdivs: usize, vsubdivs: usize) -> SceneNode {
        let mut node = self.add_trimesh(
            procedural::quad(w, h, usubdivs, vsubdivs),
            Vector3::from_element(1.0),
        );
        node.enable_backface_culling(false);

        node
    }

    /// Adds a double-sided quad with the specified vertices.
    pub fn add_quad_with_vertices(
        &mut self,
        vertices: &[Point3<f32>],
        nhpoints: usize,
        nvpoints: usize,
    ) -> SceneNode {
        let geom = procedural::quad_with_vertices(vertices, nhpoints, nvpoints);

        let mut node = self.add_trimesh(geom, Vector3::from_element(1.0));
        node.enable_backface_culling(false);

        node
    }

    /// Creates and adds a new object using the geometry registered as `geometry_name`.
    pub fn add_geom_with_name(
        &mut self,
        geometry_name: &str,
        scale: Vector3<f32>,
    ) -> Option<SceneNode> {
        MeshManager::get_global_manager(|mm| mm.get(geometry_name)).map(|g| self.add_mesh(g, scale))
    }

    /// Creates and adds a new object to this node children using a mesh.
    pub fn add_mesh(&mut self, mesh: Rc<RefCell<Mesh>>, scale: Vector3<f32>) -> SceneNode {
        let tex = TextureManager::get_global_manager(|tm| tm.get_default());
        let mat = MaterialManager::get_global_manager(|mm| mm.get_default());
        let object = Object::new(mesh, 1.0, 1.0, 1.0, tex, mat);

        self.add_object(scale, na::one(), object)
    }

    /// Creates and adds a new object using a mesh descriptor.
    pub fn add_trimesh(&mut self, descr: TriMesh<f32>, scale: Vector3<f32>) -> SceneNode {
        self.add_mesh(
            Rc::new(RefCell::new(Mesh::from_trimesh(descr, false))),
            scale,
        )
    }

    /// Creates and adds multiple nodes created from an obj file.
    ///
    /// This will create a new node serving as a root of the scene described by the obj file. This
    /// newly created node is added to this node's children.
    pub fn add_obj(&mut self, path: &Path, mtl_dir: &Path, scale: Vector3<f32>) -> SceneNode {
        let tex = TextureManager::get_global_manager(|tm| tm.get_default());
        let mat = MaterialManager::get_global_manager(|mm| mm.get_default());

        // FIXME: is there some error-handling stuff to do here instead of the `let _`.
        let result = MeshManager::load_obj(path, mtl_dir, path.to_str().unwrap()).map(|objs| {
            let mut root;

            let self_root = objs.len() == 1;
            let child_scale;

            if self_root {
                root = self.clone();
                child_scale = scale;
            } else {
                root = SceneNode::new(scale, na::one(), None);
                self.add_child(root.clone());
                child_scale = Vector3::from_element(1.0);
            }

            for (_, mesh, mtl) in objs.into_iter() {
                let mut object = Object::new(mesh, 1.0, 1.0, 1.0, tex.clone(), mat.clone());

                match mtl {
                    None => {}
                    Some(mtl) => {
                        object.set_color(mtl.diffuse.x, mtl.diffuse.y, mtl.diffuse.z);

                        for t in mtl.diffuse_texture.iter() {
                            let mut tpath = PathBuf::new();
                            tpath.push(mtl_dir);
                            tpath.push(&t[..]);
                            object.set_texture_from_file(&tpath, tpath.to_str().unwrap())
                        }

                        for t in mtl.ambiant_texture.iter() {
                            let mut tpath = PathBuf::new();
                            tpath.push(mtl_dir);
                            tpath.push(&t[..]);
                            object.set_texture_from_file(&tpath, tpath.to_str().unwrap())
                        }
                    }
                }

                let _ = root.add_object(child_scale, na::one(), object);
            }

            if self_root {
                root.data()
                    .children
                    .last()
                    .expect("There was nothing on this obj file.")
                    .clone()
            } else {
                root
            }
        });

        result.unwrap()
    }

    /// Applies a closure to each object contained by this node and its children.
    #[inline]
    pub fn apply_to_scene_nodes_mut<F: FnMut(&mut SceneNode)>(&mut self, f: &mut F) {
        f(self);

        for c in self.data_mut().children.iter_mut() {
            c.apply_to_scene_nodes_mut(f)
        }
    }

    /// Applies a closure to each object contained by this node and its children.
    #[inline]
    pub fn apply_to_scene_nodes<F: FnMut(&SceneNode)>(&self, f: &mut F) {
        f(self);

        for c in self.data().children.iter() {
            c.apply_to_scene_nodes(f)
        }
    }

    //
    //
    // fwd
    //
    //

    /// Render the scene graph rooted by this node.
    pub fn render(&mut self, pass: usize, camera: &mut Camera, light: &Light) {
        self.data_mut().render(pass, camera, light)
    }

    /// Sets the material of the objects contained by this node and its children.
    #[inline]
    pub fn set_material(&mut self, material: Rc<RefCell<Box<Material + 'static>>>) {
        self.data_mut().set_material(material)
    }

    /// Sets the material of the objects contained by this node and its children.
    #[inline]
    pub fn set_material_with_name(&mut self, name: &str) {
        self.data_mut().set_material_with_name(name)
    }

    /// Sets the width of the lines drawn for the objects contained by this node and its children.
    #[inline]
    pub fn set_lines_width(&mut self, width: f32) {
        self.data_mut().set_lines_width(width)
    }

    /// Sets the size of the points drawn for the objects contained by this node and its children.
    #[inline]
    pub fn set_points_size(&mut self, size: f32) {
        self.data_mut().set_points_size(size)
    }

    /// Activates or deactivates the rendering of the surfaces of the objects contained by this node and its
    /// children.
    #[inline]
    pub fn set_surface_rendering_activation(&mut self, active: bool) {
        self.data_mut().set_surface_rendering_activation(active)
    }

    /// Activates or deactivates backface culling for the objects contained by this node and its
    /// children.
    #[inline]
    pub fn enable_backface_culling(&mut self, active: bool) {
        self.data_mut().enable_backface_culling(active)
    }

    /// Mutably accesses the vertices of the objects contained by this node and its children.
    ///
    /// The provided closure is called once per object.
    #[inline(always)]
    pub fn modify_vertices<F: FnMut(&mut Vec<Point3<f32>>)>(&mut self, f: &mut F) {
        self.data_mut().modify_vertices(f)
    }

    /// Accesses the vertices of the objects contained by this node and its children.
    ///
    /// The provided closure is called once per object.
    #[inline(always)]
    pub fn read_vertices<F: FnMut(&[Point3<f32>])>(&self, f: &mut F) {
        self.data().read_vertices(f)
    }

    /// Recomputes the normals of the meshes of the objects contained by this node and its
    /// children.
    #[inline]
    pub fn recompute_normals(&mut self) {
        self.data_mut().recompute_normals()
    }

    /// Mutably accesses the normals of the objects contained by this node and its children.
    ///
    /// The provided closure is called once per object.
    #[inline(always)]
    pub fn modify_normals<F: FnMut(&mut Vec<Vector3<f32>>)>(&mut self, f: &mut F) {
        self.data_mut().modify_normals(f)
    }

    /// Accesses the normals of the objects contained by this node and its children.
    ///
    /// The provided closure is called once per object.
    #[inline(always)]
    pub fn read_normals<F: FnMut(&[Vector3<f32>])>(&self, f: &mut F) {
        self.data().read_normals(f)
    }

    /// Mutably accesses the faces of the objects contained by this node and its children.
    ///
    /// The provided closure is called once per object.
    #[inline(always)]
    pub fn modify_faces<F: FnMut(&mut Vec<Point3<u16>>)>(&mut self, f: &mut F) {
        self.data_mut().modify_faces(f)
    }

    /// Accesses the faces of the objects contained by this node and its children.
    ///
    /// The provided closure is called once per object.
    #[inline(always)]
    pub fn read_faces<F: FnMut(&[Point3<u16>])>(&self, f: &mut F) {
        self.data().read_faces(f)
    }

    /// Mutably accesses the texture coordinates of the objects contained by this node and its
    /// children.
    ///
    /// The provided closure is called once per object.
    #[inline(always)]
    pub fn modify_uvs<F: FnMut(&mut Vec<Point2<f32>>)>(&mut self, f: &mut F) {
        self.data_mut().modify_uvs(f)
    }

    /// Accesses the texture coordinates of the objects contained by this node and its children.
    ///
    /// The provided closure is called once per object.
    #[inline(always)]
    pub fn read_uvs<F: FnMut(&[Point2<f32>])>(&self, f: &mut F) {
        self.data().read_uvs(f)
    }

    /// Get the visibility status of node.
    #[inline]
    pub fn is_visible(&self) -> bool {
        self.data().is_visible()
    }

    /// Sets the visibility of this node.
    ///
    /// The node and its children are not rendered if it is not visible.
    #[inline]
    pub fn set_visible(&mut self, visible: bool) {
        self.data_mut().set_visible(visible)
    }

    /// Sets the color of the objects contained by this node and its children.
    ///
    /// Colors components must be on the range `[0.0, 1.0]`.
    #[inline]
    pub fn set_color(&mut self, r: f32, g: f32, b: f32) {
        self.data_mut().set_color(r, g, b)
    }

    /// Sets the texture of the objects contained by this node and its children.
    ///
    /// The texture is loaded from a file and registered by the global `TextureManager`.
    ///
    /// # Arguments
    ///   * `path` - relative path of the texture on the disk
    #[inline]
    pub fn set_texture_from_file(&mut self, path: &Path, name: &str) {
        self.data_mut().set_texture_from_file(path, name)
    }

    /// Sets the texture of the objects contained by this node and its children.
    ///
    /// The texture must already have been registered as `name`.
    #[inline]
    pub fn set_texture_with_name(&mut self, name: &str) {
        self.data_mut().set_texture_with_name(name)
    }

    /// Sets the texture of the objects contained by this node and its children.
    pub fn set_texture(&mut self, texture: Rc<Texture>) {
        self.data_mut().set_texture(texture)
    }

    /// Sets the local scaling factors of the object.
    #[inline]
    pub fn set_local_scale(&mut self, sx: f32, sy: f32, sz: f32) {
        self.data_mut().set_local_scale(sx, sy, sz)
    }

    /// Move and orient the object such that it is placed at the point `eye` and have its `x` axis
    /// oriented toward `at`.
    #[inline]
    pub fn reorient(&mut self, eye: &Point3<f32>, at: &Point3<f32>, up: &Vector3<f32>) {
        self.data_mut().reorient(eye, at, up)
    }

    /// Appends a transformation to this node local transformation.
    #[inline]
    pub fn append_transformation(&mut self, t: &Isometry3<f32>) {
        self.data_mut().append_transformation(t)
    }

    /// Prepends a transformation to this node local transformation.
    #[inline]
    pub fn prepend_to_local_transformation(&mut self, t: &Isometry3<f32>) {
        self.data_mut().prepend_to_local_transformation(t)
    }

    /// Set this node local transformation.
    #[inline]
    pub fn set_local_transformation(&mut self, t: Isometry3<f32>) {
        self.data_mut().set_local_transformation(t)
    }

    /// Appends a translation to this node local transformation.
    #[inline]
    pub fn append_translation(&mut self, t: &Translation3<f32>) {
        self.data_mut().append_translation(t)
    }

    /// Prepends a translation to this node local transformation.
    #[inline]
    pub fn prepend_to_local_translation(&mut self, t: &Translation3<f32>) {
        self.data_mut().prepend_to_local_translation(t)
    }

    /// Sets the local translation of this node.
    #[inline]
    pub fn set_local_translation(&mut self, t: Translation3<f32>) {
        self.data_mut().set_local_translation(t)
    }

    /// Appends a rotation to this node local transformation.
    #[inline]
    pub fn append_rotation(&mut self, r: &UnitQuaternion<f32>) {
        self.data_mut().append_rotation(r)
    }

    /// Appends a rotation to this node local transformation.
    #[inline]
    pub fn append_rotation_wrt_center(&mut self, r: &UnitQuaternion<f32>) {
        (*self.data_mut()).append_rotation_wrt_center(r)
    }

    /// Prepends a rotation to this node local transformation.
    #[inline]
    pub fn prepend_to_local_rotation(&mut self, r: &UnitQuaternion<f32>) {
        self.data_mut().prepend_to_local_rotation(r)
    }

    /// Sets the local rotation of this node.
    #[inline]
    pub fn set_local_rotation(&mut self, r: UnitQuaternion<f32>) {
        self.data_mut().set_local_rotation(r)
    }
}
