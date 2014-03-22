//! Data structure of a scene node geometry.

use std::num::Zero;
use sync::RWArc;
use gl::types::*;
use nalgebra::na::{Vec2, Vec3};
use nalgebra::na;
use resource::ShaderAttribute;
use resource::gpu_vector::{GPUVector, DynamicDraw, StaticDraw, ArrayBuffer, ElementArrayBuffer};
use procedural::MeshDescr;

#[path = "../error.rs"]
mod error;

/// Aggregation of vertices, indices, normals and texture coordinates.
///
/// It also contains the GPU location of those buffers.
pub struct Mesh {
    priv coords:  RWArc<GPUVector<Vec3<GLfloat>>>,
    priv faces:   RWArc<GPUVector<Vec3<GLuint>>>,
    priv normals: RWArc<GPUVector<Vec3<GLfloat>>>,
    priv uvs:     RWArc<GPUVector<Vec2<GLfloat>>>
}

impl Mesh {
    /// Creates a new mesh.
    ///
    /// If the normals and uvs are not given, they are automatically computed.
    pub fn new(coords:       Vec<Vec3<GLfloat>>,
               faces:        Vec<Vec3<GLuint>>,
               normals:      Option<Vec<Vec3<GLfloat>>>,
               uvs:          Option<Vec<Vec2<GLfloat>>>,
               dynamic_draw: bool)
               -> Mesh {
        let normals = match normals {
            Some(ns) => ns,
            None     => Mesh::compute_normals_array(coords.as_slice(), faces.as_slice())
        };

        let uvs = match uvs {
            Some(us) => us,
            None     => Vec::from_elem(coords.len(), na::zero())
        };

        let location = if dynamic_draw { DynamicDraw } else { StaticDraw };
        let cs = RWArc::new(GPUVector::new(coords, ArrayBuffer, location));
        let fs = RWArc::new(GPUVector::new(faces, ElementArrayBuffer, location));
        let ns = RWArc::new(GPUVector::new(normals, ArrayBuffer, location));
        let us = RWArc::new(GPUVector::new(uvs, ArrayBuffer, location));

        Mesh::new_with_gpu_vectors(cs, fs, ns, us)
    }

    /// Creates a new mesh from a mesh descr.
    ///
    /// In the normals and uvs are not given, they are automatically computed.
    pub fn from_mesh_desc(mesh: MeshDescr<GLfloat>, dynamic_draw: bool) -> Mesh {
        let mut mesh = mesh;

        mesh.unify_index_buffer();

        let MeshDescr { coords, normals, uvs, indices } = mesh;
        
        Mesh::new(coords, indices.unwrap_unified(), normals, uvs, dynamic_draw)
    }

    /// Creates a new mesh. Arguments set to `None` are automatically computed.
    pub fn new_with_gpu_vectors(coords:  RWArc<GPUVector<Vec3<GLfloat>>>,
                                faces:   RWArc<GPUVector<Vec3<GLuint>>>,
                                normals: RWArc<GPUVector<Vec3<GLfloat>>>,
                                uvs:     RWArc<GPUVector<Vec2<GLfloat>>>)
                                -> Mesh {
        Mesh {
            coords:  coords,
            faces:   faces,
            normals: normals,
            uvs:     uvs
        }
    }

    /// Binds this mesh vertex coordinates buffer to a vertex attribute.
    pub fn bind_coords(&mut self, coords: &mut ShaderAttribute<Vec3<GLfloat>>) {
        self.coords.write(|c| coords.bind(c));
    }

    /// Binds this mesh vertex normals buffer to a vertex attribute.
    pub fn bind_normals(&mut self, normals: &mut ShaderAttribute<Vec3<GLfloat>>) {
        self.normals.write(|n| normals.bind(n));
    }

    /// Binds this mesh vertex uvs buffer to a vertex attribute.
    pub fn bind_uvs(&mut self, uvs: &mut ShaderAttribute<Vec2<GLfloat>>) {
        self.uvs.write(|u| uvs.bind(u));
    }

    /// Binds this mesh vertex uvs buffer to a vertex attribute.
    pub fn bind_faces(&mut self) {
        self.faces.write(|c| c.bind());
    }

    /// Binds this mesh buffers to vertex attributes.
    pub fn bind(&mut self,
                coords:  &mut ShaderAttribute<Vec3<GLfloat>>,
                normals: &mut ShaderAttribute<Vec3<GLfloat>>,
                uvs:     &mut ShaderAttribute<Vec2<GLfloat>>) {
        self.bind_coords(coords);
        self.bind_normals(normals);
        self.bind_uvs(uvs);
        self.bind_faces();
    }

    /// Unbind this mesh buffers to vertex attributes.
    pub fn unbind(&self) {
        self.coords.write(|c| c.unbind());
        self.normals.write(|c| c.unbind());
        self.uvs.write(|c| c.unbind());
        self.faces.write(|c| c.unbind());
    }

    /// Number of points needed to draw this mesh.
    pub fn num_pts(&self) -> uint {
        self.faces.read(|f| f.len() * 3)
    }

    /// Recompute this mesh normals.
    pub fn recompute_normals(&mut self) {
        let _ =
            self.normals.write(|ns|
                self.coords.read(|cs|
                   self.faces.read(|fs|
                       Mesh::compute_normals(cs.data().get_ref().as_slice(),
                                             fs.data().get_ref().as_slice(),
                                             ns.data_mut().get_mut_ref())
                   )
                )
            );
    }

    /// This mesh faces.
    pub fn faces<'a>(&'a self) -> &'a RWArc<GPUVector<Vec3<GLuint>>> {
        &'a self.faces
    }

    /// This mesh normals.
    pub fn normals<'a>(&'a self) -> &'a RWArc<GPUVector<Vec3<GLfloat>>> {
        &'a self.normals
    }

    /// This mesh vertex coordinates.
    pub fn coords<'a>(&'a self) -> &'a RWArc<GPUVector<Vec3<GLfloat>>> {
        &'a self.coords
    }

    /// This mesh texture coordinates.
    pub fn uvs<'a>(&'a self) -> &'a RWArc<GPUVector<Vec2<GLfloat>>> {
        &'a self.uvs
    }

    /// Computes normals from a set of faces.
    pub fn compute_normals_array(coordinates: &[Vec3<GLfloat>], faces: &[Vec3<GLuint>]) -> Vec<Vec3<GLfloat>> {
        let mut res = Vec::new();
    
        Mesh::compute_normals(coordinates, faces, &mut res);
    
        res
    }
    
    /// Computes normals from a set of faces.
    pub fn compute_normals(coordinates: &[Vec3<GLfloat>],
                           faces:       &[Vec3<GLuint>],
                           normals:     &mut Vec<Vec3<GLfloat>>) {
        let mut divisor = Vec::from_elem(coordinates.len(), 0f32);
    
        // Shrink the output buffer if it is too big.
        if normals.len() > coordinates.len() {
            normals.truncate(coordinates.len())
        }
    
        // Reinit all normals to zero.
        for n in normals.mut_iter() {
            *n = na::zero()
        }
    
        // Grow the output buffer if it is too small.
        normals.grow_set(coordinates.len() - 1, &na::zero(), na::zero());
    
        // Accumulate normals ...
        for f in faces.iter() {
            let edge1  = coordinates[f.y] - coordinates[f.x];
            let edge2  = coordinates[f.z] - coordinates[f.x];
            let cross  = na::cross(&edge1, &edge2);
            let normal;
    
            if !cross.is_zero() {
                normal = na::normalize(&cross)
            }
            else {
                normal = cross
            }
    
            *normals.get_mut(f.x as uint) = *normals.get(f.x as uint) + normal;
            *normals.get_mut(f.y as uint) = *normals.get(f.y as uint) + normal;
            *normals.get_mut(f.z as uint) = *normals.get(f.z as uint) + normal;
    
            *divisor.get_mut(f.x as uint) = *divisor.get(f.x as uint) + 1.0;
            *divisor.get_mut(f.y as uint) = *divisor.get(f.y as uint) + 1.0;
            *divisor.get_mut(f.z as uint) = *divisor.get(f.z as uint) + 1.0;
        }
    
        // ... and compute the mean
        for (n, divisor) in normals.mut_iter().zip(divisor.iter()) {
            *n = *n / *divisor
        }
    }
}
