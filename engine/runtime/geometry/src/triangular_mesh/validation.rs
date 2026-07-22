use std::collections::BTreeSet;

use super::{TriangularMesh2D, TriangularMesh3D, TriangularMesh3DData};
use crate::line_string::{LineString2D, LineString3D};
use crate::point::Point2D;
use crate::predicates::surface_intersection::{
    face_overlap_conflicts_2d, intersecting_faces_3d, FaceConflict2D,
};
use crate::predicates::view::AreaView;
use crate::predicates::view3d::TriangleSet;
use crate::validation_next::{
    check_degenerate_ring_2d, check_degenerate_ring_3d, check_duplicate_points,
    check_edge_orientation_3d, check_finite_2d, check_finite_3d, check_ring_orientation_2d,
    tetra_volume_6x, FaceTopology, Validate, ValidationParams, ValidationReport, ValidationType,
};
use crate::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};

impl TriangularMesh3DData {
    /// The face-adjacency topology of this triangle mesh, one face per triangle.
    fn topology(&self) -> FaceTopology {
        FaceTopology::from_faces(self.triangles())
    }

    /// Whether the mesh admits a consistent orientation (see
    /// [`ValidationType::Orientable`]).
    pub(crate) fn is_orientable(&self) -> bool {
        self.topology().is_orientable()
    }

    /// Whether the mesh is a single connected component whose every edge is
    /// shared by exactly two triangles: a watertight closed 2-manifold.
    pub(crate) fn is_closed_connected_manifold(&self) -> bool {
        let topo = self.topology();
        topo.is_closed_manifold() && topo.is_connected()
    }

    /// The signed volume enclosed by this mesh, taken as a closed surface.
    /// Positive = outward normals. Meaningful only once the mesh is a closed,
    /// oriented shell.
    pub(crate) fn signed_volume(&self) -> f64 {
        let v = self.vertices();
        let mut acc = 0.0;
        for [a, b, c] in self.triangles() {
            acc += tetra_volume_6x(v[a as usize], v[b as usize], v[c as usize]);
        }
        acc / 6.0
    }
}

/// The checks that apply to a 2D triangle mesh. A triangle always has three
/// corners, so `TooFewPoints` / `UnclosedRing` cannot apply.
const TRIANGULAR_MESH_2D_CHECKS: [ValidationType; 5] = [
    ValidationType::Finite,
    ValidationType::Degenerate,
    ValidationType::DuplicatePoints,
    ValidationType::Orientation,
    ValidationType::SelfIntersection,
];

/// The checks that apply to a 3D triangle mesh: the 2D set plus `Orientable`.
const TRIANGULAR_MESH_3D_CHECKS: [ValidationType; 6] = [
    ValidationType::Finite,
    ValidationType::Degenerate,
    ValidationType::DuplicatePoints,
    ValidationType::Orientation,
    ValidationType::Orientable,
    ValidationType::SelfIntersection,
];

impl Validate for TriangularMesh2D {
    fn applicable_checks(&self) -> &'static [ValidationType] {
        &TRIANGULAR_MESH_2D_CHECKS
    }

    fn check_finite(&self, _params: &ValidationParams) -> ValidationReport {
        ValidationReport::ran(|r| {
            check_finite_2d(&self.frame, &self.vertices, self.z.as_deref(), r)
        })
    }

    fn check_orientation(&self, _params: &ValidationParams) -> ValidationReport {
        // Each triangle should wind counter-clockwise in canonical orientation
        // (after applying the frame's orientation sign). An undeterminable frame
        // skips the check.
        ValidationReport::ran(|r| {
            let Ok(sign) = self.frame.orientation_sign() else {
                return;
            };
            for [a, b, c] in self.triangles() {
                let ring = [
                    self.vertices[a as usize],
                    self.vertices[b as usize],
                    self.vertices[c as usize],
                ];
                check_ring_orientation_2d(&self.frame, sign, &ring, true, r);
            }
        })
    }

    fn check_duplicate_points(&self, params: &ValidationParams) -> ValidationReport {
        // Coincident vertices in the shared pool are a defect.
        ValidationReport::ran(|r| {
            check_duplicate_points::<2>(
                &self.frame,
                self.vertices.iter().copied(),
                params.duplicate_tolerance,
                r,
            )
        })
    }

    fn check_degenerate(&self, params: &ValidationParams) -> ValidationReport {
        // Each triangle is a three-vertex open ring measured by area.
        ValidationReport::ran(|r| {
            for [a, b, c] in self.triangles() {
                let ring = [
                    self.vertices[a as usize],
                    self.vertices[b as usize],
                    self.vertices[c as usize],
                ];
                check_degenerate_ring_2d(&self.frame, &ring, params.degenerate.min_area, r);
            }
        })
    }

    fn check_self_intersection(&self, _params: &ValidationParams) -> ValidationReport {
        // A triangle face is trivially simple, so only the global face-vs-face
        // overlap scan applies. The offending triangle's ring is the position.
        ValidationReport::ran(|r| {
            let view = AreaView::from_triangular_mesh(self);
            face_overlap_conflicts_2d(&view, |conflict| match conflict {
                FaceConflict2D::Crossing(p) => {
                    r.push(Geometry::Euclidean2D(Euclidean2DGeometry::Point(
                        Point2D::new(self.frame.clone(), p),
                    )));
                }
                FaceConflict2D::Contained { face } => {
                    let coords: Vec<[f64; 2]> = view.face(face).exterior().coords().collect();
                    r.push(Geometry::Euclidean2D(Euclidean2DGeometry::LineString(
                        LineString2D::from_coords(self.frame.clone(), coords),
                    )));
                }
            });
        })
    }
}

impl Validate for TriangularMesh3D {
    fn applicable_checks(&self) -> &'static [ValidationType] {
        &TRIANGULAR_MESH_3D_CHECKS
    }

    fn unit_kind(&self) -> crate::coordinate::UnitKind {
        self.frame.unit_kind()
    }

    fn check_finite(&self, _params: &ValidationParams) -> ValidationReport {
        ValidationReport::ran(|r| {
            check_finite_3d(&self.frame, self.data.vertices().iter().copied(), r)
        })
    }

    fn check_orientation(&self, _params: &ValidationParams) -> ValidationReport {
        // Adjacent triangles must wind coherently across each shared edge.
        ValidationReport::ran(|r| {
            check_edge_orientation_3d(&self.frame, self.data.vertices(), self.triangles(), r)
        })
    }

    fn check_orientable(&self, _params: &ValidationParams) -> ValidationReport {
        // A non-orientable mesh has no valid winding; report the whole mesh.
        ValidationReport::ran(|r| {
            if !self.data.is_orientable() {
                r.push(Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(
                    Box::new(self.clone()),
                )));
            }
        })
    }

    fn check_duplicate_points(&self, params: &ValidationParams) -> ValidationReport {
        ValidationReport::ran(|r| {
            check_duplicate_points::<3>(
                &self.frame,
                self.data.vertices().iter().copied(),
                params.duplicate_tolerance,
                r,
            )
        })
    }

    fn check_degenerate(&self, params: &ValidationParams) -> ValidationReport {
        // Each triangle is a three-vertex open ring measured by area.
        ValidationReport::ran(|r| {
            let vertices = self.data.vertices();
            for [a, b, c] in self.triangles() {
                let ring = [
                    vertices[a as usize],
                    vertices[b as usize],
                    vertices[c as usize],
                ];
                check_degenerate_ring_3d(&self.frame, &ring, params.degenerate.min_area, r);
            }
        })
    }

    fn check_self_intersection(&self, _params: &ValidationParams) -> ValidationReport {
        // A triangle face is trivially simple, so only the global face-vs-face
        // scan applies. It compares triangles as Euclidean geometry, so it is
        // skipped on angular-unit frames; the offending triangle is the position.
        ValidationReport::ran(|r| {
            if !self.frame.has_linear_units() {
                return;
            }
            let set = TriangleSet::from_triangular_data(&self.data);
            let faces: BTreeSet<usize> = intersecting_faces_3d(&[&set])
                .into_iter()
                .map(|(_, face)| face)
                .collect();
            if faces.is_empty() {
                return;
            }
            let vertices = self.data.vertices();
            for (t, tri) in self.data.triangles().enumerate() {
                if faces.contains(&t) {
                    r.push(Geometry::Euclidean3D(Euclidean3DGeometry::LineString(
                        LineString3D::from_coords(
                            self.frame.clone(),
                            tri.map(|i| vertices[i as usize]),
                        ),
                    )));
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::{CoordinateFrame, EpsgCode};
    use crate::validation_next::{validate_one, ValidationParams, ValidationResult};

    // Each helper runs just `check` (and its prerequisites) on the mesh, not the
    // leaf's other, still-unimplemented checks.
    fn is_success<T: Validate>(m: &T, check: ValidationType) -> bool {
        validate_one(m, check, &ValidationParams::default()) == ValidationResult::Success
    }

    /// The failing positions of `check` on `m`, or a panic if it did not fail.
    fn failures<T: Validate>(m: &T, check: ValidationType) -> Vec<Geometry> {
        match validate_one(m, check, &ValidationParams::default()) {
            ValidationResult::Failed(positions) => positions,
            other => panic!("expected {check} to fail, got {other:?}"),
        }
    }

    #[test]
    fn ccw_triangle_is_oriented() {
        let m = TriangularMesh2D::from_parts(
            CoordinateFrame::Euclidean,
            vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            [0u32, 1, 2],
        )
        .unwrap();
        assert!(is_success(&m, ValidationType::Orientation));
    }

    #[test]
    fn cw_triangle_is_misoriented() {
        let m = TriangularMesh2D::from_parts(
            CoordinateFrame::Euclidean,
            vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            [0u32, 2, 1],
        )
        .unwrap();
        assert_eq!(failures(&m, ValidationType::Orientation).len(), 1);
    }

    #[test]
    fn triangle_winding_is_judged_in_canonical_orientation() {
        // EPSG:6697 is lat-first (orientation sign -1), so the raw-CCW triangle is
        // canonically clockwise and misoriented, while the raw-CW triangle is
        // canonically counter-clockwise and valid.
        let reflected = CoordinateFrame::Crs(EpsgCode::new(6697));
        let ccw_raw = TriangularMesh2D::from_parts(
            reflected.clone(),
            vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            [0u32, 1, 2],
        )
        .unwrap();
        assert_eq!(failures(&ccw_raw, ValidationType::Orientation).len(), 1);
        let cw_raw = TriangularMesh2D::from_parts(
            reflected,
            vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            [0u32, 2, 1],
        )
        .unwrap();
        assert!(is_success(&cw_raw, ValidationType::Orientation));
    }

    fn quad() -> Vec<[f64; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ]
    }

    #[test]
    fn coherent_triangles_share_edge_in_opposite_directions() {
        // (0,1,2) and (0,2,3): the shared 0-2 edge is traversed 2->0 and 0->2.
        let m =
            TriangularMesh3D::from_parts(CoordinateFrame::Euclidean, quad(), [0u32, 1, 2, 0, 2, 3])
                .unwrap();
        assert!(is_success(&m, ValidationType::Orientation));
    }

    #[test]
    fn incoherent_triangles_are_misoriented() {
        // (0,1,2) and (0,3,2): both traverse the shared edge as 2->0.
        let m =
            TriangularMesh3D::from_parts(CoordinateFrame::Euclidean, quad(), [0u32, 1, 2, 0, 3, 2])
                .unwrap();
        assert_eq!(failures(&m, ValidationType::Orientation).len(), 1);
    }

    #[test]
    fn manifold_mesh_is_orientable() {
        // Two triangles sharing edge 0-2: an ordinary 2-manifold patch.
        let m =
            TriangularMesh3D::from_parts(CoordinateFrame::Euclidean, quad(), [0u32, 1, 2, 0, 2, 3])
                .unwrap();
        assert!(is_success(&m, ValidationType::Orientable));
    }

    #[test]
    fn nonmanifold_mesh_is_not_orientable() {
        // Edge 0-1 is shared by three triangles, so no consistent orientation.
        let verts = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.0, -1.0, 0.0],
        ];
        let m = TriangularMesh3D::from_parts(
            CoordinateFrame::Euclidean,
            verts,
            [0u32, 1, 2, 0, 1, 3, 0, 1, 4],
        )
        .unwrap();
        let positions = failures(&m, ValidationType::Orientable);
        assert_eq!(positions.len(), 1);
        assert!(matches!(
            positions[0],
            Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(_))
        ));
    }

    #[test]
    fn healthy_triangles_are_not_degenerate() {
        let m = TriangularMesh2D::from_parts(
            CoordinateFrame::Euclidean,
            vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            [0u32, 1, 2],
        )
        .unwrap();
        assert!(is_success(&m, ValidationType::Degenerate));
    }

    #[test]
    fn collinear_triangle_is_degenerate() {
        // The second triangle spans collinear vertices: zero area.
        let m = TriangularMesh3D::from_parts(
            CoordinateFrame::Euclidean,
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [2.0, 0.0, 0.0],
            ],
            [0u32, 1, 2, 0, 1, 3],
        )
        .unwrap();
        let positions = failures(&m, ValidationType::Degenerate);
        assert_eq!(positions.len(), 1);
        // The position is the offending triangle as a three-vertex line.
        assert!(matches!(
            positions[0],
            Geometry::Euclidean3D(Euclidean3DGeometry::LineString(_))
        ));
    }

    #[test]
    fn min_area_threshold_applies_per_triangle() {
        // Two triangles of area 0.5 each.
        let m = TriangularMesh2D::from_parts(
            CoordinateFrame::Euclidean,
            vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]],
            [0u32, 1, 2, 1, 3, 2],
        )
        .unwrap();
        let at_area = ValidationParams {
            degenerate: crate::validation_next::DegenerateThresholds {
                min_area: 0.5,
                ..Default::default()
            },
            ..Default::default()
        };
        match validate_one(&m, ValidationType::Degenerate, &at_area) {
            ValidationResult::Failed(positions) => assert_eq!(positions.len(), 2),
            other => panic!("expected both triangles to flag, got {other:?}"),
        }
    }

    #[test]
    fn disjoint_triangles_2d_do_not_self_intersect() {
        let m = TriangularMesh2D::from_parts(
            CoordinateFrame::Euclidean,
            vec![
                [0.0, 0.0],
                [1.0, 0.0],
                [0.0, 1.0],
                [5.0, 5.0],
                [6.0, 5.0],
                [5.0, 6.0],
            ],
            [0u32, 1, 2, 3, 4, 5],
        )
        .unwrap();
        assert!(is_success(&m, ValidationType::SelfIntersection));
    }

    #[test]
    fn overlapping_triangles_2d_self_intersect() {
        // The second triangle is the first shifted by (1, 1), so their edges
        // cross and their interiors overlap.
        let m = TriangularMesh2D::from_parts(
            CoordinateFrame::Euclidean,
            vec![
                [0.0, 0.0],
                [4.0, 0.0],
                [0.0, 4.0],
                [1.0, 1.0],
                [5.0, 1.0],
                [1.0, 5.0],
            ],
            [0u32, 1, 2, 3, 4, 5],
        )
        .unwrap();
        assert!(!failures(&m, ValidationType::SelfIntersection).is_empty());
    }

    #[test]
    fn adjacent_triangles_3d_do_not_self_intersect() {
        // Two coplanar triangles sharing edge 0-2: a manifold patch, no overlap.
        let m =
            TriangularMesh3D::from_parts(CoordinateFrame::Euclidean, quad(), [0u32, 1, 2, 0, 2, 3])
                .unwrap();
        assert!(is_success(&m, ValidationType::SelfIntersection));
    }

    /// A horizontal triangle in the `z = 0` plane and a vertical triangle whose
    /// edge passes through the horizontal triangle's interior.
    fn piercing_pair() -> Vec<[f64; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [0.0, 2.0, 0.0],
            [0.5, 0.5, -1.0],
            [0.5, 0.5, 1.0],
            [1.5, 0.5, 0.0],
        ]
    }

    #[test]
    fn piercing_triangles_3d_self_intersect() {
        let m = TriangularMesh3D::from_parts(
            CoordinateFrame::Euclidean,
            piercing_pair(),
            [0u32, 1, 2, 3, 4, 5],
        )
        .unwrap();
        assert_eq!(failures(&m, ValidationType::SelfIntersection).len(), 2);
    }

    #[test]
    fn self_intersection_3d_skipped_on_angular_frame() {
        // The scan compares triangles as Euclidean geometry, so an angular-unit
        // frame skips it and the piercing pair is not flagged.
        let m = TriangularMesh3D::from_parts(
            CoordinateFrame::Crs(EpsgCode::new(4326)),
            piercing_pair(),
            [0u32, 1, 2, 3, 4, 5],
        )
        .unwrap();
        assert!(is_success(&m, ValidationType::SelfIntersection));
    }
}
