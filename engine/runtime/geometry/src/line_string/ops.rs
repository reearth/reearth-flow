use super::{LineString2D, LineString3D};
use crate::coordinate::{Coordinate, EpsgCode};
use crate::ops::reproject::{transform_coords_2d, transform_coords_3d};
use crate::ops::{Aabb, BoundingBox, Reproject, ReprojectionCache, UnsupportedOperation};

impl BoundingBox for LineString2D {
    fn bounding_box(&self) -> Result<Aabb, UnsupportedOperation> {
        // 2D embedding: the optional per-vertex elevation is not folded in.
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

impl Reproject for LineString2D {
    fn reproject(
        &mut self,
        target: EpsgCode,
        cache: &mut ReprojectionCache,
    ) -> crate::error::Result<()> {
        let from = self.coordinate.require_crs()?;
        if from != target {
            transform_coords_2d(cache, from, target, &mut self.coords, self.z.as_deref_mut())?;
            self.coordinate = Coordinate::Crs(target);
        }
        Ok(())
    }
}

impl Reproject for LineString3D {
    fn reproject(
        &mut self,
        target: EpsgCode,
        cache: &mut ReprojectionCache,
    ) -> crate::error::Result<()> {
        let from = self.coordinate.require_crs()?;
        if from != target {
            transform_coords_3d(cache, from, target, &mut self.coords)?;
            self.coordinate = Coordinate::Crs(target);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::Coordinate;

    #[test]
    fn linestring2d_box_spans_all_coords() {
        let ls =
            LineString2D::from_coords(Coordinate::Euclidean, [[0.0, 0.0], [2.0, 1.0], [1.0, 3.0]]);
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
        let ls = LineString2D::from_coords_with_elevation(
            Coordinate::Euclidean,
            [[0.0, 0.0, 99.0], [2.0, 1.0, -99.0]],
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
        let ls = LineString2D::from_coords(Coordinate::Euclidean, Vec::<[f64; 2]>::new());
        assert!(ls.bounding_box().is_err());
    }

    #[test]
    fn linestring3d_box_spans_all_coords() {
        let ls =
            LineString3D::from_coords(Coordinate::Euclidean, [[0.0, 0.0, 0.0], [2.0, 1.0, -1.0]]);
        assert_eq!(
            ls.bounding_box().unwrap(),
            Aabb::D3 {
                min: [0.0, 0.0, -1.0],
                max: [2.0, 1.0, 0.0]
            }
        );
    }
}
