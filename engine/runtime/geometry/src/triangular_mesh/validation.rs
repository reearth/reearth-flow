use super::{TriangularMesh2D, TriangularMesh3D, TriangularMesh3DData};
use crate::validation_next::{
    check_duplicate_points_2d, check_duplicate_points_3d, check_edge_orientation_3d,
    check_finite_2d, check_finite_3d, check_ring_orientation_2d, tetra_volume_6x, CheckOutcome,
    FaceTopology, Validate, ValidationType,
};
use crate::{Euclidean3DGeometry, Geometry};

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
    /// shared by exactly two triangles — a watertight closed 2-manifold.
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
const TRIANGULAR_MESH_2D_CHECKS: [ValidationType; 4] = [
    ValidationType::Finite,
    ValidationType::Degenerate,
    ValidationType::DuplicatePoints,
    ValidationType::Orientation,
];

/// The checks that apply to a 3D triangle mesh — the 2D set plus `Orientable`.
const TRIANGULAR_MESH_3D_CHECKS: [ValidationType; 5] = [
    ValidationType::Finite,
    ValidationType::Degenerate,
    ValidationType::DuplicatePoints,
    ValidationType::Orientation,
    ValidationType::Orientable,
];

impl Validate for TriangularMesh2D {
    fn applicable_checks(&self) -> &'static [ValidationType] {
        &TRIANGULAR_MESH_2D_CHECKS
    }

    fn check_finite(&self) -> CheckOutcome {
        CheckOutcome::ran(|r| check_finite_2d(&self.frame, &self.vertices, self.z.as_deref(), r))
    }

    fn check_orientation(&self) -> CheckOutcome {
        // Each triangle should wind counter-clockwise.
        CheckOutcome::ran(|r| {
            for [a, b, c] in self.triangles() {
                let ring = [
                    self.vertices[a as usize],
                    self.vertices[b as usize],
                    self.vertices[c as usize],
                ];
                check_ring_orientation_2d(&self.frame, &ring, true, r);
            }
        })
    }

    fn check_duplicate_points(&self) -> CheckOutcome {
        // Coincident vertices in the shared pool are a defect.
        CheckOutcome::ran(|r| {
            check_duplicate_points_2d(&self.frame, self.vertices.iter().copied(), None, r)
        })
    }
}

impl Validate for TriangularMesh3D {
    fn applicable_checks(&self) -> &'static [ValidationType] {
        &TRIANGULAR_MESH_3D_CHECKS
    }

    fn check_finite(&self) -> CheckOutcome {
        CheckOutcome::ran(|r| check_finite_3d(&self.frame, self.data.vertices().iter().copied(), r))
    }

    fn check_orientation(&self) -> CheckOutcome {
        // Adjacent triangles must wind coherently across each shared edge.
        CheckOutcome::ran(|r| {
            check_edge_orientation_3d(&self.frame, self.data.vertices(), self.triangles(), r)
        })
    }

    fn check_orientable(&self) -> CheckOutcome {
        // A non-orientable mesh has no valid winding; report the whole mesh.
        CheckOutcome::ran(|r| {
            if !self.data.is_orientable() {
                r.push(
                    ValidationType::Orientable.to_string(),
                    Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(Box::new(
                        self.clone(),
                    ))),
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
    use crate::coordinate::CoordinateFrame;
    use crate::validation_next::{validate_one, ValidationResult};

    // Each helper runs just `check` (and its prerequisites) on the mesh, not the
    // leaf's other, still-unimplemented checks.
    fn is_success<T: Validate>(m: &T, check: ValidationType) -> bool {
        validate_one(m, check) == ValidationResult::Success
    }

    /// The failing positions of `check` on `m`, or a panic if it did not fail.
    fn failures<T: Validate>(m: &T, check: ValidationType) -> Vec<Geometry> {
        match validate_one(m, check) {
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
        // Edge 0-1 is shared by three triangles — no consistent orientation.
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
}
