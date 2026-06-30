use super::{TriangularMesh2D, TriangularMesh3D};
use crate::coordinate::{Coordinate, EpsgCode};
use crate::ops::reproject::{transform_coords_2d, transform_coords_3d};
use crate::ops::{Aabb, BoundingBox, Reproject, ReprojectionCache, UnsupportedOperation};

impl BoundingBox for TriangularMesh2D {
    fn bounding_box(&self) -> Result<Aabb, UnsupportedOperation> {
        Aabb::from_points_2d(self.vertices.iter().copied()).ok_or(UnsupportedOperation {
            geometry: "TriangularMesh2D",
            operation: "bounding_box",
        })
    }
}

impl BoundingBox for TriangularMesh3D {
    fn bounding_box(&self) -> Result<Aabb, UnsupportedOperation> {
        Aabb::from_points_3d(self.data.vertices.iter().copied()).ok_or(UnsupportedOperation {
            geometry: "TriangularMesh3D",
            operation: "bounding_box",
        })
    }
}

impl Reproject for TriangularMesh2D {
    fn reproject(
        &mut self,
        target: EpsgCode,
        cache: &mut ReprojectionCache,
    ) -> crate::error::Result<()> {
        let from = self.coordinate.require_crs()?;
        if from != target {
            transform_coords_2d(
                cache,
                from,
                target,
                &mut self.vertices,
                self.z.as_deref_mut(),
            )?;
            self.coordinate = Coordinate::Crs(target);
        }
        Ok(())
    }
}

impl Reproject for TriangularMesh3D {
    fn reproject(
        &mut self,
        target: EpsgCode,
        cache: &mut ReprojectionCache,
    ) -> crate::error::Result<()> {
        let from = self.coordinate.require_crs()?;
        if from != target {
            transform_coords_3d(cache, from, target, self.data.vertices_mut())?;
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
    fn triangular_mesh2d_box() {
        let m = TriangularMesh2D::from_soup(
            Coordinate::Euclidean,
            [[0.0, 0.0], [3.0, 0.0], [3.0, 2.0]],
        );
        assert_eq!(
            m.bounding_box().unwrap(),
            Aabb::D2 {
                min: [0.0, 0.0],
                max: [3.0, 2.0]
            }
        );
    }

    #[test]
    fn triangular_mesh3d_box() {
        let m = TriangularMesh3D::from_soup(
            Coordinate::Euclidean,
            [[0.0, 0.0, 0.0], [3.0, 0.0, 1.0], [3.0, 2.0, -1.0]],
        );
        assert_eq!(
            m.bounding_box().unwrap(),
            Aabb::D3 {
                min: [0.0, 0.0, -1.0],
                max: [3.0, 2.0, 1.0]
            }
        );
    }
}
