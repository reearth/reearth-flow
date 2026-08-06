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

use crate::ops::{
    apply_affine_3d, plan_frame_step, translate_2d, translate_3d, Affine3, ConvertFrame, FrameStep,
    Place, Translate,
};

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

impl Place for LineString3D {
    fn place(&mut self, affine: &Affine3, frame: &CoordinateFrame) -> crate::error::Result<()> {
        apply_affine_3d(&mut self.coords, affine);
        self.frame = frame.clone();
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
