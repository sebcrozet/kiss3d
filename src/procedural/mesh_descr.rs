use collections::HashMap;
use procedural::utils;
use nalgebra::na::{Iterable, Translate, Rotate, Transform, Vec3, Vec2};

/// Different representation of the index buffer.
#[deriving(Clone, Show)]
pub enum IndexBuffer {
    /// The vertex, normal, and uvs share the same indices.
    UnifiedIndexBuffer(Vec<Vec3<u32>>),
    /// The vertex, normal, and uvs have different indices.
    SplitIndexBuffer(Vec<Vec3<Vec3<u32>>>)
}

impl IndexBuffer {
    /// Returns the unified index buffer data or fails.
    pub fn unwrap_unified(self) -> Vec<Vec3<u32>> {
        match self {
            UnifiedIndexBuffer(b) => b,
            _                     => fail!("Unable to unwrap to an unified buffer.")
        }
    }

    /// Returns the split index buffer data or fails.
    pub fn unwrap_split(self) -> Vec<Vec3<Vec3<u32>>> {
        match self {
            SplitIndexBuffer(b) => b,
            _                   => fail!("Unable to unwrap to a split buffer.")
        }
    }
}

#[deriving(Clone, Show)]
/// Geometric description of a mesh.
pub struct MeshDescr<N> {
    /// Coordinates of the mesh vertices.
    pub coords:  Vec<Vec3<N>>,
    /// Coordinates of the mesh normals.
    pub normals: Option<Vec<Vec3<N>>>,
    /// Textures coordinates of the mesh.
    pub uvs:     Option<Vec<Vec2<N>>>,
    /// Index buffer of the mesh.
    pub indices: IndexBuffer
}

impl<N> MeshDescr<N> {
    /// Creates a new `MeshDescr`.
    ///
    /// If no `indices` is provided, trivial, sequential indices are generated.
    pub fn new(coords:  Vec<Vec3<N>>,
               normals: Option<Vec<Vec3<N>>>,
               uvs:     Option<Vec<Vec2<N>>>,
               indices: Option<IndexBuffer>)
               -> MeshDescr<N> {
        // generate trivial indices
        let idx = indices.unwrap_or_else(||
           UnifiedIndexBuffer(
               Vec::from_fn(coords.len() / 3, |i| Vec3::new(i as u32 * 3, i as u32 * 3 + 1, i as u32 * 3 + 2))
           )
        );

        MeshDescr {
            coords:  coords,
            normals: normals,
            uvs:     uvs,
            indices: idx
        }
    }

    /// Translates each vertex of this mesh.
    pub fn translate_by<T: Translate<Vec3<N>>>(&mut self, t: &T) {
        for c in self.coords.mut_iter() {
            *c = t.translate(c);
        }
    }

    /// Rotates each vertex and normal of this mesh.
    pub fn rotate_by<R: Rotate<Vec3<N>>>(&mut self, r: &R) {
        for c in self.coords.mut_iter() {
            *c = r.rotate(c);
        }

        for n in self.normals.mut_iter() {
            for n in n.mut_iter() {
                *n = r.rotate(n);
            }
        }
    }

    /// Transforms each vertex and rotates each normal of this mesh.
    pub fn transform_by<T: Transform<Vec3<N>> + Rotate<Vec3<N>>>(&mut self, t: &T) {
        for c in self.coords.mut_iter() {
            *c = t.transform(c);
        }

        for n in self.normals.mut_iter() {
            for n in n.mut_iter() {
                *n = t.rotate(n);
            }
        }
    }
}

impl<N: Mul<N, N>> MeshDescr<N> {
    /// Scales each vertex of this mesh.
    pub fn scale_by(&mut self, s: &Vec3<N>) {
        for c in self.coords.mut_iter() {
            c.x = c.x * s.x;
            c.y = c.y * s.y;
            c.z = c.z * s.z;
        }
        // FIXME: do something for the normals?
    }

    /// Scales each vertex of this mesh.
    pub fn scale_by_scalar(&mut self, s: &N) {
        for c in self.coords.mut_iter() {
            c.x = c.x * *s;
            c.y = c.y * *s;
            c.z = c.z * *s;
        }
    }
}

impl<N: Clone> MeshDescr<N> {
    // FIXME: looks very similar to the `reformat` on obj.rs
    /// Force the mesh to use the same index for vertices, normals and uvs.
    ///
    /// This might cause the duplication of some vertices, normals and uvs.
    /// Use this method to transform the mesh data to a OpenGL-compliant format.
    pub fn unify_index_buffer(&mut self) {
        let new_indices = match self.indices {
            SplitIndexBuffer(ref ids) => {
                let mut vt2id:HashMap<Vec3<u32>, u32> = HashMap::new();
                let mut resi: Vec<u32>                = Vec::new();
                let mut resc: Vec<Vec3<N>>            = Vec::new();
                let mut resn: Option<Vec<Vec3<N>>>    = self.normals.as_ref().map(|_| Vec::new());
                let mut resu: Option<Vec<Vec2<N>>>    = self.uvs.as_ref().map(|_| Vec::new());

                for triangle in ids.iter() {
                    for point in triangle.iter() {
                        let idx = match vt2id.find(point) {
                            Some(i) => { resi.push(*i); None },
                            None    => {
                                let idx = resc.len() as u32;

                                resc.push(self.coords.get(point.x as uint).clone());

                                let _ = resu.as_mut().map(|l| l.push(self.uvs.get_ref().get(point.y as uint).clone()));
                                let _ = resn.as_mut().map(|l| l.push(self.normals.get_ref().get(point.z as uint).clone()));

                                resi.push(idx);

                                Some(idx)
                            }
                        };

                        let _ = idx.map(|i| vt2id.insert(point.clone(), i));
                    }
                }

                self.coords  = resc;
                self.normals = resn;
                self.uvs     = resu;

                let mut batchedIndices = Vec::new();

                assert!(resi.len() % 3 == 0);
                for f in resi.as_slice().chunks(3) {
                    batchedIndices.push(Vec3::new(f[0], f[1], f[2]));
                }

                Some(UnifiedIndexBuffer(batchedIndices))
            }
            _ => None
        };

        let _ = new_indices.map(|nids| self.indices = nids);
    }

    /// Forces the mesh to use a different index for the vertices, normals and uvs.
    pub fn split_index_buffer(&mut self) {
        let new_indices = match self.indices {
            UnifiedIndexBuffer(ref ids) => {
                let resi = utils::split_index_buffer(ids.as_slice());

                Some(SplitIndexBuffer(resi))
            },
            _ => None
        };

        let _ = new_indices.map(|nids| self.indices = nids);
    }
}
