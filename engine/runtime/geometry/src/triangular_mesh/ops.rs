use super::{TriangularMesh2D, TriangularMesh3D};
use crate::ops::{Aabb, BoundingBox, UnsupportedOperation};

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
