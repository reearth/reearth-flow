use super::Solid;
use crate::ops::{Aabb, BoundingBox, UnsupportedOperation};

impl BoundingBox for Solid {
    fn bounding_box(&self) -> Result<Aabb, UnsupportedOperation> {
        // The exterior shell encloses the interior (void) shells, so the box
        // over the exterior alone already bounds the solid; iterating the
        // interiors too costs nothing and stays correct for ill-formed solids.
        let verts = std::iter::once(&self.exterior)
            .chain(self.interiors.iter())
            .flat_map(|s| s.vertices().iter().copied());
        Aabb::from_points_3d(verts).ok_or(UnsupportedOperation {
            geometry: "Solid",
            operation: "bounding_box",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::Coordinate;
    use crate::solid::Shell;
    use crate::triangular_mesh::TriangularMesh3DData;

    fn shell(verts: Vec<[f64; 3]>) -> TriangularMesh3DData {
        TriangularMesh3DData::from_parts(verts, [0u32, 1, 2]).unwrap()
    }

    #[test]
    fn solid_box_spans_exterior_shell() {
        let s = Solid::from_exterior(
            Coordinate::Euclidean,
            shell(vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 3.0]]),
        );
        assert_eq!(
            s.bounding_box().unwrap(),
            Aabb::D3 {
                min: [0.0, 0.0, 0.0],
                max: [2.0, 2.0, 3.0]
            }
        );
    }

    #[test]
    fn solid_box_includes_interior_shells() {
        let s = Solid::new(
            Coordinate::Euclidean,
            shell(vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]]),
            vec![Shell::from(shell(vec![
                [5.0, 5.0, 5.0],
                [6.0, 5.0, 5.0],
                [5.0, 6.0, 5.0],
            ]))],
        );
        assert_eq!(
            s.bounding_box().unwrap(),
            Aabb::D3 {
                min: [0.0, 0.0, 0.0],
                max: [6.0, 6.0, 5.0]
            }
        );
    }
}
