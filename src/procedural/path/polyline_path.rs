use crate::procedural::path::{CurveSampler, PathSample};
use na::{Point3, RealField, Vector3};

/// A path with its sample points given by a polyline.
///
/// This will return sequencially each vertex of the polyline.
pub struct PolylinePath<'a> {
    curr_len: f32,
    curr_dir: Vector3<f32>,
    curr_pt_id: usize,
    curr_pt: Point3<f32>,
    polyline: &'a [Point3<f32>],
}

impl<'a> PolylinePath<'a> {
    /// Creates a new polyline-based path.
    pub fn new(polyline: &'a [Point3<f32>]) -> PolylinePath<'a> {
        assert!(
            polyline.len() > 1,
            "The polyline must have at least two points."
        );

        let mut dir = polyline[1] - polyline[0];
        let len: f32 = dir.normalize_mut();

        PolylinePath {
            curr_len: len,
            curr_dir: dir,
            curr_pt_id: 0,
            curr_pt: polyline[0].clone(),
            polyline,
        }
    }
}

impl CurveSampler for PolylinePath<'_> {
    fn next(&mut self) -> PathSample {
        let poly_coords = self.polyline;

        let result = if self.curr_pt_id == 0 {
            PathSample::StartPoint(self.curr_pt.clone(), self.curr_dir.clone())
        } else if self.curr_pt_id < poly_coords.len() - 1 {
            PathSample::InnerPoint(self.curr_pt.clone(), self.curr_dir.clone())
        } else if self.curr_pt_id == poly_coords.len() - 1 {
            PathSample::EndPoint(self.curr_pt.clone(), self.curr_dir.clone())
        } else {
            PathSample::EndOfSample
        };

        self.curr_pt_id = self.curr_pt_id + 1;

        if self.curr_pt_id < poly_coords.len() {
            self.curr_pt = poly_coords[self.curr_pt_id].clone();

            if self.curr_pt_id < poly_coords.len() - 1 {
                let mut curr_diff = poly_coords[self.curr_pt_id + 1] - poly_coords[self.curr_pt_id];
                self.curr_len = curr_diff.normalize_mut();
                self.curr_dir = curr_diff;
            }
        }

        result
    }
}
