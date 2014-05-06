use nalgebra::na::Cast;
use procedural::{MeshDescr, ProceduralGenerator};
use procedural;

/// Procedural generator of capsules.
pub struct CapsuleGenerator<N> {
    // FIXME: use the radius instead
    diameter:      N,
    height:        N,
    ntheta_subdiv: u32,
    nphi_subdiv:   u32
}

impl<N: Float + Cast<f64> + Clone> CapsuleGenerator<N> {
    /// Creates a new capsule generator.
    ///
    /// # Parameters:
    /// * `height` - the height of the capsule's cylinder. This is _not_ the total height of the
    /// capsule since it does not include the caps.
    /// * `diameter` - the capsule diameter.
    /// * `ntheta_subdiv` - number of subdivisions on the horizontal planes.
    /// * `nphi_subdiv`   - number of subdivisions on the vertical planes.
    pub fn new(diameter: N, height: N, ntheta_subdiv: u32, nphi_subdiv: u32) -> CapsuleGenerator<N> {
        assert!(ntheta_subdiv > 1 && nphi_subdiv > 1);

        CapsuleGenerator {
            height:        height,
            diameter:      diameter,
            ntheta_subdiv: ntheta_subdiv,
            nphi_subdiv:   nphi_subdiv
        }
    }

    /// The capsule height.
    #[inline]
    pub fn height(&self) -> N {
        self.height.clone()
    }

    /// The capsule diameter.
    #[inline]
    pub fn diameter(&self) -> N {
        self.diameter.clone()
    }

    /// Floatber of subdivisions on the horizontal planes.
    #[inline]
    pub fn ntheta_subdiv(&self) -> u32 {
        self.ntheta_subdiv
    }

    /// Floatber of subdivisions on the vertical planes.
    #[inline]
    pub fn nphi_subdiv(&self) -> u32 {
        self.nphi_subdiv
    }

    /// The capsule height.
    #[inline]
    pub fn set_height(&mut self, height: N) {
        self.height = height
    }

    /// The capsule diameter.
    #[inline]
    pub fn set_diameter(&mut self, diameter: N) {
        self.diameter = diameter
    }

    /// Floatber of subdivisions on the horizontal planes.
    #[inline]
    pub fn set_ntheta_subdiv(&mut self, ntheta_subdiv: u32) {
        assert!(ntheta_subdiv > 1);
        self.ntheta_subdiv = ntheta_subdiv
    }

    /// Floatber of subdivisions on the vertical planes.
    #[inline]
    pub fn set_nphi_subdiv(&mut self, nphi_subdiv: u32) {
        assert!(nphi_subdiv > 1);
        self.nphi_subdiv = nphi_subdiv
    }
}

impl<N: Float + Cast<f64>> ProceduralGenerator<N> for CapsuleGenerator<N> {
    fn generate(&self) -> MeshDescr<N> {
        procedural::capsule(&self.diameter, &self.height, self.ntheta_subdiv, self.nphi_subdiv)
    }
}
