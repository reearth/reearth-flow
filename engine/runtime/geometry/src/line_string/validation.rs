use super::{LineString2D, LineString3D};
use crate::validation_next::{
    check_duplicate_points_2d, check_duplicate_points_3d, check_finite_2d, check_finite_3d,
    check_too_few_points_2d, check_too_few_points_3d, Validate, ValidationReport, ValidationType,
};

impl Validate for LineString2D {
    fn validate(&self, valid_type: ValidationType) -> Option<ValidationReport> {
        let mut report = ValidationReport::default();
        check_finite_2d(&self.frame, &self.coords, self.z.as_deref(), &mut report);
        match &valid_type {
            // A polyline is open: it needs at least two points.
            ValidationType::TooFewPoints => {
                check_too_few_points_2d(&self.frame, &self.coords, false, &mut report)
            }
            ValidationType::DuplicatePoints { tolerance } => check_duplicate_points_2d(
                &self.frame,
                self.coords.iter().copied(),
                *tolerance,
                &mut report,
            ),
            _ => {}
        }
        report.into_option()
    }
}

impl Validate for LineString3D {
    fn validate(&self, valid_type: ValidationType) -> Option<ValidationReport> {
        let mut report = ValidationReport::default();
        check_finite_3d(&self.frame, self.coords.iter().copied(), &mut report);
        match &valid_type {
            ValidationType::TooFewPoints => {
                check_too_few_points_3d(&self.frame, &self.coords, false, &mut report)
            }
            ValidationType::DuplicatePoints { tolerance } => check_duplicate_points_3d(
                &self.frame,
                self.coords.iter().copied(),
                *tolerance,
                &mut report,
            ),
            _ => {}
        }
        report.into_option()
    }
}
