use super::{LineString2D, LineString3D};
use crate::validation_next::{
    check_duplicate_points, check_finite_2d, check_finite_3d, check_too_few_points_2d,
    check_too_few_points_3d, Validate, ValidationReport, ValidationType,
};

/// The checks that apply to a line string; 2D and 3D share the table row.
const LINE_STRING_CHECKS: [ValidationType; 5] = [
    ValidationType::Finite,
    ValidationType::TooFewPoints,
    ValidationType::SelfIntersection,
    ValidationType::Degenerate,
    ValidationType::DuplicatePoints,
];

impl Validate for LineString2D {
    fn applicable_checks(&self) -> &'static [ValidationType] {
        &LINE_STRING_CHECKS
    }

    fn check_finite(&self) -> ValidationReport {
        ValidationReport::ran(|r| check_finite_2d(&self.frame, &self.coords, self.z.as_deref(), r))
    }

    fn check_too_few_points(&self) -> ValidationReport {
        // A polyline is open: it needs at least two points.
        ValidationReport::ran(|r| check_too_few_points_2d(&self.frame, &self.coords, false, r))
    }

    fn check_duplicate_points(&self) -> ValidationReport {
        ValidationReport::ran(|r| {
            check_duplicate_points(&self.frame, self.coords.iter().copied(), None, r)
        })
    }
}

impl Validate for LineString3D {
    fn applicable_checks(&self) -> &'static [ValidationType] {
        &LINE_STRING_CHECKS
    }

    fn check_finite(&self) -> ValidationReport {
        ValidationReport::ran(|r| check_finite_3d(&self.frame, self.coords.iter().copied(), r))
    }

    fn check_too_few_points(&self) -> ValidationReport {
        ValidationReport::ran(|r| check_too_few_points_3d(&self.frame, &self.coords, false, r))
    }

    fn check_duplicate_points(&self) -> ValidationReport {
        ValidationReport::ran(|r| {
            check_duplicate_points(&self.frame, self.coords.iter().copied(), None, r)
        })
    }
}
