//! Data structure of a scene node geometry.
use std::iter;
use std::sync::{Arc, RwLock};

use na::{Point2, Point3};
use resource::gpu_vector::{AllocationType, BufferType, GPUVec};
use resource::ShaderAttribute;

#[path = "../error.rs"]
mod error;

/// Aggregation of vertices, indices, normals and texture coordinates.
///
/// It also contains the GPU location of those buffers.
pub struct PlanarMesh {
    coords: Arc<RwLock<GPUVec<Point2<f32>>>>,
    faces: Arc<RwLock<GPUVec<Point3<u16>>>>,
    uvs: Arc<RwLock<GPUVec<Point2<f32>>>>,
    edges: Option<Arc<RwLock<GPUVec<Point2<u16>>>>>,
}

impl PlanarMesh {
    /// Creates a new mesh.
    ///
    /// If the normals and uvs are not given, they are automatically computed.
    pub fn new(
        coords: Vec<Point2<f32>>,
        faces: Vec<Point3<u16>>,
        uvs: Option<Vec<Point2<f32>>>,
        dynamic_draw: bool,
    ) -> PlanarMesh {
        let uvs = match uvs {
            Some(us) => us,
            None => iter::repeat(Point2::origin()).take(coords.len()).collect(),
        };

        let location = if dynamic_draw {
            AllocationType::DynamicDraw
        } else {
            AllocationType::StaticDraw
        };
        let cs = Arc::new(RwLock::new(GPUVec::new(
            coords,
            BufferType::Array,
            location,
        )));
        let fs = Arc::new(RwLock::new(GPUVec::new(
            faces,
            BufferType::ElementArray,
            location,
        )));
        let us = Arc::new(RwLock::new(GPUVec::new(uvs, BufferType::Array, location)));

        PlanarMesh::new_with_gpu_vectors(cs, fs, us)
    }

    /// Creates a new mesh. Arguments set to `None` are automatically computed.
    pub fn new_with_gpu_vectors(
        coords: Arc<RwLock<GPUVec<Point2<f32>>>>,
        faces: Arc<RwLock<GPUVec<Point3<u16>>>>,
        uvs: Arc<RwLock<GPUVec<Point2<f32>>>>,
    ) -> PlanarMesh {
        PlanarMesh {
            coords: coords,
            faces: faces,
            uvs: uvs,
            edges: None,
        }
    }

    /// Binds this mesh vertex coordinates buffer to a vertex attribute.
    pub fn bind_coords(&mut self, coords: &mut ShaderAttribute<Point2<f32>>) {
        coords.bind(&mut *self.coords.write().unwrap());
    }

    /// Binds this mesh vertex uvs buffer to a vertex attribute.
    pub fn bind_uvs(&mut self, uvs: &mut ShaderAttribute<Point2<f32>>) {
        uvs.bind(&mut *self.uvs.write().unwrap());
    }

    /// Binds this mesh vertex uvs buffer to a vertex attribute.
    pub fn bind_faces(&mut self) {
        self.faces.write().unwrap().bind();
    }

    /// Binds this mesh buffers to vertex attributes.
    pub fn bind(
        &mut self,
        coords: &mut ShaderAttribute<Point2<f32>>,
        uvs: &mut ShaderAttribute<Point2<f32>>,
    ) {
        self.bind_coords(coords);
        self.bind_uvs(uvs);
        self.bind_faces();
    }

    /// Binds this mesh buffers to vertex attributes.
    pub fn bind_edges(&mut self) {
        if self.edges.is_none() {
            // FIXME: remove internal edges.
            let mut edges = Vec::new();
            for face in self.faces.read().unwrap().data().as_ref().unwrap() {
                edges.push(Point2::new(face.x, face.y));
                edges.push(Point2::new(face.y, face.z));
                edges.push(Point2::new(face.z, face.x));
            }
            let gpu_edges =
                GPUVec::new(edges, BufferType::ElementArray, AllocationType::StaticDraw);
            self.edges = Some(Arc::new(RwLock::new(gpu_edges)));
        }

        self.edges.as_mut().unwrap().write().unwrap().bind();
    }

    /// Unbind this mesh buffers to vertex attributes.
    pub fn unbind(&self) {
        self.coords.write().unwrap().unbind();
        self.uvs.write().unwrap().unbind();
        self.faces.write().unwrap().unbind();
    }

    /// Number of points needed to draw this mesh.
    pub fn num_pts(&self) -> usize {
        self.faces.read().unwrap().len() * 3
    }

    /// This mesh faces.
    pub fn faces(&self) -> &Arc<RwLock<GPUVec<Point3<u16>>>> {
        &self.faces
    }

    /// This mesh vertex coordinates.
    pub fn coords(&self) -> &Arc<RwLock<GPUVec<Point2<f32>>>> {
        &self.coords
    }

    /// This mesh texture coordinates.
    pub fn uvs(&self) -> &Arc<RwLock<GPUVec<Point2<f32>>>> {
        &self.uvs
    }
}
