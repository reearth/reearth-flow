use super::{PolygonMesh2D, PolygonMesh3D, PolygonMesh3DData};
use crate::coordinate::CoordinateFrame;
use crate::index::IndexBuffer;
use crate::validation_next::{
    check_duplicate_points, check_finite_2d, check_finite_3d, check_ring_orientation_2d,
    check_too_few_points_2d, check_too_few_points_3d, check_unclosed_ring_2d,
    check_unclosed_ring_3d, open_ring, tetra_volume_6x, EdgeOrientation, FaceOrientation,
    FaceTopology, Validate, ValidationParams, ValidationReport, ValidationType,
};
use crate::{Euclidean3DGeometry, Geometry};

/// Decode the CSR face topology and invoke `f` once per face ring — each face's
/// exterior ring, then its hole rings — passing the ring's vertex indices and
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
        // clockwise.
        ValidationReport::ran(|r| {
            for_each_ring(
                &self.face_indices,
                &self.face_offsets,
                &self.interior_offsets,
                |ring, is_exterior| {
                    let coords = ring_coords(&self.vertices, ring);
                    check_ring_orientation_2d(&self.frame, &coords, is_exterior, r);
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
}

impl Validate for PolygonMesh3D {
    fn applicable_checks(&self) -> &'static [ValidationType] {
        &POLYGON_MESH_3D_CHECKS
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
}
