use nalgebra::na::{Vec3, Cast};
use procedural::{MeshDescr, ProceduralGenerator};
use procedural;

/// Procedural generator of non-rational Bézier surfaces.
pub struct BezierGenerator<N> {
    control_points: Vec<Vec3<N>>,
    nupoints:       uint,
    nvpoints:       uint,
    usubdivs:       uint,
    vsubdivs:       uint
}

impl<N> BezierGenerator<N> {
    /// Creates a new procedural generator of non-rational bézier surfaces.
    ///
    /// # Parameters:
    /// * `control_points`: The control points of the bézier surface.
    /// * `nupoints`:       The number of control points per u-iso-curve.
    /// * `nvpoints`:       The number of control points per v-iso-curve.
    /// * `usubdivs`:       The number of samples generated for the `u` parameter.
    /// * `vsubdivs`:       The number of samples generated for the `v` parameter.
    ///
    /// # Failures:
    /// Fails if the vector of control points does not contain exactly `nupoints * nvpoints`
    /// elements.
    pub fn new(control_points: Vec<Vec3<N>>,
               nupoints:       uint,
               nvpoints:       uint,
               usubdivs:       uint,
               vsubdivs:       uint)
               -> BezierGenerator<N> {
        assert!(nupoints * nvpoints == control_points.len());
        assert!(usubdivs > 1 && vsubdivs > 1);

        BezierGenerator {
            control_points: control_points,
            nupoints:       nupoints,
            nvpoints:       nvpoints,
            usubdivs:       usubdivs,
            vsubdivs:       vsubdivs
        }
    }

    /// The control points of this bézier surface.
    #[inline]
    pub fn control_points<'a>(&'a self) -> &'a [Vec3<N>] {
        self.control_points.as_slice()
    }

    /// The number of control points per u-iso-curve.
    #[inline]
    pub fn nupoints(&self) -> uint {
        self.nupoints
    }

    /// The number of control points per v-iso-curve.
    #[inline]
    pub fn nvpoints(&self) -> uint {
        self.nvpoints
    }

    /// The number of subdivisions along the `u` parameter axis.
    #[inline]
    pub fn usubdivs(&self) -> uint {
        self.usubdivs
    }

    /// Sets the number of subdivisions along the `u` parameter axis.
    #[inline]
    pub fn set_usubdivs(&mut self, usubdivs: uint) {
        assert!(usubdivs > 1);
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
        assert!(vsubdivs > 1);
        self.vsubdivs = vsubdivs
    }
}

impl<N: Float + Clone + Cast<f64>> ProceduralGenerator<N> for BezierGenerator<N> {
    fn generate(&self) -> MeshDescr<N> {
        procedural::bezier_surface(self.control_points.as_slice(),
                                   self.nupoints,
                                   self.nvpoints,
                                   self.usubdivs,
                                   self.vsubdivs)
    }
}
