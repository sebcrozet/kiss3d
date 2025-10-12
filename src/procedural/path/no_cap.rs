use crate::procedural::path::PolylineCompatibleCap;
use na::{Point3, Vector3};

/// A cap that renders nothing.
pub struct NoCap;

impl Default for NoCap {
    fn default() -> Self {
        Self::new()
    }
}

impl NoCap {
    /// Creates a new `NoCap`.
    #[inline]
    pub fn new() -> NoCap {
        NoCap
    }
}

impl PolylineCompatibleCap for NoCap {
    fn gen_start_cap(
        &self,
        _: u32,
        _: &[Point3<f32>],
        _: &Point3<f32>,
        _: &Vector3<f32>,
        _: bool,
        _: &mut Vec<Point3<f32>>,
        _: &mut Vec<Point3<u32>>,
    ) {
    }

    fn gen_end_cap(
        &self,
        _: u32,
        _: &[Point3<f32>],
        _: &Point3<f32>,
        _: &Vector3<f32>,
        _: bool,
        _: &mut Vec<Point3<f32>>,
        _: &mut Vec<Point3<u32>>,
    ) {
    }
}
