use super::ops::segment_positions;
use super::PointCloud;
use crate::validation_next::{
    check_duplicate_points_3d, check_finite_3d, Validate, ValidationReport, ValidationType,
};

impl Validate for PointCloud {
    fn validate(&self, valid_type: ValidationType) -> Option<ValidationReport> {
        // All segments share the cloud's frame; stream each segment's decoded
        // positions rather than materializing the whole cloud.
        let mut report = ValidationReport::default();
        check_finite_3d(
            &self.frame,
            self.segments.iter().flat_map(segment_positions),
            &mut report,
        );
        // `DuplicatePoints` (coincident samples) is the only other check that
        // applies to a point cloud.
        if let ValidationType::DuplicatePoints { tolerance } = &valid_type {
            check_duplicate_points_3d(
                &self.frame,
                self.segments.iter().flat_map(segment_positions),
                *tolerance,
                &mut report,
            );
        }
        report.into_option()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::CoordinateFrame;
    use crate::{Euclidean3DGeometry, Geometry};

    #[test]
    fn finite_point_cloud_is_valid() {
        let pc = PointCloud::from_positions(
            CoordinateFrame::Euclidean,
            [[0.0, 1.0, 2.0], [3.0, 4.0, 5.0]],
        );
        assert!(pc.validate(ValidationType::Finite).is_none());
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
        let report = pc.validate(ValidationType::Finite).unwrap();
        assert_eq!(report.error_count(), 2);
        assert!(report.0.iter().all(|p| p.problem == "Finite"));
        // Each problem is positioned at the offending sample as a 3D point.
        let bad = match &report.0[0].position {
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
        let report = pc
            .validate(ValidationType::DuplicatePoints { tolerance: None })
            .unwrap();
        assert_eq!(report.error_count(), 1);
        assert_eq!(report.0[0].problem, "DuplicatePoints(tolerance: None)");
    }
}
