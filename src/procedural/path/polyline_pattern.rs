use crate::procedural::path::{CurveSampler, PathSample, StrokePattern};
use crate::procedural::render_mesh::{IndexBuffer, RenderMesh};
use crate::procedural::utils;
use na::{self, Isometry3, Point2, Point3, Vector3};

/// A pattern composed of polyline and two caps.
pub struct PolylinePattern<C1, C2> {
    pattern: Vec<Point3<f32>>,
    closed: bool,
    last_start_id: u32,
    start_cap: C1,
    end_cap: C2,
}

/// Trait to be implemented by caps compatible with a `PolylinePattern`.
pub trait PolylineCompatibleCap {
    /// Generates the mesh for the cap at the beginning of a path.
    fn gen_start_cap(
        &self,
        attach_id: u32,
        pattern: &[Point3<f32>],
        pt: &Point3<f32>,
        dir: &Vector3<f32>,
        closed: bool,
        coords: &mut Vec<Point3<f32>>,
        indices: &mut Vec<Point3<u32>>,
    );

    /// Generates the mesh for the cap at the end of a path.
    fn gen_end_cap(
        &self,
        attach_id: u32,
        pattern: &[Point3<f32>],
        pt: &Point3<f32>,
        dir: &Vector3<f32>,
        closed: bool,
        coords: &mut Vec<Point3<f32>>,
        indices: &mut Vec<Point3<u32>>,
    );
}

impl<C1, C2> PolylinePattern<C1, C2>
where
    C1: PolylineCompatibleCap,
    C2: PolylineCompatibleCap,
{
    /// Creates a new polyline pattern.
    pub fn new(
        pattern: &[Point2<f32>],
        closed: bool,
        start_cap: C1,
        end_cap: C2,
    ) -> PolylinePattern<C1, C2> {
        let mut coords3d = Vec::with_capacity(pattern.len());

        for v in pattern.iter() {
            coords3d.push(Point3::new(v.x, v.y, na::zero()));
        }

        PolylinePattern {
            pattern: coords3d,
            closed,
            last_start_id: 0,
            start_cap,
            end_cap,
        }
    }
}

impl<C1, C2> StrokePattern for PolylinePattern<C1, C2>
where
    C1: PolylineCompatibleCap,
    C2: PolylineCompatibleCap,
{
    fn stroke<C: CurveSampler>(&mut self, sampler: &mut C) -> RenderMesh {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let npts = self.pattern.len() as u32;
        // FIXME: collect the normals too.
        // let mut normals  = Vec::new();

        loop {
            let next = sampler.next();

            // second match to add the inner triangles.
            match next {
                PathSample::StartPoint(ref pt, ref dir)
                | PathSample::InnerPoint(ref pt, ref dir)
                | PathSample::EndPoint(ref pt, ref dir) => {
                    let mut new_polyline = self.pattern.clone();

                    let transform = if dir.x == 0.0 && dir.z == 0.0 {
                        // FIXME: this might not be enough to avoid singularities.
                        Isometry3::face_towards(pt, &(*pt + *dir), &Vector3::x())
                    } else {
                        Isometry3::face_towards(pt, &(*pt + *dir), &Vector3::y())
                    };

                    for p in &mut new_polyline {
                        *p = transform * *p;
                    }

                    let new_start_id = vertices.len() as u32;

                    vertices.extend(new_polyline);

                    if new_start_id != 0 {
                        if self.closed {
                            utils::push_ring_indices(
                                new_start_id,
                                self.last_start_id,
                                npts,
                                &mut indices,
                            );
                        } else {
                            utils::push_open_ring_indices(
                                new_start_id,
                                self.last_start_id,
                                npts,
                                &mut indices,
                            );
                        }

                        self.last_start_id = new_start_id;
                    }
                }
                PathSample::EndOfSample => {
                    return RenderMesh::new(
                        vertices,
                        None,
                        None,
                        Some(IndexBuffer::Unified(indices)),
                    )
                }
            }

            // third match to add the end cap
            // FIXME: this will fail with patterns having multiple starting and end points!
            match next {
                PathSample::StartPoint(ref pt, ref dir) => {
                    self.start_cap.gen_start_cap(
                        0,
                        &self.pattern,
                        pt,
                        dir,
                        self.closed,
                        &mut vertices,
                        &mut indices,
                    );
                }
                PathSample::EndPoint(ref pt, ref dir) => {
                    self.end_cap.gen_end_cap(
                        vertices.len() as u32 - npts,
                        &self.pattern,
                        pt,
                        dir,
                        self.closed,
                        &mut vertices,
                        &mut indices,
                    );
                }
                _ => {}
            }
        }
    }
}
