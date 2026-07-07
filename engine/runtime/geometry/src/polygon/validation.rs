use super::{Polygon2D, Polygon3D};
use crate::validation_next::ValidationType;
use crate::validation_next::{check_finite_2d, check_finite_3d, Validate, ValidationReport};

impl Validate for Polygon2D {
    fn validate(&self, _valid_type: ValidationType) -> Option<ValidationReport> {
        // `coords` holds the exterior ring then all interior rings concatenated;
        // finiteness is a per-coordinate property, so scanning the whole buffer
        // covers every ring at once.
        let mut report = ValidationReport::default();
        check_finite_2d("Polygon2D", &self.coords, self.z.as_deref(), &mut report);
        // TODO(new-geometry validation): implement the `_valid_type` checks —
        // `TooFewPoints`, `UnclosedRing`, `DuplicatePoints`,
        // `DuplicateConsecutivePoints`, `SelfIntersection`,
        // `InteriorRingContainment`, `Degenerate`, and `Orientation` (2D ring
        // winding) — per ring. (`Planarity` is 3D-only; see `Polygon3D`.)
        report.into_option()
    }
}

impl Validate for Polygon3D {
    fn validate(&self, _valid_type: ValidationType) -> Option<ValidationReport> {
        let mut report = ValidationReport::default();
        check_finite_3d("Polygon3D", &self.coords, &mut report);
        // TODO(new-geometry validation): as `Polygon2D`, plus `Planarity`
        // (out-of-plane deviation of the 3D face). A lone 3D face has no
        // absolute `Orientation`, so that check does not apply here.
        report.into_option()
    }
}
