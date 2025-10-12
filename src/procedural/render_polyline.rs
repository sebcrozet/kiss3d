use na::{self, Isometry2, Point2, RealField, Rotation2, Translation2, Vector2};

/// Geometric description of a polyline.
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RenderPolyline {
    /// Coordinates of the polyline vertices.
    coords: Vec<Point2<f32>>,
    /// Coordinates of the polyline normals.
    normals: Option<Vec<Vector2<f32>>>,
}

impl RenderPolyline {
    /// Creates a new polyline.
    pub fn new(coords: Vec<Point2<f32>>, normals: Option<Vec<Vector2<f32>>>) -> RenderPolyline {
        if let Some(ref ns) = normals {
            assert!(
                coords.len() == ns.len(),
                "There must be exactly one normal per vertex."
            );
        }

        RenderPolyline { coords, normals }
    }
}

impl RenderPolyline {
    /// Moves the polyline data out of it.
    pub fn unwrap(self) -> (Vec<Point2<f32>>, Option<Vec<Vector2<f32>>>) {
        (self.coords, self.normals)
    }

    /// The coordinates of this polyline vertices.
    #[inline]
    pub fn coords(&self) -> &[Point2<f32>] {
        &self.coords[..]
    }

    /// The mutable coordinates of this polyline vertices.
    #[inline]
    pub fn coords_mut(&mut self) -> &mut [Point2<f32>] {
        &mut self.coords[..]
    }

    /// The normals of this polyline vertices.
    #[inline]
    pub fn normals(&self) -> Option<&[Vector2<f32>]> {
        self.normals.as_ref().map(Vec::as_slice)
    }

    /// The mutable normals of this polyline vertices.
    #[inline]
    pub fn normals_mut(&mut self) -> Option<&mut [Vector2<f32>]> {
        self.normals.as_mut().map(Vec::as_mut_slice)
    }

    /// Translates each vertex of this polyline.
    pub fn translate_by(&mut self, t: &Translation2<f32>) {
        for c in self.coords.iter_mut() {
            *c = t * &*c;
        }
    }

    /// Rotates each vertex and normal of this polyline.
    pub fn rotate_by(&mut self, r: &Rotation2<f32>) {
        for c in self.coords.iter_mut() {
            *c = r * &*c;
        }

        for n in self.normals.iter_mut() {
            for n in n.iter_mut() {
                *n = r * &*n;
            }
        }
    }

    /// Transforms each vertex and rotates each normal of this polyline.
    pub fn transform_by(&mut self, t: &Isometry2<f32>) {
        for c in self.coords.iter_mut() {
            *c = t * &*c;
        }

        for n in self.normals.iter_mut() {
            for n in n.iter_mut() {
                *n = t * &*n;
            }
        }
    }

    /// Apply a transformation to every vertex and normal of this polyline and returns it.
    #[inline]
    pub fn transformed(mut self, t: &Isometry2<f32>) -> Self {
        self.transform_by(t);
        self
    }

    /// Scales each vertex of this polyline.
    pub fn scale_by_scalar(&mut self, s: f32) {
        for c in self.coords.iter_mut() {
            *c = *c * s
        }
        // FIXME: do something for the normals?
    }

    /// Scales each vertex of this mesh.
    #[inline]
    pub fn scale_by(&mut self, s: &Vector2<f32>) {
        for c in self.coords.iter_mut() {
            for i in 0..2 {
                c[i] = (*c)[i] * s[i];
            }
        }
        // FIXME: do something for the normals?
    }

    /// Apply a scaling to every vertex and normal of this polyline and returns it.
    #[inline]
    pub fn scaled(mut self, s: &Vector2<f32>) -> Self {
        self.scale_by(s);
        self
    }
}
