use super::{LineString2D, LineString3D};
use crate::ops::validation::{check_finite_2d, check_finite_3d, Validate, ValidationReport};
use crate::ops::ValidationType;

impl Validate for LineString2D {
    fn validate(&self, _valid_type: ValidationType) -> Option<ValidationReport> {
        let mut report = ValidationReport::default();
        check_finite_2d("LineString2D", &self.coords, self.z.as_deref(), &mut report);
        // TODO(new-geometry validation): implement the `_valid_type` checks —
        // `TooFewPoints`, `DuplicatePoints`, `DuplicateConsecutivePoints`,
        // `SelfIntersection`, and `Degenerate` (zero-length) — for the polyline.
        report.into_option()
    }
}

impl Validate for LineString3D {
    fn validate(&self, _valid_type: ValidationType) -> Option<ValidationReport> {
        let mut report = ValidationReport::default();
        check_finite_3d("LineString3D", &self.coords, &mut report);
        // TODO(new-geometry validation): see `LineString2D`.
        report.into_option()
    }
}
