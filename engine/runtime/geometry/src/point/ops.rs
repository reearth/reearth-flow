use super::{Point2D, Point3D};
use crate::coordinate::{CoordinateFrame, EpsgCode};
use crate::ops::{Aabb, BoundingBox, Reproject, ReprojectionCache, UnsupportedOperation};

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

impl Reproject for Point2D {
    fn reproject(
        &mut self,
        target: EpsgCode,
        cache: &mut ReprojectionCache,
    ) -> crate::error::Result<()> {
        let from = self.frame.require_crs()?;
        if from != target {
            let [x, y] = self.position;
            let [nx, ny, _] = cache.transform(from, target, [x, y, 0.0])?;
            self.position = [nx, ny];
            self.frame = CoordinateFrame::Crs(target);
        }
        Ok(())
    }
}

impl Reproject for Point3D {
    fn reproject(
        &mut self,
        target: EpsgCode,
        cache: &mut ReprojectionCache,
    ) -> crate::error::Result<()> {
        let from = self.frame.require_crs()?;
        if from != target {
            self.position = cache.transform(from, target, self.position)?;
            self.frame = CoordinateFrame::Crs(target);
        }
        Ok(())
    }
}

use crate::ops::{plan_frame_step, ConvertFrame, FrameStep, Translate};

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

impl ConvertFrame for Point2D {
    fn convert_frame(
        &mut self,
        target: &CoordinateFrame,
        base_point: Option<[f64; 3]>,
        cache: &mut ReprojectionCache,
    ) -> crate::error::Result<()> {
        match plan_frame_step(&self.frame, target, base_point)? {
            FrameStep::Noop => Ok(()),
            FrameStep::Reproject(to) => self.reproject(to, cache),
            FrameStep::Translate(offset, frame) => {
                self.translate(offset)?;
                self.frame = frame;
                Ok(())
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
    ) -> crate::error::Result<()> {
        match plan_frame_step(&self.frame, target, base_point)? {
            FrameStep::Noop => Ok(()),
            FrameStep::Reproject(to) => self.reproject(to, cache),
            FrameStep::Translate(offset, frame) => {
                self.translate(offset)?;
                self.frame = frame;
                Ok(())
            }
        }
    }
}

use crate::ops::ForceTwoDimension;
use crate::Euclidean2DGeometry;

impl ForceTwoDimension for Point2D {
    fn force_2d(&mut self) -> Result<Euclidean2DGeometry, UnsupportedOperation> {
        // Already 2D and carries no elevation; hand back an equivalent point.
        Ok(Euclidean2DGeometry::Point(Point2D {
            frame: self.frame.clone(),
            position: self.position,
        }))
    }
}

impl ForceTwoDimension for Point3D {
    fn force_2d(&mut self) -> Result<Euclidean2DGeometry, UnsupportedOperation> {
        let [x, y, _] = self.position;
        Ok(Euclidean2DGeometry::Point(Point2D {
            frame: self.frame.clone(),
            position: [x, y],
        }))
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
