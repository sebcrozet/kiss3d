use nalgebra::na::Cast;
use nalgebra::na;
use procedural::{MeshDescr, ProceduralGenerator};
use procedural;

/// Procedural generator of cylinders.
pub struct CylinderGenerator<N> {
    // FIXME: use the radius instead
    diameter: N,
    height:   N,
    nsubdiv:  u32
}

impl<N: Float + Cast<f64> + Clone> CylinderGenerator<N> {
    /// Creates a new cylinder generator.
    ///
    /// # Parameters:
    /// * `height` - the height of the cylinder. This is _not_ the total height of the
    /// cylinder since it does not include the caps.
    /// * `diameter` - the cylinder diameter.
    /// * `nsubdiv` - number of subdivisions on the horizontal planes.
    pub fn new(diameter: N, height: N, nsubdiv: u32) -> CylinderGenerator<N> {
        assert!(nsubdiv > 1);

        CylinderGenerator {
            diameter: diameter,
            height:   height,
            nsubdiv:  nsubdiv,
        }
    }

    /// Creates a new generator for a cylinder with unit height and diameter.
    ///
    /// # Parameters:
    /// * `nsubdiv` - number of subdivisions on the horizontal planes.
    pub fn new_unit(nsubdiv: u32) -> CylinderGenerator<N> {
        CylinderGenerator::new(na::one(), na::one(), nsubdiv)
    }

    /// The cylinder height.
    #[inline]
    pub fn height(&self) -> N {
        self.height.clone()
    }

    /// The cylinder diameter.
    #[inline]
    pub fn diameter(&self) -> N {
        self.diameter.clone()
    }

    /// Floatber of subdivisions on the horizontal planes.
    #[inline]
    pub fn nsubdiv(&self) -> u32 {
        self.nsubdiv
    }

    /// The cylinder height.
    #[inline]
    pub fn set_height(&mut self, height: N) {
        self.height = height
    }

    /// The cylinder diameter.
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

impl<N: Float + Clone + Cast<f64>> ProceduralGenerator<N> for CylinderGenerator<N> {
    fn generate(&self) -> MeshDescr<N> {
        procedural::cylinder(self.diameter.clone(), self.height.clone(), self.nsubdiv)
    }
}
