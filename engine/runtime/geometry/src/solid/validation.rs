use super::{Shell, Solid};
use crate::coordinate::CoordinateFrame;
use crate::polygon_mesh::PolygonMesh3D;
use crate::triangular_mesh::TriangularMesh3D;
use crate::validation_next::{
    check_duplicate_points_3d, check_edge_orientation_3d, check_finite_3d, CheckOutcome, Validate,
    ValidationReport, ValidationType,
};
use crate::{Euclidean3DGeometry, Geometry};

impl Shell {
    /// Whether this shell admits a consistent orientation.
    fn is_orientable(&self) -> bool {
        match self {
            Shell::PolygonMesh(data) => data.is_orientable(),
            Shell::TriangularMesh(data) => data.is_orientable(),
        }
    }

    /// Report each face ring that winds inconsistently with a neighbour across a
    /// shared edge.
    fn check_orientation(&self, frame: &CoordinateFrame, report: &mut ValidationReport) {
        match self {
            Shell::PolygonMesh(data) => data.check_orientation(frame, report),
            Shell::TriangularMesh(data) => {
                check_edge_orientation_3d(frame, data.vertices(), data.triangles(), report)
            }
        }
    }

    /// Whether this shell is a watertight closed 2-manifold (single connected
    /// component, every edge shared by exactly two faces).
    fn is_closed_connected_manifold(&self) -> bool {
        match self {
            Shell::PolygonMesh(data) => data.is_closed_connected_manifold(),
            Shell::TriangularMesh(data) => data.is_closed_connected_manifold(),
        }
    }

    /// The signed volume the shell encloses, taken as a closed surface; its sign
    /// follows the shell's orientation (positive = outward normals).
    fn signed_volume(&self) -> f64 {
        match self {
            Shell::PolygonMesh(data) => data.signed_volume(),
            Shell::TriangularMesh(data) => data.signed_volume(),
        }
    }

    /// The shell as a standalone geometry, for use as a problem position.
    fn to_geometry(&self, frame: &CoordinateFrame) -> Geometry {
        match self {
            Shell::PolygonMesh(data) => Geometry::Euclidean3D(Euclidean3DGeometry::PolygonMesh(
                Box::new(PolygonMesh3D::new(frame.clone(), data.clone())),
            )),
            Shell::TriangularMesh(data) => {
                Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(Box::new(
                    TriangularMesh3D::new(frame.clone(), data.clone()),
                )))
            }
        }
    }
}

/// The checks that apply to a solid. `InteriorRingContainment` does not apply: a
/// solid's interiors are void *shells*, not interior rings.
const SOLID_CHECKS: [ValidationType; 10] = [
    ValidationType::Finite,
    ValidationType::TooFewPoints,
    ValidationType::UnclosedRing,
    ValidationType::SelfIntersection,
    ValidationType::Degenerate,
    ValidationType::DuplicatePoints,
    ValidationType::Orientation,
    ValidationType::Orientable,
    ValidationType::ShellManifold,
    ValidationType::ShellOrientation,
];

impl Solid {
    /// Each shell (exterior, then voids) of the solid.
    fn shells(&self) -> impl Iterator<Item = &Shell> {
        std::iter::once(self.exterior()).chain(self.interiors())
    }
}

// A solid validates by its shells' vertex pools. The indices refer into these
// vertices, so scanning each shell's pool covers the whole boundary.
impl Validate for Solid {
    fn applicable_checks(&self) -> &'static [ValidationType] {
        &SOLID_CHECKS
    }

    fn check_finite(&self) -> CheckOutcome {
        CheckOutcome::ran(|r| {
            for shell in self.shells() {
                check_finite_3d(&self.frame, shell.vertices().iter().copied(), r);
            }
        })
    }

    fn check_too_few_points(&self) -> CheckOutcome {
        CheckOutcome::ran(|r| {
            for shell in self.shells() {
                // A triangle shell's faces always have three corners; only a
                // polygon shell can carry a degenerate ring.
                if let Shell::PolygonMesh(data) = shell {
                    data.check_too_few_points(&self.frame, r);
                }
            }
        })
    }

    fn check_unclosed_ring(&self) -> CheckOutcome {
        CheckOutcome::ran(|r| {
            for shell in self.shells() {
                // A triangle shell's faces are implicitly closed; only a polygon
                // shell stores an explicit closing vertex.
                if let Shell::PolygonMesh(data) = shell {
                    data.check_unclosed_rings(&self.frame, r);
                }
            }
        })
    }

    fn check_duplicate_points(&self) -> CheckOutcome {
        CheckOutcome::ran(|r| {
            for shell in self.shells() {
                check_duplicate_points_3d(&self.frame, shell.vertices().iter().copied(), None, r);
            }
        })
    }

    fn check_orientation(&self) -> CheckOutcome {
        // Each shell must be consistently wound across its shared edges.
        CheckOutcome::ran(|r| {
            for shell in self.shells() {
                shell.check_orientation(&self.frame, r);
            }
        })
    }

    fn check_orientable(&self) -> CheckOutcome {
        // Each shell must admit a consistent orientation.
        CheckOutcome::ran(|r| {
            for shell in self.shells() {
                if !shell.is_orientable() {
                    r.push(
                        ValidationType::Orientable.to_string(),
                        shell.to_geometry(&self.frame),
                    );
                }
            }
        })
    }

    fn check_shell_manifold(&self) -> CheckOutcome {
        // Each shell must be a watertight closed connected 2-manifold.
        CheckOutcome::ran(|r| {
            for shell in self.shells() {
                if !shell.is_closed_connected_manifold() {
                    r.push(
                        ValidationType::ShellManifold.to_string(),
                        shell.to_geometry(&self.frame),
                    );
                }
            }
        })
    }

    fn check_shell_orientation(&self) -> CheckOutcome {
        // The exterior shell must enclose positive volume (outward normals); each
        // void shell negative volume (normals into the void).
        CheckOutcome::ran(|r| {
            if self.exterior().signed_volume() <= 0.0 {
                r.push(
                    ValidationType::ShellOrientation.to_string(),
                    self.exterior().to_geometry(&self.frame),
                );
            }
            for void in self.interiors() {
                if void.signed_volume() >= 0.0 {
                    r.push(
                        ValidationType::ShellOrientation.to_string(),
                        void.to_geometry(&self.frame),
                    );
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::triangular_mesh::TriangularMesh3DData;
    use crate::validation_next::{validate_one, ValidationResult};

    fn tetra_verts() -> Vec<[f64; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ]
    }

    /// A closed tetrahedron shell with outward-facing (positive-volume) winding.
    fn tetra_outward() -> TriangularMesh3DData {
        TriangularMesh3DData::from_parts(tetra_verts(), [1u32, 2, 3, 0, 3, 2, 0, 1, 3, 0, 2, 1])
            .unwrap()
    }

    /// The same tetrahedron with every face reversed: inward-facing (negative
    /// volume).
    fn tetra_inward() -> TriangularMesh3DData {
        TriangularMesh3DData::from_parts(tetra_verts(), [1u32, 3, 2, 0, 2, 3, 0, 3, 1, 0, 1, 2])
            .unwrap()
    }

    // Each helper runs just `check` (and its prerequisites) on the solid, not the
    // solid's other, still-unimplemented checks.
    fn is_success(s: &Solid, check: ValidationType) -> bool {
        validate_one(s, check) == ValidationResult::Success
    }

    fn failure_count(s: &Solid, check: ValidationType) -> usize {
        match validate_one(s, check) {
            ValidationResult::Failed(positions) => positions.len(),
            other => panic!("expected {check} to fail, got {other:?}"),
        }
    }

    #[test]
    fn closed_tetra_is_a_shell_manifold() {
        let s = Solid::from_exterior(CoordinateFrame::Euclidean, tetra_outward());
        assert!(is_success(&s, ValidationType::ShellManifold));
    }

    #[test]
    fn open_shell_is_not_a_manifold() {
        // A single triangle has boundary edges, so it is not watertight.
        let open = TriangularMesh3DData::from_parts(tetra_verts(), [0u32, 1, 2]).unwrap();
        let s = Solid::from_exterior(CoordinateFrame::Euclidean, open);
        assert_eq!(failure_count(&s, ValidationType::ShellManifold), 1);
    }

    #[test]
    fn tetra_shell_is_orientable() {
        let s = Solid::from_exterior(CoordinateFrame::Euclidean, tetra_outward());
        assert!(is_success(&s, ValidationType::Orientable));
    }

    #[test]
    fn outward_exterior_passes_shell_orientation() {
        let s = Solid::from_exterior(CoordinateFrame::Euclidean, tetra_outward());
        assert!(is_success(&s, ValidationType::ShellOrientation));
    }

    #[test]
    fn inward_exterior_fails_shell_orientation() {
        let s = Solid::from_exterior(CoordinateFrame::Euclidean, tetra_inward());
        assert_eq!(failure_count(&s, ValidationType::ShellOrientation), 1);
    }

    #[test]
    fn void_must_be_inward_for_shell_orientation() {
        // Exterior outward + void inward = correct.
        let ok = Solid::new(
            CoordinateFrame::Euclidean,
            tetra_outward(),
            vec![tetra_inward().into()],
        );
        assert!(is_success(&ok, ValidationType::ShellOrientation));
        // A void wound outward (positive volume) is misoriented.
        let bad = Solid::new(
            CoordinateFrame::Euclidean,
            tetra_outward(),
            vec![tetra_outward().into()],
        );
        assert_eq!(failure_count(&bad, ValidationType::ShellOrientation), 1);
    }
}
