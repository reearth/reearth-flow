use super::{Point2D, Point3D};
use crate::validation_next::ValidationType;
use crate::validation_next::{check_finite_2d, check_finite_3d, Validate, ValidationReport};

impl Validate for Point2D {
    fn validate(&self, _valid_type: ValidationType) -> Option<ValidationReport> {
        // A point carries a single coordinate: only finiteness applies. The
        // duplicate / structural checks selected by `_valid_type` are defined
        // over multi-coordinate geometries and are no-ops here.
        let mut report = ValidationReport::default();
        check_finite_2d(
            "Point2D",
            std::slice::from_ref(&self.position),
            None,
            &mut report,
        );
        report.into_option()
    }
}

impl Validate for Point3D {
    fn validate(&self, _valid_type: ValidationType) -> Option<ValidationReport> {
        let mut report = ValidationReport::default();
        check_finite_3d("Point3D", std::slice::from_ref(&self.position), &mut report);
        report.into_option()
    }
}
