//! Data structure of a scene node geometry.

use extra::arc::RWArc;
use std::num::Zero;
use std::vec;
use gl::types::*;
use nalgebra::na::{Vec2, Vec3};
use nalgebra::na;
use resource::gpu_vector::{GPUVector, DynamicDraw, StaticDraw, ArrayBuffer, ElementArrayBuffer};

type Coord  = Vec3<GLfloat>;
type Normal = Vec3<GLfloat>;
type UV     = Vec2<GLfloat>;
type Vertex = GLuint;
type Face   = Vec3<Vertex>;

#[path = "../error.rs"]
mod error;

/// Aggregation of vertices, indices, normals and texture coordinates.
///
/// It also contains the GPU location of those buffers.
pub struct Mesh {
    priv coords:  RWArc<GPUVector<Coord>>,
    priv faces:   RWArc<GPUVector<Face>>,
    priv normals: RWArc<GPUVector<Normal>>,
    priv uvs:     RWArc<GPUVector<UV>>
}

impl Mesh {
    /// Creates a new mesh.
    ///
    /// If the normals and uvs are not given, they are automatically computed.
    pub fn new(coords:       ~[Coord],
               faces:        ~[Face],
               normals:      Option<~[Normal]>,
               uvs:          Option<~[UV]>,
               dynamic_draw: bool)
               -> Mesh {
        let normals = match normals {
            Some(ns) => ns,
            None     => Mesh::compute_normals_array(coords, faces)
        };

        let uvs = match uvs {
            Some(us) => us,
            None     => vec::from_elem(coords.len(), na::zero())
        };

        let location = if dynamic_draw { DynamicDraw } else { StaticDraw };
        let cs = RWArc::new(GPUVector::new(coords, ArrayBuffer, location));
        let fs = RWArc::new(GPUVector::new(faces, ElementArrayBuffer, location));
        let ns = RWArc::new(GPUVector::new(normals, ArrayBuffer, location));
        let us = RWArc::new(GPUVector::new(uvs, ArrayBuffer, location));

        Mesh::new_with_gpu_vectors(cs, fs, ns, us)
    }

    /// Creates a new mesh. Arguments set to `None` are automatically computed.
    pub fn new_with_gpu_vectors(coords:  RWArc<GPUVector<Coord>>,
                                faces:   RWArc<GPUVector<Face>>,
                                normals: RWArc<GPUVector<Normal>>,
                                uvs:     RWArc<GPUVector<UV>>)
                                -> Mesh {
        Mesh {
            coords:  coords,
            faces:   faces,
            normals: normals,
            uvs:     uvs
        }
    }

    /// Binds this mesh vertex coordinates buffer to a vertex attribute.
    pub fn bind_coords(&mut self, coords: GLuint) {
        self.coords.write(|c| c.bind(Some(coords)));
    }

    /// Binds this mesh vertex normals buffer to a vertex attribute.
    pub fn bind_normals(&mut self, normals: GLuint) {
        self.normals.write(|c| c.bind(Some(normals)));
    }

    /// Binds this mesh vertex uvs buffer to a vertex attribute.
    pub fn bind_uvs(&mut self, uvs: GLuint) {
        self.uvs.write(|c| c.bind(Some(uvs)));
    }

    /// Binds this mesh vertex uvs buffer to a vertex attribute.
    pub fn bind_faces(&mut self) {
        self.faces.write(|c| c.bind(None));
    }

    /// Binds this mesh buffers to vertex attributes.
    pub fn bind(&mut self, coords: GLuint, normals: GLuint, uvs: GLuint) {
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
        let _ = self.normals.write(|ns|
            ns.write(
                |normals| {
                    self.coords.read(|cs| cs.read(|cs|
                       self.faces.read(|fs| fs.read(|fs|
                           Mesh::compute_normals(cs, fs, normals)
                       ))
                    ))
                }
            )
        );
    }

    /// This mesh faces.
    pub fn faces<'a>(&'a self) -> &'a RWArc<GPUVector<Face>> {
        &'a self.faces
    }

    /// This mesh normals.
    pub fn normals<'a>(&'a self) -> &'a RWArc<GPUVector<Normal>> {
        &'a self.normals
    }

    /// This mesh vertex coordinates.
    pub fn coords<'a>(&'a self) -> &'a RWArc<GPUVector<Coord>> {
        &'a self.coords
    }

    /// This mesh texture coordinates.
    pub fn uvs<'a>(&'a self) -> &'a RWArc<GPUVector<UV>> {
        &'a self.uvs
    }

    /// Computes normals from a set of faces.
    pub fn compute_normals_array(coordinates: &[Coord], faces: &[Face]) -> ~[Normal] {
        let mut res = ~[];
    
        Mesh::compute_normals(coordinates, faces, &mut res);
    
        res
    }
    
    /// Computes normals from a set of faces.
    pub fn compute_normals(coordinates: &[Coord],
                           faces:       &[Face],
                           normals:     &mut ~[Normal]) {
        let mut divisor = vec::from_elem(coordinates.len(), 0f32);
    
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
    
            normals[f.x] = normals[f.x] + normal;
            normals[f.y] = normals[f.y] + normal;
            normals[f.z] = normals[f.z] + normal;
    
            divisor[f.x] = divisor[f.x] + 1.0;
            divisor[f.y] = divisor[f.y] + 1.0;
            divisor[f.z] = divisor[f.z] + 1.0;
        }
    
        // ... and compute the mean
        for (n, divisor) in normals.mut_iter().zip(divisor.iter()) {
            *n = *n / *divisor
        }
    }
}
