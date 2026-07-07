use super::{TriangularMesh2D, TriangularMesh3D};
use crate::validation_next::ValidationType;
use crate::validation_next::{check_finite_2d, check_finite_3d, Validate, ValidationReport};

impl Validate for TriangularMesh2D {
    fn validate(&self, _valid_type: ValidationType) -> Option<ValidationReport> {
        let mut report = ValidationReport::default();
        check_finite_2d(&self.frame, &self.vertices, self.z.as_deref(), &mut report);
        // TODO(new-geometry validation): implement the `_valid_type` checks over
        // the mesh's triangles (`Degenerate` / zero-area, `DuplicatePoints`,
        // `Orientation`) plus `Connected` over the whole mesh. A triangle is
        // always planar, so `Planarity` does not apply.
        report.into_option()
    }
}

impl Validate for TriangularMesh3D {
    fn validate(&self, _valid_type: ValidationType) -> Option<ValidationReport> {
        let mut report = ValidationReport::default();
        check_finite_3d(&self.frame, self.data.vertices(), &mut report);
        // TODO(new-geometry validation): see `TriangularMesh2D`.
        report.into_option()
    }
}
