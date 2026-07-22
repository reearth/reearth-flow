use std::collections::BTreeSet;

use super::{PolygonMesh2D, PolygonMesh3D, PolygonMesh3DData};
use crate::coordinate::CoordinateFrame;
use crate::index::IndexBuffer;
use crate::line_string::{LineString2D, LineString3D};
use crate::ops::triangulation::Cache;
use crate::point::Point2D;
use crate::predicates::surface_intersection::{
    face_overlap_conflicts_2d, intersecting_faces_3d, FaceConflict2D,
};
use crate::predicates::view::AreaView;
use crate::predicates::view3d::TriangleSet;
use crate::validation_next::{
    check_chain_simple_2d, check_chain_simple_3d, check_degenerate_ring_2d,
    check_degenerate_ring_3d, check_duplicate_points, check_finite_2d, check_finite_3d,
    check_holes_in_exterior_2d, check_holes_in_exterior_3d, check_ring_orientation_2d,
    check_ring_pair_2d, check_ring_pair_3d, check_too_few_points_2d, check_too_few_points_3d,
    check_unclosed_ring_2d, check_unclosed_ring_3d, open_ring, tetra_volume_6x, EdgeOrientation,
    FaceOrientation, FaceTopology, Validate, ValidationParams, ValidationReport, ValidationType,
};
use crate::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};

/// Decode the CSR face topology and invoke `f` once per face ring (each face's
/// exterior ring, then its hole rings), passing the ring's vertex indices and
/// whether it is an exterior ring (vs. a hole).
///
/// The flat index buffer is streamed rather than collected, and each ring is
/// materialized into a single buffer reused across rings, so nothing allocated
/// here scales with the corner count. Only the small per-face offset lists (one
/// entry per face / per hole) are collected.
fn for_each_ring(
    face_indices: &IndexBuffer<1>,
    face_offsets: &IndexBuffer<1>,
    interior_offsets: &IndexBuffer<1>,
    mut f: impl FnMut(&[u32], bool),
) {
    let n = face_indices.len();
    if n == 0 {
        return;
    }
    let face_ends: Vec<usize> = face_offsets.iter_u32().map(|[i]| i as usize).collect();
    let holes: Vec<usize> = interior_offsets.iter_u32().map(|[i]| i as usize).collect();
    let n_faces = face_ends.len() + 1;
    let mut indices = face_indices.iter_u32().map(|[i]| i);
    let mut ring: Vec<u32> = Vec::new();
    let mut start = 0usize;
    // `interior_offsets` are strictly increasing, and faces are visited in order,
    // so a single moving cursor walks the holes once across the whole mesh.
    let mut hole = 0usize;
    for face in 0..n_faces {
        let end = face_ends.get(face).copied().unwrap_or(n);
        // Hole rings of this face begin at the interior offsets inside (start, end);
        // the exterior ring runs up to the first hole (or the face end).
        let mut ring_start = start;
        let mut is_exterior = true;
        while hole < holes.len() && holes[hole] <= start {
            hole += 1;
        }
        while hole < holes.len() && holes[hole] < end {
            let h = holes[hole];
            ring.clear();
            ring.extend(indices.by_ref().take(h - ring_start));
            f(&ring, is_exterior);
            ring_start = h;
            is_exterior = false;
            hole += 1;
        }
        ring.clear();
        ring.extend(indices.by_ref().take(end - ring_start));
        f(&ring, is_exterior);
        start = end;
    }
}

/// The `[f64; N]` coordinates of one ring, gathered from the shared vertex pool.
fn ring_coords<const N: usize>(vertices: &[[f64; N]], ring: &[u32]) -> Vec<[f64; N]> {
    ring.iter().map(|&i| vertices[i as usize]).collect()
}

/// Decode the CSR face topology and invoke `f` once per face with that face's
/// ring coordinates, exterior first, then the face's holes.
fn for_each_face_coords<const N: usize>(
    vertices: &[[f64; N]],
    face_indices: &IndexBuffer<1>,
    face_offsets: &IndexBuffer<1>,
    interior_offsets: &IndexBuffer<1>,
    mut f: impl FnMut(&[Vec<[f64; N]>]),
) {
    let mut face: Vec<Vec<[f64; N]>> = Vec::new();
    for_each_ring(
        face_indices,
        face_offsets,
        interior_offsets,
        |ring, is_exterior| {
            if is_exterior && !face.is_empty() {
                f(&face);
                face.clear();
            }
            face.push(ring_coords(vertices, ring));
        },
    );
    if !face.is_empty() {
        f(&face);
    }
}

/// Report a [`ValidationType::SelfIntersection`] problem for every face whose
/// rings are not simple or cross each other. Shared by the 2D and 3D meshes;
/// `csr` is the mesh's `(face_indices, face_offsets, interior_offsets)`, and
/// `check_ring` / `check_pair` are the dimension-specific detectors.
fn check_mesh_ring_self_intersections<const N: usize>(
    frame: &CoordinateFrame,
    vertices: &[[f64; N]],
    csr: (&IndexBuffer<1>, &IndexBuffer<1>, &IndexBuffer<1>),
    check_ring: impl Fn(&CoordinateFrame, &[[f64; N]], &mut ValidationReport),
    check_pair: impl Fn(&CoordinateFrame, &[[f64; N]], &[[f64; N]], &mut ValidationReport),
    report: &mut ValidationReport,
) {
    for_each_face_coords(vertices, csr.0, csr.1, csr.2, |rings| {
        for ring in rings {
            check_ring(frame, ring, report);
        }
        for i in 0..rings.len() {
            for j in (i + 1)..rings.len() {
                check_pair(frame, &rings[i], &rings[j], report);
            }
        }
    });
}

/// Report a [`ValidationType::TooFewPoints`] problem for every face ring with
/// fewer than four (closed-ring) vertices. Shared by the 2D and 3D meshes;
/// `push` is the dimension-specific leaf check.
fn check_mesh_too_few_points<const N: usize>(
    frame: &CoordinateFrame,
    vertices: &[[f64; N]],
    face_indices: &IndexBuffer<1>,
    face_offsets: &IndexBuffer<1>,
    interior_offsets: &IndexBuffer<1>,
    push: impl Fn(&CoordinateFrame, &[[f64; N]], bool, &mut ValidationReport),
    report: &mut ValidationReport,
) {
    for_each_ring(face_indices, face_offsets, interior_offsets, |ring, _| {
        if ring.len() < 4 {
            push(frame, &ring_coords(vertices, ring), true, report);
        }
    });
}

/// Report a [`ValidationType::UnclosedRing`] problem for every face ring whose
/// first and last vertices differ. Shared by the 2D and 3D meshes; `push` is the
/// dimension-specific leaf check.
fn check_mesh_unclosed_rings<const N: usize>(
    frame: &CoordinateFrame,
    vertices: &[[f64; N]],
    face_indices: &IndexBuffer<1>,
    face_offsets: &IndexBuffer<1>,
    interior_offsets: &IndexBuffer<1>,
    push: impl Fn(&CoordinateFrame, &[[f64; N]], &mut ValidationReport),
    report: &mut ValidationReport,
) {
    for_each_ring(face_indices, face_offsets, interior_offsets, |ring, _| {
        // Only materialize the coords when the endpoints actually differ.
        if let (Some(&first), Some(&last)) = (ring.first(), ring.last()) {
            if vertices[first as usize] != vertices[last as usize] {
                push(frame, &ring_coords(vertices, ring), report);
            }
        }
    });
}

impl PolygonMesh3DData {
    /// Report a [`ValidationType::TooFewPoints`] problem for every face ring with
    /// fewer than four (closed-ring) vertices. Shared by the [`PolygonMesh3D`]
    /// leaf and [`Solid`](crate::solid::Solid) shells, which supply the frame.
    pub(crate) fn check_too_few_points(
        &self,
        frame: &CoordinateFrame,
        report: &mut ValidationReport,
    ) {
        check_mesh_too_few_points(
            frame,
            &self.vertices,
            &self.face_indices,
            &self.face_offsets,
            &self.interior_offsets,
            check_too_few_points_3d,
            report,
        );
    }

    /// Report a [`ValidationType::UnclosedRing`] problem for every face ring whose
    /// first and last vertices differ. Shared by the [`PolygonMesh3D`] leaf and
    /// [`Solid`](crate::solid::Solid) shells, which supply the frame.
    pub(crate) fn check_unclosed_rings(
        &self,
        frame: &CoordinateFrame,
        report: &mut ValidationReport,
    ) {
        check_mesh_unclosed_rings(
            frame,
            &self.vertices,
            &self.face_indices,
            &self.face_offsets,
            &self.interior_offsets,
            check_unclosed_ring_3d,
            report,
        );
    }

    /// Report a [`ValidationType::Orientation`] problem for face rings that wind
    /// inconsistently across a shared edge (cross-face) or for a hole that fails to
    /// wind opposite its own face's exterior (per-face). Shared by the
    /// [`PolygonMesh3D`] leaf and [`Solid`](crate::solid::Solid) shells.
    pub(crate) fn check_orientation(&self, frame: &CoordinateFrame, report: &mut ValidationReport) {
        let mut edges = EdgeOrientation::new();
        let mut faces = FaceOrientation::new();
        // The per-face hole check only fires when some face has a hole; skip its
        // per-ring coord materialization and normal entirely otherwise.
        let has_holes = self.interior_offsets.len() != 0;
        for_each_ring(
            &self.face_indices,
            &self.face_offsets,
            &self.interior_offsets,
            |ring, is_exterior| {
                edges.check_ring(frame, &self.vertices, ring, report);
                if has_holes {
                    faces.check_ring(
                        frame,
                        &ring_coords(&self.vertices, ring),
                        is_exterior,
                        report,
                    );
                }
            },
        );
    }

    /// Report a [`ValidationType::SelfIntersection`] problem for every face
    /// whose rings are not simple or cross each other. Shared by the
    /// [`PolygonMesh3D`] leaf and [`Solid`](crate::solid::Solid) shells, which
    /// supply the frame.
    pub(crate) fn check_ring_self_intersections(
        &self,
        frame: &CoordinateFrame,
        report: &mut ValidationReport,
    ) {
        check_mesh_ring_self_intersections(
            frame,
            &self.vertices,
            (
                &self.face_indices,
                &self.face_offsets,
                &self.interior_offsets,
            ),
            check_chain_simple_3d,
            check_ring_pair_3d,
            report,
        );
    }

    /// Report an [`ValidationType::InteriorRingContainment`] problem for every
    /// face hole outside its own face's exterior ring. Shared by the
    /// [`PolygonMesh3D`] leaf and [`Solid`](crate::solid::Solid) shells.
    pub(crate) fn check_interior_ring_containment(
        &self,
        frame: &CoordinateFrame,
        report: &mut ValidationReport,
    ) {
        for_each_face_coords(
            &self.vertices,
            &self.face_indices,
            &self.face_offsets,
            &self.interior_offsets,
            |rings| {
                if rings.len() > 1 {
                    check_holes_in_exterior_3d(
                        frame,
                        &rings[0],
                        rings[1..].iter().map(|hole| hole.as_slice()),
                        report,
                    );
                }
            },
        );
    }

    /// Report a [`ValidationType::Degenerate`] problem for every face ring
    /// whose area is at most `min_area`. Shared by the [`PolygonMesh3D`] leaf
    /// and [`Solid`](crate::solid::Solid) shells.
    pub(crate) fn check_degenerate_rings(
        &self,
        frame: &CoordinateFrame,
        min_area: f64,
        report: &mut ValidationReport,
    ) {
        for_each_ring(
            &self.face_indices,
            &self.face_offsets,
            &self.interior_offsets,
            |ring, _| {
                check_degenerate_ring_3d(
                    frame,
                    &ring_coords(&self.vertices, ring),
                    min_area,
                    report,
                );
            },
        );
    }

    /// Report a [`ValidationType::SelfIntersection`] problem for every face
    /// that intersects another face of this mesh beyond shared corners and
    /// edges, through the triangulated surface. The position is the offending
    /// face's exterior ring.
    pub(crate) fn check_face_intersections(
        &self,
        frame: &CoordinateFrame,
        report: &mut ValidationReport,
    ) {
        let mut cache = Cache::new();
        let set = TriangleSet::from_polygon_mesh_data(self, &mut cache);
        let conflicts = intersecting_faces_3d(&[&set]);
        let faces: BTreeSet<usize> = conflicts.into_iter().map(|(_, face)| face).collect();
        self.push_face_exteriors(frame, &faces, report);
    }

    /// Push the exterior ring of each listed face (by CSR face order) as a
    /// LineString position.
    pub(crate) fn push_face_exteriors(
        &self,
        frame: &CoordinateFrame,
        faces: &BTreeSet<usize>,
        report: &mut ValidationReport,
    ) {
        if faces.is_empty() {
            return;
        }
        let mut face = usize::MAX;
        for_each_ring(
            &self.face_indices,
            &self.face_offsets,
            &self.interior_offsets,
            |ring, is_exterior| {
                if is_exterior {
                    face = face.wrapping_add(1);
                    if faces.contains(&face) {
                        report.push(Geometry::Euclidean3D(Euclidean3DGeometry::LineString(
                            LineString3D::from_coords(
                                frame.clone(),
                                ring.iter().map(|&i| self.vertices[i as usize]),
                            ),
                        )));
                    }
                }
            },
        );
    }

    /// The face-adjacency topology of this mesh, one face per decoded ring.
    fn topology(&self) -> FaceTopology {
        let mut topology = FaceTopology::new();
        for_each_ring(
            &self.face_indices,
            &self.face_offsets,
            &self.interior_offsets,
            |ring, _| topology.add_face(ring),
        );
        topology
    }

    /// Whether the mesh admits a consistent orientation (see
    /// [`ValidationType::Orientable`]).
    pub(crate) fn is_orientable(&self) -> bool {
        self.topology().is_orientable()
    }

    /// Whether the mesh is a single connected component whose every edge is
    /// shared by exactly two faces: a watertight closed 2-manifold.
    pub(crate) fn is_closed_connected_manifold(&self) -> bool {
        let topo = self.topology();
        topo.is_closed_manifold() && topo.is_connected()
    }

    /// The signed volume enclosed by this mesh, taken as a closed surface: each
    /// face is fan-triangulated and its tetrahedra summed. Positive = outward
    /// normals. Meaningful only once the mesh is a closed, oriented shell.
    pub(crate) fn signed_volume(&self) -> f64 {
        let mut acc = 0.0;
        for_each_ring(
            &self.face_indices,
            &self.face_offsets,
            &self.interior_offsets,
            |ring, _| {
                // Drop the closing vertex so the fan uses only distinct corners.
                let ring = open_ring(ring);
                if ring.len() < 3 {
                    return;
                }
                let p0 = self.vertices[ring[0] as usize];
                for k in 1..ring.len() - 1 {
                    let p1 = self.vertices[ring[k] as usize];
                    let p2 = self.vertices[ring[k + 1] as usize];
                    acc += tetra_volume_6x(p0, p1, p2);
                }
            },
        );
        acc / 6.0
    }
}

/// The checks that apply to a 2D polygon mesh.
const POLYGON_MESH_2D_CHECKS: [ValidationType; 8] = [
    ValidationType::Finite,
    ValidationType::TooFewPoints,
    ValidationType::UnclosedRing,
    ValidationType::SelfIntersection,
    ValidationType::InteriorRingContainment,
    ValidationType::Degenerate,
    ValidationType::DuplicatePoints,
    ValidationType::Orientation,
];

/// The checks that apply to a 3D polygon mesh: the 2D set plus `Orientable`.
const POLYGON_MESH_3D_CHECKS: [ValidationType; 9] = [
    ValidationType::Finite,
    ValidationType::TooFewPoints,
    ValidationType::UnclosedRing,
    ValidationType::SelfIntersection,
    ValidationType::InteriorRingContainment,
    ValidationType::Degenerate,
    ValidationType::DuplicatePoints,
    ValidationType::Orientation,
    ValidationType::Orientable,
];

impl Validate for PolygonMesh2D {
    fn applicable_checks(&self) -> &'static [ValidationType] {
        &POLYGON_MESH_2D_CHECKS
    }

    fn check_finite(&self, _params: &ValidationParams) -> ValidationReport {
        ValidationReport::ran(|r| {
            check_finite_2d(&self.frame, &self.vertices, self.z.as_deref(), r)
        })
    }

    fn check_too_few_points(&self, _params: &ValidationParams) -> ValidationReport {
        ValidationReport::ran(|r| {
            check_mesh_too_few_points(
                &self.frame,
                &self.vertices,
                &self.face_indices,
                &self.face_offsets,
                &self.interior_offsets,
                check_too_few_points_2d,
                r,
            );
        })
    }

    fn check_unclosed_ring(&self, _params: &ValidationParams) -> ValidationReport {
        ValidationReport::ran(|r| {
            check_mesh_unclosed_rings(
                &self.frame,
                &self.vertices,
                &self.face_indices,
                &self.face_offsets,
                &self.interior_offsets,
                check_unclosed_ring_2d,
                r,
            );
        })
    }

    fn check_orientation(&self, _params: &ValidationParams) -> ValidationReport {
        // Each face's exterior ring must wind counter-clockwise, its holes
        // clockwise, in canonical orientation (after applying the frame's
        // orientation sign). An undeterminable frame skips the check.
        ValidationReport::ran(|r| {
            let Ok(sign) = self.frame.orientation_sign() else {
                return;
            };
            for_each_ring(
                &self.face_indices,
                &self.face_offsets,
                &self.interior_offsets,
                |ring, is_exterior| {
                    let coords = ring_coords(&self.vertices, ring);
                    check_ring_orientation_2d(&self.frame, sign, &coords, is_exterior, r);
                },
            );
        })
    }

    fn check_duplicate_points(&self, params: &ValidationParams) -> ValidationReport {
        // A shared vertex pool should hold no coincident vertices; elevation is
        // not considered.
        ValidationReport::ran(|r| {
            check_duplicate_points(
                &self.frame,
                self.vertices.iter().copied(),
                params.duplicate_tolerance,
                r,
            )
        })
    }

    fn check_self_intersection(&self, _params: &ValidationParams) -> ValidationReport {
        // Per-face ring simplicity plus the global face-vs-face overlap scan.
        ValidationReport::ran(|r| {
            check_mesh_ring_self_intersections(
                &self.frame,
                &self.vertices,
                (
                    &self.face_indices,
                    &self.face_offsets,
                    &self.interior_offsets,
                ),
                check_chain_simple_2d,
                check_ring_pair_2d,
                r,
            );
            let view = AreaView::from_polygon_mesh(self);
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

    fn check_interior_ring_containment(&self, _params: &ValidationParams) -> ValidationReport {
        ValidationReport::ran(|r| {
            for_each_face_coords(
                &self.vertices,
                &self.face_indices,
                &self.face_offsets,
                &self.interior_offsets,
                |rings| {
                    if rings.len() > 1 {
                        check_holes_in_exterior_2d(
                            &self.frame,
                            &rings[0],
                            rings[1..].iter().map(|hole| hole.as_slice()),
                            r,
                        );
                    }
                },
            );
        })
    }

    fn check_degenerate(&self, params: &ValidationParams) -> ValidationReport {
        ValidationReport::ran(|r| {
            for_each_ring(
                &self.face_indices,
                &self.face_offsets,
                &self.interior_offsets,
                |ring, _| {
                    check_degenerate_ring_2d(
                        &self.frame,
                        &ring_coords(&self.vertices, ring),
                        params.degenerate.min_area,
                        r,
                    );
                },
            );
        })
    }
}

impl Validate for PolygonMesh3D {
    fn applicable_checks(&self) -> &'static [ValidationType] {
        &POLYGON_MESH_3D_CHECKS
    }

    fn metric_kind(&self) -> crate::coordinate::MetricKind {
        self.frame.metric_kind()
    }

    fn check_finite(&self, _params: &ValidationParams) -> ValidationReport {
        ValidationReport::ran(|r| {
            check_finite_3d(&self.frame, self.data.vertices().iter().copied(), r)
        })
    }

    fn check_too_few_points(&self, _params: &ValidationParams) -> ValidationReport {
        ValidationReport::ran(|r| self.data.check_too_few_points(&self.frame, r))
    }

    fn check_unclosed_ring(&self, _params: &ValidationParams) -> ValidationReport {
        ValidationReport::ran(|r| self.data.check_unclosed_rings(&self.frame, r))
    }

    fn check_orientation(&self, _params: &ValidationParams) -> ValidationReport {
        ValidationReport::ran(|r| self.data.check_orientation(&self.frame, r))
    }

    fn check_orientable(&self, _params: &ValidationParams) -> ValidationReport {
        // A non-orientable mesh has no valid winding; report the whole mesh.
        ValidationReport::ran(|r| {
            if !self.data.is_orientable() {
                r.push(Geometry::Euclidean3D(Euclidean3DGeometry::PolygonMesh(
                    Box::new(self.clone()),
                )));
            }
        })
    }

    fn check_duplicate_points(&self, params: &ValidationParams) -> ValidationReport {
        ValidationReport::ran(|r| {
            check_duplicate_points(
                &self.frame,
                self.data.vertices().iter().copied(),
                params.duplicate_tolerance,
                r,
            )
        })
    }

    fn check_self_intersection(&self, _params: &ValidationParams) -> ValidationReport {
        // Per-face ring simplicity (exact, frame-agnostic) plus the global
        // face-vs-face surface scan. The surface scan triangulates each face, and
        // triangulation on non-metric (angular-unit) coordinates is unreliable, so
        // it is skipped there; the ring checks still run.
        ValidationReport::ran(|r| {
            self.data.check_ring_self_intersections(&self.frame, r);
            if self.frame.is_metric() {
                self.data.check_face_intersections(&self.frame, r);
            }
        })
    }

    fn check_interior_ring_containment(&self, _params: &ValidationParams) -> ValidationReport {
        ValidationReport::ran(|r| self.data.check_interior_ring_containment(&self.frame, r))
    }

    fn check_degenerate(&self, params: &ValidationParams) -> ValidationReport {
        ValidationReport::ran(|r| {
            self.data
                .check_degenerate_rings(&self.frame, params.degenerate.min_area, r)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::polygon_mesh::{PolygonMesh2D, PolygonMesh3D};
    use crate::validation_next::{validate_one, ValidationParams, ValidationResult};

    fn quad_verts() -> Vec<[f64; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ]
    }

    fn quad_verts_2d() -> Vec<[f64; 2]> {
        vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
    }

    // Each helper runs just `check` (and its prerequisites) on the mesh, not the
    // leaf's other, still-unimplemented checks.
    fn is_success<T: Validate>(m: &T, check: ValidationType) -> bool {
        validate_one(m, check, &ValidationParams::default()) == ValidationResult::Success
    }

    fn failure_count<T: Validate>(m: &T, check: ValidationType) -> usize {
        match validate_one(m, check, &ValidationParams::default()) {
            ValidationResult::Failed(positions) => positions.len(),
            other => panic!("expected {check} to fail, got {other:?}"),
        }
    }

    #[test]
    fn closed_face_ring_is_valid() {
        // Ring stored closed (last index repeats the first).
        let m = PolygonMesh3D::from_parts(
            CoordinateFrame::Euclidean,
            quad_verts(),
            [[0u32, 1, 2, 3, 0]],
        )
        .unwrap();
        assert!(is_success(&m, ValidationType::UnclosedRing));
        assert!(is_success(&m, ValidationType::TooFewPoints));
    }

    #[test]
    fn open_face_ring_is_unclosed() {
        let m =
            PolygonMesh3D::from_parts(CoordinateFrame::Euclidean, quad_verts(), [[0u32, 1, 2, 3]])
                .unwrap();
        assert_eq!(failure_count(&m, ValidationType::UnclosedRing), 1);
    }

    #[test]
    fn short_face_ring_is_too_few_points() {
        // A closed but degenerate ring: [0,1,0] is only three stored coords.
        let m = PolygonMesh3D::from_parts(CoordinateFrame::Euclidean, quad_verts(), [[0u32, 1, 0]])
            .unwrap();
        assert_eq!(failure_count(&m, ValidationType::TooFewPoints), 1);
    }

    #[test]
    fn ccw_exterior_ring_is_oriented() {
        let m = PolygonMesh2D::from_parts(
            CoordinateFrame::Euclidean,
            quad_verts_2d(),
            [[0u32, 1, 2, 3, 0]],
        )
        .unwrap();
        assert!(is_success(&m, ValidationType::Orientation));
    }

    #[test]
    fn cw_exterior_ring_is_misoriented() {
        let m = PolygonMesh2D::from_parts(
            CoordinateFrame::Euclidean,
            quad_verts_2d(),
            [[0u32, 3, 2, 1, 0]],
        )
        .unwrap();
        assert_eq!(failure_count(&m, ValidationType::Orientation), 1);
    }

    #[test]
    fn coherent_3d_faces_are_oriented() {
        // Two triangular faces sharing edge 0-2 in opposite directions.
        let m = PolygonMesh3D::from_parts(
            CoordinateFrame::Euclidean,
            quad_verts(),
            [[0u32, 1, 2, 0], [0, 2, 3, 0]],
        )
        .unwrap();
        assert!(is_success(&m, ValidationType::Orientation));
    }

    #[test]
    fn incoherent_3d_faces_are_misoriented() {
        // Both faces traverse the shared 0-2 edge as 2->0.
        let m = PolygonMesh3D::from_parts(
            CoordinateFrame::Euclidean,
            quad_verts(),
            [[0u32, 1, 2, 0], [0, 3, 2, 0]],
        )
        .unwrap();
        assert_eq!(failure_count(&m, ValidationType::Orientation), 1);
    }

    /// A one-face mesh: CCW exterior square in z = 0 with one hole.
    fn face_with_hole(hole: Vec<[f64; 3]>) -> PolygonMesh3D {
        let exterior = [
            [0.0, 0.0, 0.0],
            [4.0, 0.0, 0.0],
            [4.0, 4.0, 0.0],
            [0.0, 4.0, 0.0],
            [0.0, 0.0, 0.0],
        ];
        let poly =
            crate::polygon::Polygon3D::from_rings(CoordinateFrame::Euclidean, exterior, vec![hole]);
        PolygonMesh3D::from_polygons(CoordinateFrame::Euclidean, [&poly]).unwrap()
    }

    #[test]
    fn face_hole_opposite_exterior_is_oriented() {
        // CW hole opposes the CCW exterior: valid.
        let m = face_with_hole(vec![
            [1.0, 1.0, 0.0],
            [1.0, 2.0, 0.0],
            [2.0, 2.0, 0.0],
            [2.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ]);
        assert!(is_success(&m, ValidationType::Orientation));
    }

    #[test]
    fn face_hole_winding_like_exterior_is_misoriented() {
        // CCW hole winds like the exterior (not opposite): one problem.
        let m = face_with_hole(vec![
            [1.0, 1.0, 0.0],
            [2.0, 1.0, 0.0],
            [2.0, 2.0, 0.0],
            [1.0, 2.0, 0.0],
            [1.0, 1.0, 0.0],
        ]);
        assert_eq!(failure_count(&m, ValidationType::Orientation), 1);
    }

    #[test]
    fn shared_edge_2d_faces_are_not_self_intersecting() {
        let m = PolygonMesh2D::from_parts(
            CoordinateFrame::Euclidean,
            vec![
                [0.0, 0.0],
                [1.0, 0.0],
                [1.0, 1.0],
                [0.0, 1.0],
                [2.0, 0.0],
                [2.0, 1.0],
            ],
            [[0u32, 1, 2, 3, 0], [1, 4, 5, 2, 1]],
        )
        .unwrap();
        assert!(is_success(&m, ValidationType::SelfIntersection));
    }

    #[test]
    fn crossing_2d_faces_self_intersect_at_a_point() {
        let m = PolygonMesh2D::from_parts(
            CoordinateFrame::Euclidean,
            vec![
                [0.0, 0.0],
                [2.0, 0.0],
                [2.0, 2.0],
                [0.0, 2.0],
                [1.0, 1.0],
                [3.0, 1.0],
                [3.0, 3.0],
                [1.0, 3.0],
            ],
            [[0u32, 1, 2, 3, 0], [4, 5, 6, 7, 4]],
        )
        .unwrap();
        assert!(failure_count(&m, ValidationType::SelfIntersection) >= 1);
    }

    #[test]
    fn face_inside_another_2d_face_self_intersects() {
        let m = PolygonMesh2D::from_parts(
            CoordinateFrame::Euclidean,
            vec![
                [0.0, 0.0],
                [4.0, 0.0],
                [4.0, 4.0],
                [0.0, 4.0],
                [1.0, 1.0],
                [2.0, 1.0],
                [2.0, 2.0],
                [1.0, 2.0],
            ],
            [[0u32, 1, 2, 3, 0], [4, 5, 6, 7, 4]],
        )
        .unwrap();
        let positions = match validate_one(
            &m,
            ValidationType::SelfIntersection,
            &ValidationParams::default(),
        ) {
            ValidationResult::Failed(positions) => positions,
            other => panic!("expected a failure, got {other:?}"),
        };
        // The contained face is reported as its exterior ring.
        assert!(positions.iter().any(|p| matches!(
            p,
            crate::Geometry::Euclidean2D(crate::Euclidean2DGeometry::LineString(_))
        )));
    }

    #[test]
    fn bowtie_2d_face_ring_self_intersects() {
        let m = PolygonMesh2D::from_parts(
            CoordinateFrame::Euclidean,
            vec![[0.0, 0.0], [2.0, 2.0], [2.0, 0.0], [0.0, 2.0]],
            [[0u32, 1, 2, 3, 0]],
        )
        .unwrap();
        assert!(failure_count(&m, ValidationType::SelfIntersection) >= 1);
    }

    #[test]
    fn mesh_2d_hole_outside_its_face_is_not_contained() {
        // One face with a hole ring lying outside the exterior.
        let m = PolygonMesh2D::from_raw_parts(
            CoordinateFrame::Euclidean,
            vec![
                [0.0, 0.0],
                [4.0, 0.0],
                [4.0, 4.0],
                [0.0, 4.0],
                [5.0, 5.0],
                [6.0, 5.0],
                [6.0, 6.0],
                [5.0, 6.0],
            ],
            vec![0, 1, 2, 3, 0, 4, 5, 6, 7, 4],
            vec![],
            vec![5],
        )
        .unwrap();
        assert_eq!(
            failure_count(&m, ValidationType::InteriorRingContainment),
            1
        );
    }

    #[test]
    fn mesh_2d_hole_inside_its_face_is_contained() {
        let m = PolygonMesh2D::from_raw_parts(
            CoordinateFrame::Euclidean,
            vec![
                [0.0, 0.0],
                [4.0, 0.0],
                [4.0, 4.0],
                [0.0, 4.0],
                [1.0, 1.0],
                [2.0, 1.0],
                [2.0, 2.0],
                [1.0, 2.0],
            ],
            vec![0, 1, 2, 3, 0, 4, 7, 6, 5, 4],
            vec![],
            vec![5],
        )
        .unwrap();
        assert!(is_success(&m, ValidationType::InteriorRingContainment));
    }

    #[test]
    fn zero_area_2d_face_ring_is_degenerate() {
        let m = PolygonMesh2D::from_parts(
            CoordinateFrame::Euclidean,
            vec![[0.0, 0.0], [1.0, 1.0], [2.0, 2.0]],
            [[0u32, 1, 2, 0]],
        )
        .unwrap();
        assert_eq!(failure_count(&m, ValidationType::Degenerate), 1);
    }

    #[test]
    fn folded_3d_faces_sharing_an_edge_are_not_self_intersecting() {
        // Two wings folded along the shared 0-1 edge.
        let m = PolygonMesh3D::from_parts(
            CoordinateFrame::Euclidean,
            vec![
                [0.0, 0.0, 0.0],
                [4.0, 0.0, 0.0],
                [0.0, 4.0, 0.0],
                [0.0, -4.0, 4.0],
            ],
            [[0u32, 1, 2, 0], [1, 0, 3, 1]],
        )
        .unwrap();
        assert!(is_success(&m, ValidationType::SelfIntersection));
    }

    #[test]
    fn coplanar_overlapping_3d_faces_self_intersect() {
        // Two coplanar triangles on the same side of their shared edge.
        let m = PolygonMesh3D::from_parts(
            CoordinateFrame::Euclidean,
            vec![
                [0.0, 0.0, 0.0],
                [4.0, 0.0, 0.0],
                [0.0, 4.0, 0.0],
                [2.0, 4.0, 0.0],
            ],
            [[0u32, 1, 2, 0], [0, 1, 3, 0]],
        )
        .unwrap();
        assert_eq!(failure_count(&m, ValidationType::SelfIntersection), 2);
    }

    #[test]
    fn piercing_3d_face_self_intersects() {
        // The second face passes through the first away from any shared vertex.
        let m = PolygonMesh3D::from_parts(
            CoordinateFrame::Euclidean,
            vec![
                [0.0, 0.0, 0.0],
                [4.0, 0.0, 0.0],
                [0.0, 4.0, 0.0],
                [1.0, 1.0, -1.0],
                [1.0, 1.0, 1.0],
                [3.0, 3.0, 1.0],
            ],
            [[0u32, 1, 2, 0], [3, 4, 5, 3]],
        )
        .unwrap();
        assert_eq!(failure_count(&m, ValidationType::SelfIntersection), 2);
    }

    #[test]
    fn vertex_fan_3d_faces_are_not_self_intersecting() {
        // Two triangles sharing only vertex 0, bending apart.
        let m = PolygonMesh3D::from_parts(
            CoordinateFrame::Euclidean,
            vec![
                [0.0, 0.0, 0.0],
                [4.0, 0.0, 0.0],
                [0.0, 4.0, 0.0],
                [-4.0, 0.0, 1.0],
                [0.0, -4.0, 1.0],
            ],
            [[0u32, 1, 2, 0], [0, 3, 4, 0]],
        )
        .unwrap();
        assert!(is_success(&m, ValidationType::SelfIntersection));
    }

    #[test]
    fn mesh_3d_hole_outside_its_face_is_not_contained() {
        let m = face_with_hole(vec![
            [5.0, 5.0, 0.0],
            [5.0, 6.0, 0.0],
            [6.0, 6.0, 0.0],
            [6.0, 5.0, 0.0],
            [5.0, 5.0, 0.0],
        ]);
        assert_eq!(
            failure_count(&m, ValidationType::InteriorRingContainment),
            1
        );
    }

    #[test]
    fn mesh_3d_hole_inside_its_face_is_contained() {
        let m = face_with_hole(vec![
            [1.0, 1.0, 0.0],
            [1.0, 2.0, 0.0],
            [2.0, 2.0, 0.0],
            [2.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ]);
        assert!(is_success(&m, ValidationType::InteriorRingContainment));
    }

    #[test]
    fn zero_area_3d_face_ring_is_degenerate() {
        let m = PolygonMesh3D::from_parts(
            CoordinateFrame::Euclidean,
            vec![[0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [2.0, 2.0, 2.0]],
            [[0u32, 1, 2, 0]],
        )
        .unwrap();
        assert_eq!(failure_count(&m, ValidationType::Degenerate), 1);
    }
}
