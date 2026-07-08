use super::{Point2D, Point3D};
use crate::validation_next::{
    check_finite_2d, check_finite_3d, Validate, ValidationReport, ValidationType,
};

impl Validate for Point2D {
    fn applicable_checks(&self) -> &'static [ValidationType] {
        // A point carries a single coordinate: only finiteness applies.
        &[ValidationType::Finite]
    }

    fn check_finite(&self) -> ValidationReport {
        ValidationReport::ran(|r| {
            check_finite_2d(&self.frame, std::slice::from_ref(&self.position), None, r)
        })
    }
}

impl Validate for Point3D {
    fn applicable_checks(&self) -> &'static [ValidationType] {
        &[ValidationType::Finite]
    }

    fn check_finite(&self) -> ValidationReport {
        ValidationReport::ran(|r| check_finite_3d(&self.frame, [self.position], r))
    }
}
