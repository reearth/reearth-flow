use super::{PolygonMesh2D, PolygonMesh3D};
use crate::validation_next::ValidationType;
use crate::validation_next::{check_finite_2d, check_finite_3d, Validate, ValidationReport};

impl Validate for PolygonMesh2D {
    fn validate(&self, _valid_type: ValidationType) -> Option<ValidationReport> {
        let mut report = ValidationReport::default();
        check_finite_2d(&self.frame, &self.vertices, self.z.as_deref(), &mut report);
        // TODO(new-geometry validation): implement the `_valid_type` checks over
        // the mesh's faces (`TooFewPoints`, `UnclosedRing`, `SelfIntersection`,
        // `InteriorRingContainment`, `Degenerate`, `Orientation`) plus
        // `Connected` over the whole mesh. `Planarity` is 3D-only (`PolygonMesh3D`).
        report.into_option()
    }
}

impl Validate for PolygonMesh3D {
    fn validate(&self, _valid_type: ValidationType) -> Option<ValidationReport> {
        let mut report = ValidationReport::default();
        check_finite_3d(&self.frame, self.data.vertices(), &mut report);
        // TODO(new-geometry validation): see `PolygonMesh2D`.
        report.into_option()
    }
}
