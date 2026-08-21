use super::{LineString2D, LineString3D};
use crate::coordinate::{CoordinateFrame, EpsgCode};
use crate::ops::reproject::{transform_coords_2d, transform_coords_3d};
use crate::ops::{
    lift_coords, Aabb, BoundingBox, Reproject, ReprojectionCache, UnsupportedOperation,
};
use crate::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};

impl BoundingBox for LineString2D {
    fn bounding_box(&self) -> Result<Aabb, UnsupportedOperation> {
        // 2D embedding: the optional elevation is not folded in.
        Aabb::from_points_2d(self.coords.iter().copied()).ok_or(UnsupportedOperation {
            geometry: "LineString2D",
            operation: "bounding_box",
        })
    }
}

impl BoundingBox for LineString3D {
    fn bounding_box(&self) -> Result<Aabb, UnsupportedOperation> {
        Aabb::from_points_3d(self.coords.iter().copied()).ok_or(UnsupportedOperation {
            geometry: "LineString3D",
            operation: "bounding_box",
        })
    }
}

impl LineString2D {
    /// Move the chain out, leaving an empty husk.
    fn take(&mut self) -> Self {
        Self {
            frame: std::mem::take(&mut self.frame),
            coords: std::mem::take(&mut self.coords),
            z: self.z.take(),
        }
    }

    /// The leaf moved out and wrapped as a [`Geometry`].
    fn take_geometry(&mut self) -> Geometry {
        Geometry::Euclidean2D(Euclidean2DGeometry::LineString(self.take()))
    }
}

impl LineString3D {
    /// Move the chain out, leaving an empty husk.
    fn take(&mut self) -> Self {
        Self {
            frame: std::mem::take(&mut self.frame),
            coords: std::mem::take(&mut self.coords),
        }
    }

    /// The leaf moved out and wrapped as a [`Geometry`].
    fn take_geometry(&mut self) -> Geometry {
        Geometry::Euclidean3D(Euclidean3DGeometry::LineString(self.take()))
    }
}

impl Reproject for LineString2D {
    fn reproject(
        &mut self,
        target: EpsgCode,
        cache: &mut ReprojectionCache,
    ) -> crate::error::Result<Geometry> {
        let from = self.frame.require_crs()?;
        if from == target {
            return Ok(self.take_geometry());
        }
        if self.z.is_some() {
            return self.take().into_3d().reproject(target, cache);
        }
        let mut ls = self.take();
        transform_coords_2d(cache, from, target, &mut ls.coords)?;
        ls.frame = CoordinateFrame::Crs(target);
        Ok(Geometry::Euclidean2D(Euclidean2DGeometry::LineString(ls)))
    }
}

impl Reproject for LineString3D {
    fn reproject(
        &mut self,
        target: EpsgCode,
        cache: &mut ReprojectionCache,
    ) -> crate::error::Result<Geometry> {
        let from = self.frame.require_crs()?;
        let mut ls = self.take();
        if from != target {
            transform_coords_3d(cache, from, target, &mut ls.coords)?;
            ls.frame = CoordinateFrame::Crs(target);
        }
        Ok(Geometry::Euclidean3D(Euclidean3DGeometry::LineString(ls)))
    }
}

use crate::ops::{plan_frame_step, translate_2d, translate_3d, ConvertFrame, FrameStep, Translate};

impl Translate for LineString2D {
    fn translate(&mut self, delta: [f64; 3]) -> crate::error::Result<()> {
        translate_2d(&mut self.coords, &mut self.z, delta);
        Ok(())
    }
}

impl Translate for LineString3D {
    fn translate(&mut self, delta: [f64; 3]) -> crate::error::Result<()> {
        translate_3d(&mut self.coords, delta);
        Ok(())
    }
}

impl ConvertFrame for LineString2D {
    fn convert_frame(
        &mut self,
        target: &CoordinateFrame,
        base_point: Option<[f64; 3]>,
        cache: &mut ReprojectionCache,
    ) -> crate::error::Result<Geometry> {
        match plan_frame_step(&self.frame, target, base_point)? {
            FrameStep::Noop => Ok(self.take_geometry()),
            FrameStep::Reproject(to) => self.reproject(to, cache),
            FrameStep::Translate(offset, frame) => {
                self.translate(offset)?;
                self.frame = frame;
                Ok(self.take_geometry())
            }
        }
    }
}

impl ConvertFrame for LineString3D {
    fn convert_frame(
        &mut self,
        target: &CoordinateFrame,
        base_point: Option<[f64; 3]>,
        cache: &mut ReprojectionCache,
    ) -> crate::error::Result<Geometry> {
        match plan_frame_step(&self.frame, target, base_point)? {
            FrameStep::Noop => Ok(self.take_geometry()),
            FrameStep::Reproject(to) => self.reproject(to, cache),
            FrameStep::Translate(offset, frame) => {
                self.translate(offset)?;
                self.frame = frame;
                Ok(self.take_geometry())
            }
        }
    }
}

use crate::ops::{ForceTwoDimension, ForceTwoDimensionError};

impl ForceTwoDimension for LineString2D {
    fn force_2d(&mut self) -> Result<Euclidean2DGeometry, ForceTwoDimensionError> {
        let frame = self.frame.demote_to_2d()?;
        self.z = None; // drop any 2.5D elevation; already 2D otherwise
        Ok(Euclidean2DGeometry::LineString(LineString2D {
            frame,
            coords: std::mem::take(&mut self.coords),
            z: None,
        }))
    }
}

impl ForceTwoDimension for LineString3D {
    fn force_2d(&mut self) -> Result<Euclidean2DGeometry, ForceTwoDimensionError> {
        let frame = self.frame.demote_to_2d()?;
        let coords = std::mem::take(&mut self.coords)
            .iter()
            .map(|&[x, y, _]| [x, y])
            .collect();
        Ok(Euclidean2DGeometry::LineString(LineString2D {
            frame,
            coords,
            z: None,
        }))
    }
}

impl LineString2D {
    /// The 3D counterpart of this leaf, with every coordinate placed at the
    /// elevation the leaf lies at, or at `0.0` when it carries none.
    pub(crate) fn into_3d(self) -> LineString3D {
        LineString3D {
            frame: self.frame,
            coords: lift_coords(self.coords.iter(), self.z).into_boxed_slice(),
        }
    }
}

use crate::ops::coerce::{closes_a_ring, unchanged};
use crate::ops::triangulation::Cache;
use crate::ops::{Coerce, CoercionTarget};
use crate::polygon::{Polygon2D, Polygon3D};

impl Coerce for LineString2D {
    fn coerce(
        &mut self,
        target: CoercionTarget,
        _cache: &mut Cache,
    ) -> Result<Geometry, UnsupportedOperation> {
        match target {
            // A curve already is one, and bounds no area to tessellate.
            CoercionTarget::LineString | CoercionTarget::TriangularMesh => Err(unchanged::<Self>()),
            CoercionTarget::Polygon => {
                if !closes_a_ring(&self.coords) {
                    return Err(unchanged::<Self>());
                }
                let ring = Vec::from(std::mem::take(&mut self.coords));
                let no_holes = Vec::<Vec<[f64; 2]>>::new();
                let face = match self.z.take() {
                    None => Polygon2D::from_rings(self.frame.clone(), ring, no_holes),
                    Some(elevation) => Polygon2D::from_rings_at_elevation(
                        self.frame.clone(),
                        ring,
                        no_holes,
                        elevation,
                    ),
                };
                Ok(Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(
                    Box::new(face),
                )))
            }
        }
    }
}

impl Coerce for LineString3D {
    fn coerce(
        &mut self,
        target: CoercionTarget,
        _cache: &mut Cache,
    ) -> Result<Geometry, UnsupportedOperation> {
        match target {
            // A curve already is one, and bounds no area to tessellate.
            CoercionTarget::LineString | CoercionTarget::TriangularMesh => Err(unchanged::<Self>()),
            CoercionTarget::Polygon => {
                if !closes_a_ring(&self.coords) {
                    return Err(unchanged::<Self>());
                }
                let ring = Vec::from(std::mem::take(&mut self.coords));
                let face =
                    Polygon3D::from_rings(self.frame.clone(), ring, Vec::<Vec<[f64; 3]>>::new());
                Ok(Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(
                    Box::new(face),
                )))
            }
        }
    }
}

#[cfg(feature = "new-geometry")]
use crate::ops::{Footprint, FootprintError, FootprintSink};

#[cfg(feature = "new-geometry")]
impl Footprint for LineString2D {
    fn footprint(&self, sink: &mut FootprintSink<'_>) -> Result<(), FootprintError> {
        sink.enter(self.frame())?;
        sink.push_curve_2d(self.coords(), self.elevation());
        Ok(())
    }
}

#[cfg(feature = "new-geometry")]
impl Footprint for LineString3D {
    fn footprint(&self, sink: &mut FootprintSink<'_>) -> Result<(), FootprintError> {
        sink.enter(self.frame())?;
        sink.push_curve_3d(self.coords());
        Ok(())
    }
}

use crate::collection::{Collection2D, Collection3D};
use crate::ops::boundary::{endpoints, Boundary, ExtractBoundary};
use crate::point::{Point2D, Point3D};

// A chain is bounded by its two ends. One that closes on itself has no ends, so
// its boundary is empty.
//
// The ends come back as bare points, which carry no elevation of their own, so a
// 2.5D chain's elevation is not preserved onto them.
impl ExtractBoundary for LineString2D {
    fn extract_boundary(&self) -> Result<Boundary, UnsupportedOperation> {
        let Some((first, last)) = endpoints(self.coords()) else {
            return Ok(Boundary::EMPTY);
        };
        let frame = self.frame();
        Ok(Boundary::Bounded(Geometry::Euclidean2D(
            Euclidean2DGeometry::Collection(Collection2D::new([
                Euclidean2DGeometry::Point(Point2D::new(frame.clone(), first)),
                Euclidean2DGeometry::Point(Point2D::new(frame.clone(), last)),
            ])),
        )))
    }
}

impl ExtractBoundary for LineString3D {
    fn extract_boundary(&self) -> Result<Boundary, UnsupportedOperation> {
        let Some((first, last)) = endpoints(self.coords()) else {
            return Ok(Boundary::EMPTY);
        };
        let frame = self.frame();
        Ok(Boundary::Bounded(Geometry::Euclidean3D(
            Euclidean3DGeometry::Collection(Collection3D::new([
                Euclidean3DGeometry::Point(Point3D::new(frame.clone(), first)),
                Euclidean3DGeometry::Point(Point3D::new(frame.clone(), last)),
            ])),
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::CoordinateFrame;

    #[test]
    fn linestring2d_box_spans_all_coords() {
        let ls = LineString2D::from_coords(
            CoordinateFrame::Euclidean,
            [[0.0, 0.0], [2.0, 1.0], [1.0, 3.0]],
        );
        assert_eq!(
            ls.bounding_box().unwrap(),
            Aabb::D2 {
                min: [0.0, 0.0],
                max: [2.0, 3.0]
            }
        );
    }

    #[test]
    fn linestring2d_box_ignores_elevation() {
        let ls = LineString2D::from_coords_at_elevation(
            CoordinateFrame::Euclidean,
            [[0.0, 0.0], [2.0, 1.0]],
            99.0,
        );
        // 2.5D elevation does not widen the 2D box.
        assert_eq!(
            ls.bounding_box().unwrap(),
            Aabb::D2 {
                min: [0.0, 0.0],
                max: [2.0, 1.0]
            }
        );
    }

    #[test]
    fn empty_linestring_has_no_box() {
        let ls = LineString2D::from_coords(CoordinateFrame::Euclidean, Vec::<[f64; 2]>::new());
        assert!(ls.bounding_box().is_err());
    }

    #[test]
    fn linestring3d_box_spans_all_coords() {
        let ls = LineString3D::from_coords(
            CoordinateFrame::Euclidean,
            [[0.0, 0.0, 0.0], [2.0, 1.0, -1.0]],
        );
        assert_eq!(
            ls.bounding_box().unwrap(),
            Aabb::D3 {
                min: [0.0, 0.0, -1.0],
                max: [2.0, 1.0, 0.0]
            }
        );
    }
}
