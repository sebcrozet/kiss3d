use nalgebra::na::Cast;
use nalgebra::na;
use procedural::{MeshDescr, ProceduralGenerator};
use procedural;

/// Procedural generator of spheres.
pub struct SphereGenerator<N> {
    // FIXME: use the radius instead
    diameter:      N,
    ntheta_subdiv: u32,
    nphi_subdiv:   u32
}

impl<N: Float + Cast<f64> + Clone> SphereGenerator<N> {
    /// Creates a new sphere generator.
    ///
    /// # Parameters:
    /// * `diameter` - the sphere diameter.
    /// * `ntheta_subdiv` - number of subdivisions on the horizontal planes.
    /// * `nphi_subdiv`   - number of subdivisions on the vertical planes.
    pub fn new(diameter: N, ntheta_subdiv: u32, nphi_subdiv: u32) -> SphereGenerator<N> {
        assert!(ntheta_subdiv > 1 && nphi_subdiv > 1);

        SphereGenerator {
            diameter:      diameter,
            ntheta_subdiv: ntheta_subdiv,
            nphi_subdiv:   nphi_subdiv
        }
    }

    /// Creates a new generator of spheres with unit diameter.
    ///
    /// # Parameters:
    /// * `ntheta_subdiv` - number of subdivisions on the horizontal planes.
    /// * `nphi_subdiv`   - number of subdivisions on the vertical planes.
    pub fn new_unit(ntheta_subdiv: u32, nphi_subdiv: u32) -> SphereGenerator<N> {
        SphereGenerator::new(na::one(), ntheta_subdiv, nphi_subdiv)
    }

    /// The sphere diameter.
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

    /// The sphere diameter.
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

impl<N: Float + Cast<f64>> ProceduralGenerator<N> for SphereGenerator<N> {
    fn generate(&self) -> MeshDescr<N> {
        procedural::sphere(&self.diameter, self.ntheta_subdiv, self.nphi_subdiv)
    }
}
