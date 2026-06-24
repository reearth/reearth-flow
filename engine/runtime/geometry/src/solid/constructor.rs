//! Solid constructors.
//!
//! A `Solid`'s shells are already-constructed coordinate-free meshes
//! ([`PolygonMesh3DData`] / [`TriangularMesh3DData`]); these constructors just pair
//! an exterior shell (and any interior void shells) with the frame they are
//! expressed in. The `From` impls let a mesh be passed where a [`Shell`] is
//! expected.

use crate::coordinate::Coordinate;
use crate::polygon_mesh::PolygonMesh3DData;
use crate::triangular_mesh::TriangularMesh3DData;

use super::{Shell, Solid};

impl From<PolygonMesh3DData> for Shell {
    fn from(mesh: PolygonMesh3DData) -> Self {
        Shell::PolygonMesh(mesh)
    }
}

impl From<TriangularMesh3DData> for Shell {
    fn from(mesh: TriangularMesh3DData) -> Self {
        Shell::TriangularMesh(mesh)
    }
}

impl Solid {
    /// A solid bounded by `exterior` with the given interior (void) shells, in
    /// `coordinate`.
    pub fn new(coordinate: Coordinate, exterior: impl Into<Shell>, interiors: Vec<Shell>) -> Self {
        Self {
            coordinate,
            exterior: exterior.into(),
            interiors,
        }
    }

    /// A solid bounded by a single exterior shell, with no voids.
    pub fn from_exterior(coordinate: Coordinate, exterior: impl Into<Shell>) -> Self {
        Self::new(coordinate, exterior, Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cube_shell() -> TriangularMesh3DData {
        // A degenerate stand-in is fine here: construction does not validate closure.
        TriangularMesh3DData::from_parts(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            [0u32, 1, 2],
        )
        .unwrap()
    }

    #[test]
    fn from_exterior_has_no_voids() {
        let s = Solid::from_exterior(Coordinate::Euclidean, cube_shell());
        assert!(matches!(s.exterior, Shell::TriangularMesh(_)));
        assert!(s.interiors.is_empty());
        assert_eq!(s.coordinate, Coordinate::Euclidean);
    }

    #[test]
    fn new_carries_void_shells() {
        let s = Solid::new(
            Coordinate::Euclidean,
            cube_shell(),
            vec![Shell::from(cube_shell())],
        );
        assert_eq!(s.interiors.len(), 1);
    }
}
