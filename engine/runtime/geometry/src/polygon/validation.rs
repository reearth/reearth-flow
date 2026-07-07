use super::{Polygon2D, Polygon3D};
use crate::validation_next::{
    check_duplicate_points_2d, check_duplicate_points_3d, check_finite_2d, check_finite_3d,
    check_too_few_points_2d, check_too_few_points_3d, check_unclosed_ring_2d,
    check_unclosed_ring_3d, Validate, ValidationReport, ValidationType,
};

/// A ring stored closed (first == last) with its closing vertex dropped, so the
/// mandatory closure is not itself reported as a duplicate. Open rings pass
/// through unchanged.
macro_rules! open_ring {
    ($ring:expr) => {{
        let r = $ring;
        match r.split_last() {
            Some((last, head)) if !head.is_empty() && r.first() == Some(last) => head,
            _ => r,
        }
    }};
}

impl Validate for Polygon2D {
    fn validate(&self, valid_type: ValidationType) -> Option<ValidationReport> {
        // `coords` holds the exterior ring then all interior rings concatenated;
        // finiteness is a per-coordinate property, so scanning the whole buffer
        // covers every ring at once.
        let mut report = ValidationReport::default();
        check_finite_2d(&self.frame, &self.coords, self.z.as_deref(), &mut report);
        match &valid_type {
            // Each ring is closed, so it needs at least four coordinates.
            ValidationType::TooFewPoints => {
                check_too_few_points_2d(&self.frame, self.exterior(), true, &mut report);
                for hole in self.interiors() {
                    check_too_few_points_2d(&self.frame, hole, true, &mut report);
                }
            }
            // Every ring — exterior and holes — must close (first == last).
            ValidationType::UnclosedRing => {
                check_unclosed_ring_2d(&self.frame, self.exterior(), &mut report);
                for hole in self.interiors() {
                    check_unclosed_ring_2d(&self.frame, hole, &mut report);
                }
            }
            // Coincidences are per ring, excluding the closing vertex; elevation
            // is not considered.
            ValidationType::DuplicatePoints { tolerance } => {
                for ring in std::iter::once(self.exterior()).chain(self.interiors()) {
                    check_duplicate_points_2d(
                        &self.frame,
                        open_ring!(ring).iter().copied(),
                        *tolerance,
                        &mut report,
                    );
                }
            }
            _ => {}
        }
        report.into_option()
    }
}

impl Validate for Polygon3D {
    fn validate(&self, valid_type: ValidationType) -> Option<ValidationReport> {
        let mut report = ValidationReport::default();
        check_finite_3d(&self.frame, self.coords.iter().copied(), &mut report);
        match &valid_type {
            ValidationType::TooFewPoints => {
                check_too_few_points_3d(&self.frame, self.exterior(), true, &mut report);
                for hole in self.interiors() {
                    check_too_few_points_3d(&self.frame, hole, true, &mut report);
                }
            }
            ValidationType::UnclosedRing => {
                check_unclosed_ring_3d(&self.frame, self.exterior(), &mut report);
                for hole in self.interiors() {
                    check_unclosed_ring_3d(&self.frame, hole, &mut report);
                }
            }
            ValidationType::DuplicatePoints { tolerance } => {
                for ring in std::iter::once(self.exterior()).chain(self.interiors()) {
                    check_duplicate_points_3d(
                        &self.frame,
                        open_ring!(ring).iter().copied(),
                        *tolerance,
                        &mut report,
                    );
                }
            }
            _ => {}
        }
        report.into_option()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::CoordinateFrame;
    use crate::polygon::Polygon2D;

    fn square() -> [[f64; 2]; 5] {
        [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0], [0.0, 0.0]]
    }

    #[test]
    fn closed_ring_closure_is_not_a_duplicate() {
        let p = Polygon2D::from_rings(
            CoordinateFrame::Euclidean,
            square(),
            Vec::<Vec<[f64; 2]>>::new(),
        );
        assert!(p
            .validate(ValidationType::DuplicatePoints { tolerance: None })
            .is_none());
    }

    #[test]
    fn interior_duplicate_beyond_closure_is_reported() {
        // A repeated interior vertex (not the closure) is a real duplicate.
        let ring = [[0.0, 0.0], [4.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 0.0]];
        let p = Polygon2D::from_rings(
            CoordinateFrame::Euclidean,
            ring,
            Vec::<Vec<[f64; 2]>>::new(),
        );
        let report = p
            .validate(ValidationType::DuplicatePoints { tolerance: None })
            .unwrap();
        assert_eq!(report.error_count(), 1);
    }

    #[test]
    fn degenerate_ring_is_too_few_points() {
        // Three stored coords (< 4) cannot be a closed ring.
        let ring = [[0.0, 0.0], [1.0, 0.0], [0.0, 0.0]];
        let p = Polygon2D::from_rings(
            CoordinateFrame::Euclidean,
            ring,
            Vec::<Vec<[f64; 2]>>::new(),
        );
        let report = p.validate(ValidationType::TooFewPoints).unwrap();
        assert_eq!(report.error_count(), 1);
        assert_eq!(report.0[0].problem, "TooFewPoints");
    }

    #[test]
    fn too_few_points_checks_each_ring() {
        // Valid exterior, degenerate hole → one problem for the hole.
        let hole = vec![[1.0, 1.0], [2.0, 1.0], [1.0, 1.0]];
        let p = Polygon2D::from_rings(CoordinateFrame::Euclidean, square(), vec![hole]);
        let report = p.validate(ValidationType::TooFewPoints).unwrap();
        assert_eq!(report.error_count(), 1);
    }

    #[test]
    fn closed_ring_passes_unclosed_check() {
        let p = Polygon2D::from_rings(
            CoordinateFrame::Euclidean,
            square(),
            Vec::<Vec<[f64; 2]>>::new(),
        );
        assert!(p.validate(ValidationType::UnclosedRing).is_none());
    }

    #[test]
    fn open_ring_is_reported_unclosed() {
        // Exterior first != last.
        let open = [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]];
        let p = Polygon2D::from_rings(
            CoordinateFrame::Euclidean,
            open,
            Vec::<Vec<[f64; 2]>>::new(),
        );
        let report = p.validate(ValidationType::UnclosedRing).unwrap();
        assert_eq!(report.error_count(), 1);
        assert_eq!(report.0[0].problem, "UnclosedRing");
        assert!(matches!(
            report.0[0].position,
            crate::Geometry::Euclidean2D(crate::Euclidean2DGeometry::LineString(_))
        ));
    }

    #[test]
    fn unclosed_check_covers_holes() {
        // Closed exterior, open hole → one problem for the hole.
        let open_hole = vec![[1.0, 1.0], [2.0, 1.0], [2.0, 2.0]];
        let p = Polygon2D::from_rings(CoordinateFrame::Euclidean, square(), vec![open_hole]);
        let report = p.validate(ValidationType::UnclosedRing).unwrap();
        assert_eq!(report.error_count(), 1);
    }
}
