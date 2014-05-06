use nalgebra::na::Cast;
use nalgebra::na;
use procedural::{MeshDescr, ProceduralGenerator};
use procedural;

/// Procedural generator of quads.
pub struct QuadGenerator<N> {
    width:          N,
    height:         N,
    usubdivs:       uint,
    vsubdivs:       uint
}

impl<N: Float> QuadGenerator<N> {
    /// Creates a new procedural generator of quads.
    ///
    /// # Parameters:
    /// * `width`:  the length of the quad along the `x` axis.
    /// * `height`: the length of the quad along the `y` axis.
    /// elements.
    /// * `usubdivs`: number of points to generate along the `x` axis.
    /// * `vsubdivs`: number of points to generate along the `y` axis.
    pub fn new(width: N, height: N, usubdivs: uint, vsubdivs: uint) -> QuadGenerator<N> {
        assert!(usubdivs > 0 && vsubdivs > 0, "The number of subdivisions cannot be zero.");

        QuadGenerator {
            width:    width,
            height:   height,
            usubdivs: usubdivs,
            vsubdivs: vsubdivs
        }
    }

    /// Creates a new procedural generator of quads with width and height set to 1,0.
    ///
    /// # Parameters:
    /// * `usubdivs`: number of points to generate along the `x` axis.
    /// * `vsubdivs`: number of points to generate along the `y` axis.
    pub fn new_unit(usubdivs: uint, vsubdivs: uint) -> QuadGenerator<N> {
        QuadGenerator::new(na::one(), na::one(), usubdivs, vsubdivs)
    }

    /// The number of subdivisions along the `u` parameter axis.
    #[inline]
    pub fn usubdivs(&self) -> uint {
        self.usubdivs
    }

    /// Sets the number of subdivisions along the `u` parameter axis.
    #[inline]
    pub fn set_usubdivs(&mut self, usubdivs: uint) {
        assert!(usubdivs > 0, "The number of subdivisions cannot be zero.");
        self.usubdivs = usubdivs
    }

    /// The number of subdivisions along the `v` parameter axis.
    #[inline]
    pub fn vsubdivs(&self) -> uint {
        self.vsubdivs
    }

    /// Sets the number of subdivisions along the `v` parameter axis.
    #[inline]
    pub fn set_vsubdivs(&mut self, vsubdivs: uint) {
        assert!(vsubdivs > 0, "The number of subdivisions cannot be zero.");
        self.vsubdivs = vsubdivs
    }
}

impl<N: Float + Clone + Cast<f64>> ProceduralGenerator<N> for QuadGenerator<N> {
    fn generate(&self) -> MeshDescr<N> {
        procedural::quad(self.width, self.height, self.usubdivs, self.vsubdivs)
    }
}
