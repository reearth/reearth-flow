use super::{PolygonMesh2D, PolygonMesh3D, PolygonMesh3DData};
use crate::coordinate::CoordinateFrame;
use crate::index::IndexBuffer;
use crate::validation_next::{
    check_duplicate_points_2d, check_duplicate_points_3d, check_edge_orientation_3d,
    check_finite_2d, check_finite_3d, check_ring_orientation_2d, check_too_few_points_2d,
    check_too_few_points_3d, check_unclosed_ring_2d, check_unclosed_ring_3d, tetra_volume_6x,
    CheckOutcome, FaceTopology, Validate, ValidationReport, ValidationType,
};
use crate::{Euclidean3DGeometry, Geometry};

/// One decoded face ring: its `(start, end)` range into the flat vertex-index
/// buffer and whether it is a face's exterior ring (vs. a hole).
struct Ring {
    start: usize,
    end: usize,
    is_exterior: bool,
}

/// Decode the CSR face topology into the flat vertex-index buffer plus one
/// [`Ring`] per ring — each face's exterior ring, then its hole rings.
fn ring_ranges(
    face_indices: &IndexBuffer<1>,
    face_offsets: &IndexBuffer<1>,
    interior_offsets: &IndexBuffer<1>,
) -> (Vec<u32>, Vec<Ring>) {
    let indices: Vec<u32> = face_indices.iter_u32().map(|[i]| i).collect();
    let n = indices.len();
    let mut ranges = Vec::new();
    if n == 0 {
        return (indices, ranges);
    }
    let face_ends: Vec<usize> = face_offsets.iter_u32().map(|[i]| i as usize).collect();
    let holes: Vec<usize> = interior_offsets.iter_u32().map(|[i]| i as usize).collect();
    let n_faces = face_ends.len() + 1;
    let mut start = 0usize;
    for f in 0..n_faces {
        let end = face_ends.get(f).copied().unwrap_or(n);
        // Hole rings of this face begin at the interior offsets inside (start, end).
        let mut ring_start = start;
        let mut is_exterior = true;
        for &h in holes.iter().filter(|&&h| h > start && h < end) {
            ranges.push(Ring {
                start: ring_start,
                end: h,
                is_exterior,
            });
            ring_start = h;
            is_exterior = false;
        }
        ranges.push(Ring {
            start: ring_start,
            end,
            is_exterior,
        });
        start = end;
    }
    (indices, ranges)
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
        let (indices, ranges) = ring_ranges(
            &self.face_indices,
            &self.face_offsets,
            &self.interior_offsets,
        );
        for Ring { start, end, .. } in ranges {
            if end - start < 4 {
                let coords: Vec<[f64; 3]> = indices[start..end]
                    .iter()
                    .map(|&i| self.vertices[i as usize])
                    .collect();
                check_too_few_points_3d(frame, &coords, true, report);
            }
        }
    }

    /// Report a [`ValidationType::UnclosedRing`] problem for every face ring whose
    /// first and last vertices differ. Shared by the [`PolygonMesh3D`] leaf and
    /// [`Solid`](crate::solid::Solid) shells, which supply the frame.
    pub(crate) fn check_unclosed_rings(
        &self,
        frame: &CoordinateFrame,
        report: &mut ValidationReport,
    ) {
        let (indices, ranges) = ring_ranges(
            &self.face_indices,
            &self.face_offsets,
            &self.interior_offsets,
        );
        for Ring { start, end, .. } in ranges {
            // Only materialize the ring when its endpoints actually differ.
            if end > start
                && self.vertices[indices[start] as usize]
                    != self.vertices[indices[end - 1] as usize]
            {
                let coords: Vec<[f64; 3]> = indices[start..end]
                    .iter()
                    .map(|&i| self.vertices[i as usize])
                    .collect();
                check_unclosed_ring_3d(frame, &coords, report);
            }
        }
    }

    /// Report a [`ValidationType::Orientation`] problem for face rings that wind
    /// inconsistently with a neighbour across a shared edge. Shared by the
    /// [`PolygonMesh3D`] leaf and [`Solid`](crate::solid::Solid) shells.
    pub(crate) fn check_orientation(&self, frame: &CoordinateFrame, report: &mut ValidationReport) {
        let (indices, ranges) = ring_ranges(
            &self.face_indices,
            &self.face_offsets,
            &self.interior_offsets,
        );
        check_edge_orientation_3d(
            frame,
            &self.vertices,
            ranges.iter().map(|r| &indices[r.start..r.end]),
            report,
        );
    }

    /// The face-adjacency topology of this mesh, one face per decoded ring.
    fn topology(&self) -> FaceTopology {
        let (indices, ranges) = ring_ranges(
            &self.face_indices,
            &self.face_offsets,
            &self.interior_offsets,
        );
        FaceTopology::from_faces(
            ranges
                .iter()
                .map(|r| indices[r.start..r.end].to_vec())
                .collect::<Vec<_>>(),
        )
    }

    /// Whether the mesh admits a consistent orientation (see
    /// [`ValidationType::Orientable`]).
    pub(crate) fn is_orientable(&self) -> bool {
        self.topology().is_orientable()
    }

    /// Whether the mesh is a single connected component whose every edge is
    /// shared by exactly two faces — a watertight closed 2-manifold.
    pub(crate) fn is_closed_connected_manifold(&self) -> bool {
        let topo = self.topology();
        topo.is_closed_manifold() && topo.is_connected()
    }

    /// The signed volume enclosed by this mesh, taken as a closed surface: each
    /// face is fan-triangulated and its tetrahedra summed. Positive = outward
    /// normals. Meaningful only once the mesh is a closed, oriented shell.
    pub(crate) fn signed_volume(&self) -> f64 {
        let (indices, ranges) = ring_ranges(
            &self.face_indices,
            &self.face_offsets,
            &self.interior_offsets,
        );
        let mut acc = 0.0;
        for Ring { start, end, .. } in ranges {
            let ring = &indices[start..end];
            // Drop the closing vertex so the fan uses only distinct corners.
            let ring = match ring.split_last() {
                Some((last, head)) if !head.is_empty() && ring.first() == Some(last) => head,
                _ => ring,
            };
            if ring.len() < 3 {
                continue;
            }
            let p0 = self.vertices[ring[0] as usize];
            for k in 1..ring.len() - 1 {
                let p1 = self.vertices[ring[k] as usize];
                let p2 = self.vertices[ring[k + 1] as usize];
                acc += tetra_volume_6x(p0, p1, p2);
            }
        }
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

/// The checks that apply to a 3D polygon mesh — the 2D set plus `Orientable`.
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

    fn check_finite(&self) -> CheckOutcome {
        CheckOutcome::ran(|r| check_finite_2d(&self.frame, &self.vertices, self.z.as_deref(), r))
    }

    fn check_too_few_points(&self) -> CheckOutcome {
        CheckOutcome::ran(|r| {
            let (indices, ranges) = ring_ranges(
                &self.face_indices,
                &self.face_offsets,
                &self.interior_offsets,
            );
            for Ring { start, end, .. } in ranges {
                if end - start < 4 {
                    let coords: Vec<[f64; 2]> = indices[start..end]
                        .iter()
                        .map(|&i| self.vertices[i as usize])
                        .collect();
                    check_too_few_points_2d(&self.frame, &coords, true, r);
                }
            }
        })
    }

    fn check_unclosed_ring(&self) -> CheckOutcome {
        CheckOutcome::ran(|r| {
            let (indices, ranges) = ring_ranges(
                &self.face_indices,
                &self.face_offsets,
                &self.interior_offsets,
            );
            for Ring { start, end, .. } in ranges {
                if end > start
                    && self.vertices[indices[start] as usize]
                        != self.vertices[indices[end - 1] as usize]
                {
                    let coords: Vec<[f64; 2]> = indices[start..end]
                        .iter()
                        .map(|&i| self.vertices[i as usize])
                        .collect();
                    check_unclosed_ring_2d(&self.frame, &coords, r);
                }
            }
        })
    }

    fn check_orientation(&self) -> CheckOutcome {
        // Each face's exterior ring must wind counter-clockwise, its holes
        // clockwise.
        CheckOutcome::ran(|r| {
            let (indices, ranges) = ring_ranges(
                &self.face_indices,
                &self.face_offsets,
                &self.interior_offsets,
            );
            for Ring {
                start,
                end,
                is_exterior,
            } in ranges
            {
                let coords: Vec<[f64; 2]> = indices[start..end]
                    .iter()
                    .map(|&i| self.vertices[i as usize])
                    .collect();
                check_ring_orientation_2d(&self.frame, &coords, is_exterior, r);
            }
        })
    }

    fn check_duplicate_points(&self) -> CheckOutcome {
        // A shared vertex pool should hold no coincident vertices; elevation is
        // not considered.
        CheckOutcome::ran(|r| {
            check_duplicate_points_2d(&self.frame, self.vertices.iter().copied(), None, r)
        })
    }
}

impl Validate for PolygonMesh3D {
    fn applicable_checks(&self) -> &'static [ValidationType] {
        &POLYGON_MESH_3D_CHECKS
    }

    fn check_finite(&self) -> CheckOutcome {
        CheckOutcome::ran(|r| check_finite_3d(&self.frame, self.data.vertices().iter().copied(), r))
    }

    fn check_too_few_points(&self) -> CheckOutcome {
        CheckOutcome::ran(|r| self.data.check_too_few_points(&self.frame, r))
    }

    fn check_unclosed_ring(&self) -> CheckOutcome {
        CheckOutcome::ran(|r| self.data.check_unclosed_rings(&self.frame, r))
    }

    fn check_orientation(&self) -> CheckOutcome {
        CheckOutcome::ran(|r| self.data.check_orientation(&self.frame, r))
    }

    fn check_orientable(&self) -> CheckOutcome {
        // A non-orientable mesh has no valid winding; report the whole mesh.
        CheckOutcome::ran(|r| {
            if !self.data.is_orientable() {
                r.push(
                    ValidationType::Orientable.to_string(),
                    Geometry::Euclidean3D(Euclidean3DGeometry::PolygonMesh(Box::new(self.clone()))),
                );
            }
        })
    }

    fn check_duplicate_points(&self) -> CheckOutcome {
        CheckOutcome::ran(|r| {
            check_duplicate_points_3d(&self.frame, self.data.vertices().iter().copied(), None, r)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::polygon_mesh::{PolygonMesh2D, PolygonMesh3D};
    use crate::validation_next::{validate_one, ValidationResult};

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
        validate_one(m, check) == ValidationResult::Success
    }

    fn failure_count<T: Validate>(m: &T, check: ValidationType) -> usize {
        match validate_one(m, check) {
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
}
