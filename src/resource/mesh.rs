//! Data structure of a scene node geometry.

use std::sync::{Arc, RwLock};
use gl::types::*;
use na::{Pnt2, Vec3, Pnt3};
use na;
use ncollide_procedural::{TriMesh, TriMesh3, IndexBuffer};
use resource::ShaderAttribute;
use resource::gpu_vector::{GPUVector, AllocationType, BufferType};
use std::iter;

#[path = "../error.rs"]
mod error;

/// Aggregation of vertices, indices, normals and texture coordinates.
///
/// It also contains the GPU location of those buffers.
pub struct Mesh {
    coords:  Arc<RwLock<GPUVector<Pnt3<GLfloat>>>>,
    faces:   Arc<RwLock<GPUVector<Pnt3<GLuint>>>>,
    normals: Arc<RwLock<GPUVector<Vec3<GLfloat>>>>,
    uvs:     Arc<RwLock<GPUVector<Pnt2<GLfloat>>>>
}

impl Mesh {
    /// Creates a new mesh.
    ///
    /// If the normals and uvs are not given, they are automatically computed.
    pub fn new(coords:       Vec<Pnt3<GLfloat>>,
               faces:        Vec<Pnt3<GLuint>>,
               normals:      Option<Vec<Vec3<GLfloat>>>,
               uvs:          Option<Vec<Pnt2<GLfloat>>>,
               dynamic_draw: bool)
               -> Mesh {
        let normals = match normals {
            Some(ns) => ns,
            None     => Mesh::compute_normals_array(&coords[..], &faces[..])
        };

        let uvs = match uvs {
            Some(us) => us,
            None     => iter::repeat(na::orig()).take(coords.len()).collect()
        };

        let location = if dynamic_draw { AllocationType::DynamicDraw } else { AllocationType::StaticDraw };
        let cs = Arc::new(RwLock::new(GPUVector::new(coords, BufferType::Array, location)));
        let fs = Arc::new(RwLock::new(GPUVector::new(faces, BufferType::ElementArray, location)));
        let ns = Arc::new(RwLock::new(GPUVector::new(normals, BufferType::Array, location)));
        let us = Arc::new(RwLock::new(GPUVector::new(uvs, BufferType::Array, location)));

        Mesh::new_with_gpu_vectors(cs, fs, ns, us)
    }

    /// Creates a new mesh from a mesh descr.
    ///
    /// In the normals and uvs are not given, they are automatically computed.
    pub fn from_trimesh(mesh: TriMesh3<GLfloat>, dynamic_draw: bool) -> Mesh {
        let mut mesh = mesh;

        mesh.unify_index_buffer();

        let TriMesh { coords, normals, uvs, indices } = mesh;
        
        Mesh::new(coords, indices.unwrap_unified(), normals, uvs, dynamic_draw)
    }

    /// Creates a triangle mesh from this mesh.
    pub fn to_trimesh(&self) -> Option<TriMesh3<GLfloat>> {
        let unload_coords  = !self.coords.read().unwrap().is_on_ram();
        let unload_faces   = !self.faces.read().unwrap().is_on_ram();
        let unload_normals = !self.normals.read().unwrap().is_on_ram();
        let unload_uvs     = !self.uvs.read().unwrap().is_on_ram();

        self.coords.write().unwrap().load_to_ram();
        self.faces.write().unwrap().load_to_ram();
        self.normals.write().unwrap().load_to_ram();
        self.uvs.write().unwrap().load_to_ram();

        let coords  = self.coords.read().unwrap().to_owned();
        let faces   = self.faces.read().unwrap().to_owned();
        let normals = self.normals.read().unwrap().to_owned();
        let uvs     = self.uvs.read().unwrap().to_owned();

        if unload_coords {
            self.coords.write().unwrap().unload_from_ram();
        }
        if unload_faces {
            self.coords.write().unwrap().unload_from_ram();
        }
        if unload_normals {
            self.coords.write().unwrap().unload_from_ram();
        }
        if unload_uvs {
            self.coords.write().unwrap().unload_from_ram();
        }

        if coords.is_none() || faces.is_none() {
            None
        }
        else {
            Some(TriMesh::new(coords.unwrap(), normals, uvs, Some(IndexBuffer::Unified(faces.unwrap()))))
        }
    }

    /// Creates a new mesh. Arguments set to `None` are automatically computed.
    pub fn new_with_gpu_vectors(coords:  Arc<RwLock<GPUVector<Pnt3<GLfloat>>>>,
                                faces:   Arc<RwLock<GPUVector<Pnt3<GLuint>>>>,
                                normals: Arc<RwLock<GPUVector<Vec3<GLfloat>>>>,
                                uvs:     Arc<RwLock<GPUVector<Pnt2<GLfloat>>>>)
                                -> Mesh {
        Mesh {
            coords:  coords,
            faces:   faces,
            normals: normals,
            uvs:     uvs
        }
    }

    /// Binds this mesh vertex coordinates buffer to a vertex attribute.
    pub fn bind_coords(&mut self, coords: &mut ShaderAttribute<Pnt3<GLfloat>>) {
        coords.bind(&mut *self.coords.write().unwrap());
    }

    /// Binds this mesh vertex normals buffer to a vertex attribute.
    pub fn bind_normals(&mut self, normals: &mut ShaderAttribute<Vec3<GLfloat>>) {
        normals.bind(&mut *self.normals.write().unwrap());
    }

    /// Binds this mesh vertex uvs buffer to a vertex attribute.
    pub fn bind_uvs(&mut self, uvs: &mut ShaderAttribute<Pnt2<GLfloat>>) {
        uvs.bind(&mut *self.uvs.write().unwrap());
    }

    /// Binds this mesh vertex uvs buffer to a vertex attribute.
    pub fn bind_faces(&mut self) {
        self.faces.write().unwrap().bind();
    }

    /// Binds this mesh buffers to vertex attributes.
    pub fn bind(&mut self,
                coords:  &mut ShaderAttribute<Pnt3<GLfloat>>,
                normals: &mut ShaderAttribute<Vec3<GLfloat>>,
                uvs:     &mut ShaderAttribute<Pnt2<GLfloat>>) {
        self.bind_coords(coords);
        self.bind_normals(normals);
        self.bind_uvs(uvs);
        self.bind_faces();
    }

    /// Unbind this mesh buffers to vertex attributes.
    pub fn unbind(&self) {
        self.coords.write().unwrap().unbind();
        self.normals.write().unwrap().unbind();
        self.uvs.write().unwrap().unbind();
        self.faces.write().unwrap().unbind();
    }

    /// Number of points needed to draw this mesh.
    pub fn num_pts(&self) -> usize {
        self.faces.read().unwrap().len() * 3
    }

    /// Recompute this mesh normals.
    pub fn recompute_normals(&mut self) {
        Mesh::compute_normals(&self.coords.read().unwrap().data().as_ref().unwrap()[..],
                              &self.faces.read().unwrap().data().as_ref().unwrap()[..],
                              self.normals.write().unwrap().data_mut().as_mut().unwrap());
    }

    /// This mesh faces.
    pub fn faces<'a>(&'a self) -> &'a Arc<RwLock<GPUVector<Pnt3<GLuint>>>> {
        &self.faces
    }

    /// This mesh normals.
    pub fn normals<'a>(&'a self) -> &'a Arc<RwLock<GPUVector<Vec3<GLfloat>>>> {
        &self.normals
    }

    /// This mesh vertex coordinates.
    pub fn coords<'a>(&'a self) -> &'a Arc<RwLock<GPUVector<Pnt3<GLfloat>>>> {
        &self.coords
    }

    /// This mesh texture coordinates.
    pub fn uvs<'a>(&'a self) -> &'a Arc<RwLock<GPUVector<Pnt2<GLfloat>>>> {
        &self.uvs
    }

    /// Computes normals from a set of faces.
    pub fn compute_normals_array(coordinates: &[Pnt3<GLfloat>], faces: &[Pnt3<GLuint>]) -> Vec<Vec3<GLfloat>> {
        let mut res = Vec::new();
    
        Mesh::compute_normals(coordinates, faces, &mut res);
    
        res
    }
    
    /// Computes normals from a set of faces.
    pub fn compute_normals(coordinates: &[Pnt3<GLfloat>],
                           faces:       &[Pnt3<GLuint>],
                           normals:     &mut Vec<Vec3<GLfloat>>) {
        let mut divisor:Vec<f32> = iter::repeat(0f32).take(coordinates.len()).collect();
    
        normals.clear();
        normals.extend(iter::repeat(na::zero::<Vec3<GLfloat>>()).take(coordinates.len()));
    
        // Accumulate normals ...
        for f in faces.iter() {
            let edge1  = coordinates[f.y as usize] - coordinates[f.x as usize];
            let edge2  = coordinates[f.z as usize] - coordinates[f.x as usize];
            let cross  = na::cross(&edge1, &edge2);
            let normal;
    
            if !na::is_zero(&cross) {
                normal = na::normalize(&cross)
            }
            else {
                normal = cross
            }
    
            normals[f.x as usize] = normals[f.x as usize] + normal;
            normals[f.y as usize] = normals[f.y as usize] + normal;
            normals[f.z as usize] = normals[f.z as usize] + normal;
    
            divisor[f.x as usize] = divisor[f.x as usize] + 1.0;
            divisor[f.y as usize] = divisor[f.y as usize] + 1.0;
            divisor[f.z as usize] = divisor[f.z as usize] + 1.0;
        }
    
        // ... and compute the mean
        for (n, divisor) in normals.iter_mut().zip(divisor.iter()) {
            *n = *n / *divisor
        }
    }
}
