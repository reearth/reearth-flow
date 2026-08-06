use super::{Point2D, Point3D};
use crate::coordinate::{CoordinateFrame, EpsgCode};
use crate::ops::{Aabb, BoundingBox, Reproject, ReprojectionCache, UnsupportedOperation};
use crate::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};

impl BoundingBox for Point2D {
    fn bounding_box(&self) -> Result<Aabb, UnsupportedOperation> {
        Ok(Aabb::point_2d(self.position))
    }
}

impl BoundingBox for Point3D {
    fn bounding_box(&self) -> Result<Aabb, UnsupportedOperation> {
        Ok(Aabb::point_3d(self.position))
    }
}

impl Point2D {
    /// Move the point out, leaving an empty husk.
    fn take(&mut self) -> Self {
        Self {
            frame: std::mem::take(&mut self.frame),
            position: std::mem::take(&mut self.position),
        }
    }

    /// The leaf moved out and wrapped as a [`Geometry`].
    fn take_geometry(&mut self) -> Geometry {
        Geometry::Euclidean2D(Euclidean2DGeometry::Point(self.take()))
    }
}

impl Point3D {
    /// Move the point out, leaving an empty husk.
    fn take(&mut self) -> Self {
        Self {
            frame: std::mem::take(&mut self.frame),
            position: std::mem::take(&mut self.position),
        }
    }

    /// The leaf moved out and wrapped as a [`Geometry`].
    fn take_geometry(&mut self) -> Geometry {
        Geometry::Euclidean3D(Euclidean3DGeometry::Point(self.take()))
    }
}

impl Reproject for Point2D {
    fn reproject(
        &mut self,
        target: EpsgCode,
        cache: &mut ReprojectionCache,
    ) -> crate::error::Result<Geometry> {
        let from = self.frame.require_crs()?;
        let mut p = self.take();
        if from != target {
            let [x, y] = p.position;
            let [nx, ny, _] = cache.transform(from, target, [x, y, 0.0])?;
            p.position = [nx, ny];
            p.frame = CoordinateFrame::Crs(target);
        }
        Ok(Geometry::Euclidean2D(Euclidean2DGeometry::Point(p)))
    }
}

impl Reproject for Point3D {
    fn reproject(
        &mut self,
        target: EpsgCode,
        cache: &mut ReprojectionCache,
    ) -> crate::error::Result<Geometry> {
        let from = self.frame.require_crs()?;
        let mut p = self.take();
        if from != target {
            p.position = cache.transform(from, target, p.position)?;
            p.frame = CoordinateFrame::Crs(target);
        }
        Ok(Geometry::Euclidean3D(Euclidean3DGeometry::Point(p)))
    }
}

use crate::ops::{plan_frame_step, Affine3, ConvertFrame, FrameStep, Place, Translate};

impl Translate for Point2D {
    fn translate(&mut self, delta: [f64; 3]) -> crate::error::Result<()> {
        self.position[0] += delta[0];
        self.position[1] += delta[1];
        Ok(())
    }
}

impl Translate for Point3D {
    fn translate(&mut self, delta: [f64; 3]) -> crate::error::Result<()> {
        self.position[0] += delta[0];
        self.position[1] += delta[1];
        self.position[2] += delta[2];
        Ok(())
    }
}

impl Place for Point3D {
    fn place(&mut self, affine: &Affine3, frame: &CoordinateFrame) -> crate::error::Result<()> {
        self.position = affine.apply(self.position);
        self.frame = frame.clone();
        Ok(())
    }
}

impl ConvertFrame for Point2D {
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

impl ConvertFrame for Point3D {
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

impl ForceTwoDimension for Point2D {
    fn force_2d(&mut self) -> Result<Euclidean2DGeometry, ForceTwoDimensionError> {
        // Already 2D and carries no elevation; hand back an equivalent point.
        Ok(Euclidean2DGeometry::Point(Point2D {
            frame: self.frame.demote_to_2d()?,
            position: self.position,
        }))
    }
}

impl ForceTwoDimension for Point3D {
    fn force_2d(&mut self) -> Result<Euclidean2DGeometry, ForceTwoDimensionError> {
        let frame = self.frame.demote_to_2d()?;
        let [x, y, _] = self.position;
        Ok(Euclidean2DGeometry::Point(Point2D {
            frame,
            position: [x, y],
        }))
    }
}

impl Point2D {
    /// The 3D counterpart of this leaf, with every coordinate placed at the
    /// elevation the leaf lies at, or at `0.0` when it carries none.
    pub(crate) fn into_3d(self) -> Point3D {
        let [x, y] = self.position;
        Point3D::new(self.frame, [x, y, 0.0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::CoordinateFrame;

    #[test]
    fn point2d_box_is_degenerate_d2() {
        let p = Point2D::new(CoordinateFrame::Euclidean, [1.0, 2.0]);
        assert_eq!(
            p.bounding_box().unwrap(),
            Aabb::D2 {
                min: [1.0, 2.0],
                max: [1.0, 2.0]
            }
        );
    }

    #[test]
    fn point3d_box_is_degenerate_d3() {
        let p = Point3D::new(CoordinateFrame::Euclidean, [1.0, 2.0, 3.0]);
        assert_eq!(
            p.bounding_box().unwrap(),
            Aabb::D3 {
                min: [1.0, 2.0, 3.0],
                max: [1.0, 2.0, 3.0]
            }
        );
    }
}
