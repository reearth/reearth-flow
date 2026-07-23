use super::{LineString2D, LineString3D};
use crate::validation_next::{
    check_chain_simple_2d, check_chain_simple_3d, check_degenerate_chain_2d,
    check_degenerate_chain_3d, check_duplicate_points, check_finite_2d, check_finite_3d,
    check_too_few_points_2d, check_too_few_points_3d, Validate, ValidationParams, ValidationReport,
    ValidationType,
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

    fn check_finite(&self, _params: &ValidationParams) -> ValidationReport {
        ValidationReport::ran(|r| check_finite_2d(&self.frame, &self.coords, self.z.as_deref(), r))
    }

    fn check_too_few_points(&self, _params: &ValidationParams) -> ValidationReport {
        // A polyline is open: it needs at least two points.
        ValidationReport::ran(|r| check_too_few_points_2d(&self.frame, &self.coords, false, r))
    }

    fn check_duplicate_points(&self, params: &ValidationParams) -> ValidationReport {
        ValidationReport::ran(|r| {
            check_duplicate_points(
                &self.frame,
                self.coords.iter().copied(),
                params.duplicate_tolerance,
                r,
            )
        })
    }

    fn check_self_intersection(&self, _params: &ValidationParams) -> ValidationReport {
        ValidationReport::ran(|r| check_chain_simple_2d(&self.frame, &self.coords, r))
    }

    fn check_degenerate(&self, params: &ValidationParams) -> ValidationReport {
        ValidationReport::ran(|r| {
            check_degenerate_chain_2d(&self.frame, &self.coords, params.degenerate.min_length, r)
        })
    }
}

impl Validate for LineString3D {
    fn applicable_checks(&self) -> &'static [ValidationType] {
        &LINE_STRING_CHECKS
    }

    fn check_finite(&self, _params: &ValidationParams) -> ValidationReport {
        ValidationReport::ran(|r| check_finite_3d(&self.frame, self.coords.iter().copied(), r))
    }

    fn check_too_few_points(&self, _params: &ValidationParams) -> ValidationReport {
        ValidationReport::ran(|r| check_too_few_points_3d(&self.frame, &self.coords, false, r))
    }

    fn check_duplicate_points(&self, params: &ValidationParams) -> ValidationReport {
        ValidationReport::ran(|r| {
            check_duplicate_points(
                &self.frame,
                self.coords.iter().copied(),
                params.duplicate_tolerance,
                r,
            )
        })
    }

    fn check_self_intersection(&self, _params: &ValidationParams) -> ValidationReport {
        ValidationReport::ran(|r| check_chain_simple_3d(&self.frame, &self.coords, r))
    }

    fn check_degenerate(&self, params: &ValidationParams) -> ValidationReport {
        ValidationReport::ran(|r| {
            check_degenerate_chain_3d(&self.frame, &self.coords, params.degenerate.min_length, r)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::CoordinateFrame;
    use crate::validation_next::{validate_one, ValidationResult};
    use crate::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};

    fn line3(coords: impl IntoIterator<Item = [f64; 3]>) -> LineString3D {
        LineString3D::from_coords(CoordinateFrame::Euclidean, coords)
    }

    fn line2(coords: impl IntoIterator<Item = [f64; 2]>) -> LineString2D {
        LineString2D::from_coords(CoordinateFrame::Euclidean, coords)
    }

    fn params() -> ValidationParams {
        ValidationParams::default()
    }

    /// The failing positions of one check, or a panic if it did not fail.
    fn failures<T: Validate>(leaf: &T, check: ValidationType) -> Vec<Geometry> {
        match validate_one(leaf, check, &params()) {
            ValidationResult::Failed(positions) => positions,
            other => panic!("expected {check} to fail, got {other:?}"),
        }
    }

    fn is_success<T: Validate>(leaf: &T, check: ValidationType) -> bool {
        validate_one(leaf, check, &params()) == ValidationResult::Success
    }

    #[test]
    fn straight_chain_is_simple() {
        let ls = line3([[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 1.0, 0.0]]);
        assert!(is_success(&ls, ValidationType::SelfIntersection));
    }

    #[test]
    fn figure_eight_crossing_is_flagged_at_the_point() {
        let ls = line2([[0.0, 0.0], [2.0, 2.0], [2.0, 0.0], [0.0, 2.0]]);
        let positions = failures(&ls, ValidationType::SelfIntersection);
        assert_eq!(positions.len(), 1);
        match &positions[0] {
            Geometry::Euclidean2D(Euclidean2DGeometry::Point(p)) => {
                assert_eq!(p.position(), [1.0, 1.0]);
            }
            other => panic!("expected a 2D point, got {other:?}"),
        }
    }

    #[test]
    fn spike_folds_back_and_is_flagged() {
        let ls = line3([[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        assert_eq!(failures(&ls, ValidationType::SelfIntersection).len(), 1);
        let flat = line2([[0.0, 0.0], [2.0, 0.0], [1.0, 0.0]]);
        assert_eq!(failures(&flat, ValidationType::SelfIntersection).len(), 1);
    }

    #[test]
    fn collinear_revisit_is_flagged() {
        // The last segment rides back over the first.
        let ls = line2([[0.0, 0.0], [4.0, 0.0], [4.0, 2.0], [2.0, 0.0], [-1.0, 0.0]]);
        assert!(!failures(&ls, ValidationType::SelfIntersection).is_empty());
    }

    #[test]
    fn closed_loop_is_simple() {
        let ls = line3([
            [0.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [2.0, 2.0, 0.0],
            [0.0, 0.0, 0.0],
        ]);
        assert!(is_success(&ls, ValidationType::SelfIntersection));
    }

    #[test]
    fn closed_loop_crossing_itself_is_flagged() {
        // A closed bowtie chain.
        let ls = line2([[0.0, 0.0], [2.0, 2.0], [2.0, 0.0], [0.0, 2.0], [0.0, 0.0]]);
        assert!(!failures(&ls, ValidationType::SelfIntersection).is_empty());
    }

    #[test]
    fn mid_chain_vertex_touch_is_flagged() {
        // The chain returns to an earlier interior vertex without closing.
        let ls = line2([[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [1.0, 0.0]]);
        assert!(!failures(&ls, ValidationType::SelfIntersection).is_empty());
    }

    #[test]
    fn consecutive_duplicates_alone_are_simple() {
        let ls = line3([
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 1.0, 0.0],
        ]);
        assert!(is_success(&ls, ValidationType::SelfIntersection));
    }

    #[test]
    fn skew_3d_chain_is_simple_despite_crossing_in_projection() {
        // The XY projection crosses, but the segments are skew in 3D.
        let ls = line3([
            [0.0, 0.0, 0.0],
            [2.0, 2.0, 0.0],
            [2.0, 0.0, 5.0],
            [0.0, 2.0, 5.0],
        ]);
        assert!(is_success(&ls, ValidationType::SelfIntersection));
    }

    #[test]
    fn zero_length_chain_is_degenerate() {
        let ls = line3([[1.0, 1.0, 1.0], [1.0, 1.0, 1.0]]);
        let positions = failures(&ls, ValidationType::Degenerate);
        assert_eq!(positions.len(), 1);
        assert!(matches!(
            positions[0],
            Geometry::Euclidean3D(Euclidean3DGeometry::LineString(_))
        ));
    }

    #[test]
    fn positive_length_chain_is_not_degenerate() {
        let ls = line2([[0.0, 0.0], [3.0, 4.0]]);
        assert!(is_success(&ls, ValidationType::Degenerate));
    }

    #[test]
    fn min_length_threshold_is_inclusive() {
        let ls = line2([[0.0, 0.0], [3.0, 4.0]]);
        let strict = ValidationParams {
            degenerate: crate::validation_next::DegenerateThresholds {
                min_length: 5.0,
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(matches!(
            validate_one(&ls, ValidationType::Degenerate, &strict),
            ValidationResult::Failed(_)
        ));
        let lenient = ValidationParams {
            degenerate: crate::validation_next::DegenerateThresholds {
                min_length: 4.9,
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(
            validate_one(&ls, ValidationType::Degenerate, &lenient),
            ValidationResult::Success
        );
    }
}
