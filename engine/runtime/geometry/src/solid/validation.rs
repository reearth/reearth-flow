use std::collections::BTreeSet;

use super::{Shell, Solid};
use crate::coordinate::CoordinateFrame;
use crate::line_string::LineString3D;
use crate::ops::triangulation::Cache;
use crate::polygon_mesh::PolygonMesh3D;
use crate::predicates::surface_intersection::intersecting_faces_3d;
use crate::predicates::view3d::TriangleSet;
use crate::triangular_mesh::TriangularMesh3D;
use crate::validation_next::{
    check_duplicate_points, check_edge_orientation_3d, check_finite_3d, Validate, ValidationParams,
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
/// solid's interiors are void *shells*, not interior rings. `Orientable` and
/// `Orientation` are checked per shell; they are also the prerequisites that make
/// `ShellOrientation`'s signed-volume test meaningful.
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

    fn metric_kind(&self) -> crate::coordinate::MetricKind {
        self.frame.metric_kind()
    }

    fn check_finite(&self, _params: &ValidationParams) -> ValidationReport {
        ValidationReport::ran(|r| {
            for shell in self.shells() {
                check_finite_3d(&self.frame, shell.vertices().iter().copied(), r);
            }
        })
    }

    fn check_too_few_points(&self, _params: &ValidationParams) -> ValidationReport {
        ValidationReport::ran(|r| {
            for shell in self.shells() {
                // A triangle shell's faces always have three corners; only a
                // polygon shell can carry a degenerate ring.
                if let Shell::PolygonMesh(data) = shell {
                    data.check_too_few_points(&self.frame, r);
                }
            }
        })
    }

    fn check_unclosed_ring(&self, _params: &ValidationParams) -> ValidationReport {
        ValidationReport::ran(|r| {
            for shell in self.shells() {
                // A triangle shell's faces are implicitly closed; only a polygon
                // shell stores an explicit closing vertex.
                if let Shell::PolygonMesh(data) = shell {
                    data.check_unclosed_rings(&self.frame, r);
                }
            }
        })
    }

    fn check_duplicate_points(&self, params: &ValidationParams) -> ValidationReport {
        ValidationReport::ran(|r| {
            for shell in self.shells() {
                check_duplicate_points(
                    &self.frame,
                    shell.vertices().iter().copied(),
                    params.duplicate_tolerance,
                    r,
                );
            }
        })
    }

    fn check_orientation(&self, _params: &ValidationParams) -> ValidationReport {
        // Each shell must be consistently wound across its shared edges.
        ValidationReport::ran(|r| {
            for shell in self.shells() {
                shell.check_orientation(&self.frame, r);
            }
        })
    }

    fn check_orientable(&self, _params: &ValidationParams) -> ValidationReport {
        // Each shell must admit a consistent orientation.
        ValidationReport::ran(|r| {
            for shell in self.shells() {
                if !shell.is_orientable() {
                    r.push(shell.to_geometry(&self.frame));
                }
            }
        })
    }

    fn check_shell_manifold(&self, _params: &ValidationParams) -> ValidationReport {
        // Each shell must be a watertight closed connected 2-manifold.
        ValidationReport::ran(|r| {
            for shell in self.shells() {
                if !shell.is_closed_connected_manifold() {
                    r.push(shell.to_geometry(&self.frame));
                }
            }
        })
    }

    fn check_self_intersection(&self, _params: &ValidationParams) -> ValidationReport {
        // Per-face ring checks for polygon shells (a proper triangle face is
        // trivially simple), then one global face-vs-face scan across all
        // shells' triangulated surfaces, so cross-shell pairs are covered. The
        // surface scan triangulates each shell, and triangulation on non-metric
        // (angular-unit) coordinates is unreliable, so it is skipped there; the
        // ring checks still run.
        ValidationReport::ran(|r| {
            for shell in self.shells() {
                if let Shell::PolygonMesh(data) = shell {
                    data.check_ring_self_intersections(&self.frame, r);
                }
            }
            if !self.frame.is_metric() {
                return;
            }
            let mut cache = Cache::new();
            let sets: Vec<TriangleSet> = self
                .shells()
                .map(|shell| TriangleSet::from_shell(shell, &mut cache))
                .collect();
            let refs: Vec<&TriangleSet> = sets.iter().collect();
            let mut per_shell: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); sets.len()];
            for (shell, face) in intersecting_faces_3d(&refs) {
                per_shell[shell].insert(face);
            }
            for (shell, faces) in self.shells().zip(per_shell) {
                if faces.is_empty() {
                    continue;
                }
                match shell {
                    Shell::PolygonMesh(data) => {
                        data.push_face_exteriors(&self.frame, &faces, r);
                    }
                    Shell::TriangularMesh(data) => {
                        let vertices = data.vertices();
                        for (t, tri) in data.triangles().enumerate() {
                            if faces.contains(&t) {
                                r.push(Geometry::Euclidean3D(Euclidean3DGeometry::LineString(
                                    LineString3D::from_coords(
                                        self.frame.clone(),
                                        tri.map(|i| vertices[i as usize]),
                                    ),
                                )));
                            }
                        }
                    }
                }
            }
        })
    }

    fn check_degenerate(&self, params: &ValidationParams) -> ValidationReport {
        // A shell enclosing at most `min_volume` is degenerate; the position
        // is the shell itself.
        ValidationReport::ran(|r| {
            for shell in self.shells() {
                if shell.signed_volume().abs() <= params.degenerate.min_volume {
                    r.push(shell.to_geometry(&self.frame));
                }
            }
        })
    }

    fn check_shell_orientation(&self, _params: &ValidationParams) -> ValidationReport {
        // In canonical orientation the exterior shell must enclose positive volume
        // (outward normals) and each void shell negative volume (normals into the
        // void); a signed volume in a reflected frame carries the opposite sign, so
        // it is judged after applying the frame's orientation sign (see
        // [`crate::coordinate`]). An undeterminable frame skips the check.
        ValidationReport::ran(|r| {
            let Ok(sign) = self.frame.orientation_sign() else {
                return;
            };
            let sign = sign as f64;
            if self.exterior().signed_volume() * sign <= 0.0 {
                r.push(self.exterior().to_geometry(&self.frame));
            }
            for void in self.interiors() {
                if void.signed_volume() * sign >= 0.0 {
                    r.push(void.to_geometry(&self.frame));
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::EpsgCode;
    use crate::triangular_mesh::TriangularMesh3DData;
    use crate::validation_next::{validate_one, ValidationParams, ValidationResult};

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
        validate_one(s, check, &ValidationParams::default()) == ValidationResult::Success
    }

    fn failure_count(s: &Solid, check: ValidationType) -> usize {
        match validate_one(s, check, &ValidationParams::default()) {
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

    #[test]
    fn tetra_is_not_self_intersecting() {
        let s = Solid::from_exterior(CoordinateFrame::Euclidean, tetra_outward());
        assert!(is_success(&s, ValidationType::SelfIntersection));
    }

    #[test]
    fn self_crossing_shell_is_flagged() {
        // Two triangles of one shell piercing each other.
        let crossing = TriangularMesh3DData::from_parts(
            vec![
                [0.0, 0.0, 0.0],
                [4.0, 0.0, 0.0],
                [0.0, 4.0, 0.0],
                [1.0, 1.0, -1.0],
                [1.0, 1.0, 1.0],
                [3.0, 3.0, 1.0],
            ],
            [0u32, 1, 2, 3, 4, 5],
        )
        .unwrap();
        let s = Solid::from_exterior(CoordinateFrame::Euclidean, crossing);
        assert_eq!(failure_count(&s, ValidationType::SelfIntersection), 2);
    }

    /// The outward tetra scaled by 4, roomy enough to host voids.
    fn big_tetra_outward() -> TriangularMesh3DData {
        let verts: Vec<[f64; 3]> = tetra_verts()
            .into_iter()
            .map(|p| p.map(|c| c * 4.0))
            .collect();
        TriangularMesh3DData::from_parts(verts, [1u32, 2, 3, 0, 3, 2, 0, 1, 3, 0, 2, 1]).unwrap()
    }

    /// An inward-wound tetra over the given vertices.
    fn inward_tetra(verts: Vec<[f64; 3]>) -> TriangularMesh3DData {
        TriangularMesh3DData::from_parts(verts, [1u32, 3, 2, 0, 2, 3, 0, 3, 1, 0, 1, 2]).unwrap()
    }

    #[test]
    fn void_strictly_inside_does_not_self_intersect() {
        let void = inward_tetra(vec![
            [1.0, 1.0, 1.0],
            [1.5, 1.0, 1.0],
            [1.0, 1.5, 1.0],
            [1.0, 1.0, 1.5],
        ]);
        let s = Solid::new(
            CoordinateFrame::Euclidean,
            big_tetra_outward(),
            vec![void.into()],
        );
        assert!(is_success(&s, ValidationType::SelfIntersection));
    }

    #[test]
    fn void_crossing_the_exterior_shell_is_flagged() {
        // The void's apex pokes through the exterior's bottom face (z = 0).
        let void = inward_tetra(vec![
            [1.0, 1.0, -1.0],
            [1.5, 1.0, 1.0],
            [1.0, 1.5, 1.0],
            [1.0, 1.0, 1.5],
        ]);
        let s = Solid::new(
            CoordinateFrame::Euclidean,
            big_tetra_outward(),
            vec![void.into()],
        );
        assert!(failure_count(&s, ValidationType::SelfIntersection) >= 2);
    }

    #[test]
    fn shells_touching_at_a_corner_coordinate_are_allowed() {
        // The void's apex coincides with the exterior's corner; the pools are
        // separate, so the contact matches by coordinate only.
        let void = inward_tetra(vec![
            [0.0, 0.0, 0.0],
            [1.5, 1.0, 1.0],
            [1.0, 1.5, 1.0],
            [1.0, 1.0, 1.5],
        ]);
        let s = Solid::new(
            CoordinateFrame::Euclidean,
            big_tetra_outward(),
            vec![void.into()],
        );
        assert!(is_success(&s, ValidationType::SelfIntersection));
    }

    #[test]
    fn healthy_tetra_is_not_degenerate() {
        let s = Solid::from_exterior(CoordinateFrame::Euclidean, tetra_outward());
        assert!(is_success(&s, ValidationType::Degenerate));
    }

    #[test]
    fn flat_shell_is_degenerate() {
        // A doubled triangle encloses no volume.
        let flat = TriangularMesh3DData::from_parts(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            [0u32, 1, 2, 0, 2, 1],
        )
        .unwrap();
        let s = Solid::from_exterior(CoordinateFrame::Euclidean, flat);
        let positions =
            match validate_one(&s, ValidationType::Degenerate, &ValidationParams::default()) {
                ValidationResult::Failed(positions) => positions,
                other => panic!("expected a failure, got {other:?}"),
            };
        assert_eq!(positions.len(), 1);
        assert!(matches!(
            positions[0],
            Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(_))
        ));
    }

    #[test]
    fn min_volume_threshold_is_inclusive() {
        // The unit tetra encloses exactly 1/6.
        let s = Solid::from_exterior(CoordinateFrame::Euclidean, tetra_outward());
        let at_volume = ValidationParams {
            degenerate: crate::validation_next::DegenerateThresholds {
                min_volume: 1.0 / 6.0,
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(matches!(
            validate_one(&s, ValidationType::Degenerate, &at_volume),
            ValidationResult::Failed(_)
        ));
    }

    #[test]
    fn reflected_frame_inverts_shell_orientation() {
        // EPSG:6697 is lat-first (orientation sign -1), so a shell's signed volume
        // carries the opposite sign there: the raw-outward tetra encloses negative
        // volume and is canonically inward (misoriented as an exterior), while the
        // raw-inward tetra is canonically outward and valid.
        let reflected = CoordinateFrame::Crs(EpsgCode::new(6697));
        let outward = Solid::from_exterior(reflected.clone(), tetra_outward());
        assert_eq!(failure_count(&outward, ValidationType::ShellOrientation), 1);
        let inward = Solid::from_exterior(reflected, tetra_inward());
        assert!(is_success(&inward, ValidationType::ShellOrientation));
    }
}
