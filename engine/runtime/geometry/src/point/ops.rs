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
