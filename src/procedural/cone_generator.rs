use nalgebra::na::Cast;
use nalgebra::na;
use procedural::{MeshDescr, ProceduralGenerator};
use procedural;

/// Procedural generator of cones.
pub struct ConeGenerator<N> {
    // FIXME: use the radius instead
    diameter: N,
    height:   N,
    nsubdiv:  u32
}

impl<N: Float + Cast<f64> + Clone> ConeGenerator<N> {
    /// Creates a new cone generator.
    ///
    /// # Parameters:
    /// * `height` - the height of the cone. This is _not_ the total height of the
    /// cone since it does not include the caps.
    /// * `diameter` - the cone diameter.
    /// * `nsubdiv` - number of subdivisions on the horizontal planes.
    pub fn new(diameter: N, height: N, nsubdiv: u32) -> ConeGenerator<N> {
        assert!(nsubdiv > 1);

        ConeGenerator {
            diameter:      diameter,
            height:        height,
            nsubdiv: nsubdiv,
        }
    }

    /// Creates a new generator for a cone with unit height and diameter.
    ///
    /// # Parameters:
    /// * `nsubdiv` - number of subdivisions on the horizontal planes.
    pub fn new_unit(nsubdiv: u32) -> ConeGenerator<N> {
        ConeGenerator::new(na::one(), na::one(), nsubdiv)
    }

    /// The cone height.
    #[inline]
    pub fn height(&self) -> N {
        self.height.clone()
    }

    /// The cone diameter.
    #[inline]
    pub fn diameter(&self) -> N {
        self.diameter.clone()
    }

    /// Floatber of subdivisions on the horizontal planes.
    #[inline]
    pub fn nsubdiv(&self) -> u32 {
        self.nsubdiv
    }

    /// The cone height.
    #[inline]
    pub fn set_height(&mut self, height: N) {
        self.height = height
    }

    /// The cone diameter.
    #[inline]
    pub fn set_diameter(&mut self, diameter: N) {
        self.diameter = diameter
    }

    /// Floatber of subdivisions on the horizontal planes.
    #[inline]
    pub fn set_nsubdiv(&mut self, nsubdiv: u32) {
        assert!(nsubdiv > 1);
        self.nsubdiv = nsubdiv
    }
}

impl<N: Float + Clone + Cast<f64>> ProceduralGenerator<N> for ConeGenerator<N> {
    fn generate(&self) -> MeshDescr<N> {
        procedural::cone(self.diameter.clone(), self.height.clone(), self.nsubdiv)
    }
}
