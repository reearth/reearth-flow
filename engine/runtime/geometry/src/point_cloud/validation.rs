use super::ops::segment_positions;
use super::PointCloud;
use crate::validation_next::{
    check_duplicate_points_3d, check_finite_3d, CheckOutcome, Validate, ValidationType,
};

// All segments share the cloud's frame; stream each segment's decoded positions
// rather than materializing the whole cloud. Finiteness and coincident samples
// (`DuplicatePoints`) are the only checks that apply.
impl Validate for PointCloud {
    fn applicable_checks(&self) -> &'static [ValidationType] {
        &[ValidationType::Finite, ValidationType::DuplicatePoints]
    }

    fn check_finite(&self) -> CheckOutcome {
        CheckOutcome::ran(|r| {
            check_finite_3d(
                &self.frame,
                self.segments.iter().flat_map(segment_positions),
                r,
            )
        })
    }

    fn check_duplicate_points(&self) -> CheckOutcome {
        CheckOutcome::ran(|r| {
            check_duplicate_points_3d(
                &self.frame,
                self.segments.iter().flat_map(segment_positions),
                None,
                r,
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::CoordinateFrame;
    use crate::validation_next::{validate_leaf, ValidationResult, ValidationResults};
    use crate::{Euclidean3DGeometry, Geometry};

    /// The failing positions recorded for `check`, or a panic if it did not fail.
    fn failures(results: &ValidationResults, check: ValidationType) -> Vec<Geometry> {
        match &results[&check] {
            ValidationResult::Failed(positions) => positions.clone(),
            other => panic!("expected {check} to fail, got {other:?}"),
        }
    }

    #[test]
    fn finite_point_cloud_is_valid() {
        let pc = PointCloud::from_positions(
            CoordinateFrame::Euclidean,
            [[0.0, 1.0, 2.0], [3.0, 4.0, 5.0]],
        );
        assert_eq!(
            validate_leaf(&pc)[&ValidationType::Finite],
            ValidationResult::Success
        );
    }

    #[test]
    fn non_finite_samples_are_reported_at_their_positions() {
        let pc = PointCloud::from_positions(
            CoordinateFrame::Euclidean,
            [
                [0.0, 1.0, 2.0],
                [f64::NAN, 4.0, 5.0],
                [6.0, f64::INFINITY, 8.0],
            ],
        );
        let positions = failures(&validate_leaf(&pc), ValidationType::Finite);
        assert_eq!(positions.len(), 2);
        // Each problem is positioned at the offending sample as a 3D point.
        let bad = match &positions[0] {
            Geometry::Euclidean3D(Euclidean3DGeometry::Point(p)) => p.position(),
            other => panic!("expected a 3D point position, got {other:?}"),
        };
        assert!(bad[0].is_nan());
        assert_eq!(bad[1], 4.0);
    }

    #[test]
    fn coincident_samples_are_reported_as_duplicates() {
        let pc = PointCloud::from_positions(
            CoordinateFrame::Euclidean,
            [[0.0, 1.0, 2.0], [3.0, 4.0, 5.0], [0.0, 1.0, 2.0]],
        );
        assert_eq!(
            failures(&validate_leaf(&pc), ValidationType::DuplicatePoints).len(),
            1
        );
    }
}
