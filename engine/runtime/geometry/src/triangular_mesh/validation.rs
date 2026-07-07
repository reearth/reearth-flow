use super::{TriangularMesh2D, TriangularMesh3D};
use crate::validation_next::{
    check_duplicate_points_2d, check_duplicate_points_3d, check_finite_2d, check_finite_3d,
    Validate, ValidationReport, ValidationType,
};

impl Validate for TriangularMesh2D {
    fn validate(&self, valid_type: ValidationType) -> Option<ValidationReport> {
        let mut report = ValidationReport::default();
        check_finite_2d(&self.frame, &self.vertices, self.z.as_deref(), &mut report);
        // A triangle always has three corners, so `TooFewPoints` cannot apply;
        // duplicate coincident vertices in the shared pool are a defect.
        if let ValidationType::DuplicatePoints { tolerance } = &valid_type {
            check_duplicate_points_2d(
                &self.frame,
                self.vertices.iter().copied(),
                *tolerance,
                &mut report,
            );
        }
        report.into_option()
    }
}

impl Validate for TriangularMesh3D {
    fn validate(&self, valid_type: ValidationType) -> Option<ValidationReport> {
        let mut report = ValidationReport::default();
        check_finite_3d(
            &self.frame,
            self.data.vertices().iter().copied(),
            &mut report,
        );
        if let ValidationType::DuplicatePoints { tolerance } = &valid_type {
            check_duplicate_points_3d(
                &self.frame,
                self.data.vertices().iter().copied(),
                *tolerance,
                &mut report,
            );
        }
        report.into_option()
    }
}
