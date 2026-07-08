use super::{Polygon2D, Polygon3D};
use crate::validation_next::{
    check_duplicate_points, check_finite_2d, check_finite_3d, check_ring_orientation_2d,
    check_too_few_points_2d, check_too_few_points_3d, check_unclosed_ring_2d,
    check_unclosed_ring_3d, open_ring, Validate, ValidationParams, ValidationReport,
    ValidationType,
};

/// The checks that apply to a 2D polygon face. Planarity is omitted: a 2D face
/// lies in the coordinate plane by construction, so it is trivially planar.
const POLYGON_2D_CHECKS: [ValidationType; 8] = [
    ValidationType::Finite,
    ValidationType::TooFewPoints,
    ValidationType::UnclosedRing,
    ValidationType::SelfIntersection,
    ValidationType::InteriorRingContainment,
    ValidationType::Degenerate,
    ValidationType::DuplicatePoints,
    ValidationType::Orientation,
];

/// The checks that apply to a 3D polygon face: the 2D set with `Planarity`
/// (a face embedded in 3D can be non-planar) but without `Orientation`
/// (a face embedded in 3D has no signed winding).
const POLYGON_3D_CHECKS: [ValidationType; 8] = [
    ValidationType::Finite,
    ValidationType::TooFewPoints,
    ValidationType::UnclosedRing,
    ValidationType::SelfIntersection,
    ValidationType::InteriorRingContainment,
    ValidationType::Degenerate,
    ValidationType::Planarity,
    ValidationType::DuplicatePoints,
];

impl Validate for Polygon2D {
    fn applicable_checks(&self) -> &'static [ValidationType] {
        &POLYGON_2D_CHECKS
    }

    fn check_finite(&self, _params: &ValidationParams) -> ValidationReport {
        // `coords` holds the exterior ring then all interior rings concatenated;
        // finiteness is a per-coordinate property, so scanning the whole buffer
        // covers every ring at once.
        ValidationReport::ran(|r| check_finite_2d(&self.frame, &self.coords, self.z.as_deref(), r))
    }

    fn check_too_few_points(&self, _params: &ValidationParams) -> ValidationReport {
        // Each ring is closed, so it needs at least four coordinates.
        ValidationReport::ran(|r| {
            check_too_few_points_2d(&self.frame, self.exterior(), true, r);
            for hole in self.interiors() {
                check_too_few_points_2d(&self.frame, hole, true, r);
            }
        })
    }

    fn check_unclosed_ring(&self, _params: &ValidationParams) -> ValidationReport {
        // Every ring must close (first == last).
        ValidationReport::ran(|r| {
            check_unclosed_ring_2d(&self.frame, self.exterior(), r);
            for hole in self.interiors() {
                check_unclosed_ring_2d(&self.frame, hole, r);
            }
        })
    }

    fn check_duplicate_points(&self, params: &ValidationParams) -> ValidationReport {
        // Coincidences are per ring, excluding the closing vertex; elevation is
        // not considered.
        ValidationReport::ran(|r| {
            for ring in std::iter::once(self.exterior()).chain(self.interiors()) {
                check_duplicate_points(
                    &self.frame,
                    open_ring(ring).iter().copied(),
                    params.duplicate_tolerance,
                    r,
                );
            }
        })
    }

    fn check_orientation(&self, _params: &ValidationParams) -> ValidationReport {
        // Flow orients 2D faces counter-clockwise: the exterior ring must wind
        // CCW, each hole clockwise.
        ValidationReport::ran(|r| {
            check_ring_orientation_2d(&self.frame, self.exterior(), true, r);
            for hole in self.interiors() {
                check_ring_orientation_2d(&self.frame, hole, false, r);
            }
        })
    }
}

impl Validate for Polygon3D {
    fn applicable_checks(&self) -> &'static [ValidationType] {
        &POLYGON_3D_CHECKS
    }

    fn check_finite(&self, _params: &ValidationParams) -> ValidationReport {
        ValidationReport::ran(|r| check_finite_3d(&self.frame, self.coords.iter().copied(), r))
    }

    fn check_too_few_points(&self, _params: &ValidationParams) -> ValidationReport {
        ValidationReport::ran(|r| {
            check_too_few_points_3d(&self.frame, self.exterior(), true, r);
            for hole in self.interiors() {
                check_too_few_points_3d(&self.frame, hole, true, r);
            }
        })
    }

    fn check_unclosed_ring(&self, _params: &ValidationParams) -> ValidationReport {
        ValidationReport::ran(|r| {
            check_unclosed_ring_3d(&self.frame, self.exterior(), r);
            for hole in self.interiors() {
                check_unclosed_ring_3d(&self.frame, hole, r);
            }
        })
    }

    fn check_duplicate_points(&self, params: &ValidationParams) -> ValidationReport {
        ValidationReport::ran(|r| {
            for ring in std::iter::once(self.exterior()).chain(self.interiors()) {
                check_duplicate_points(
                    &self.frame,
                    open_ring(ring).iter().copied(),
                    params.duplicate_tolerance,
                    r,
                );
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::CoordinateFrame;
    use crate::polygon::Polygon2D;
    use crate::validation_next::{validate_one, ValidationParams, ValidationResult};

    fn square() -> [[f64; 2]; 5] {
        [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0], [0.0, 0.0]]
    }

    /// The failing positions of `check` on `p`, or a panic if it did not fail.
    /// Runs just that check (and its prerequisites), not the leaf's other,
    /// still-unimplemented checks.
    fn failures(p: &Polygon2D, check: ValidationType) -> Vec<crate::Geometry> {
        match validate_one(p, check, &ValidationParams::default()) {
            ValidationResult::Failed(positions) => positions,
            other => panic!("expected {check} to fail, got {other:?}"),
        }
    }

    #[test]
    fn closed_ring_closure_is_not_a_duplicate() {
        let p = Polygon2D::from_rings(
            CoordinateFrame::Euclidean,
            square(),
            Vec::<Vec<[f64; 2]>>::new(),
        );
        assert_eq!(
            validate_one(
                &p,
                ValidationType::DuplicatePoints,
                &ValidationParams::default()
            ),
            ValidationResult::Success
        );
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
        assert_eq!(failures(&p, ValidationType::DuplicatePoints).len(), 1);
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
        assert_eq!(failures(&p, ValidationType::TooFewPoints).len(), 1);
    }

    #[test]
    fn too_few_points_checks_each_ring() {
        // Valid exterior, degenerate hole → one problem for the hole.
        let hole = vec![[1.0, 1.0], [2.0, 1.0], [1.0, 1.0]];
        let p = Polygon2D::from_rings(CoordinateFrame::Euclidean, square(), vec![hole]);
        assert_eq!(failures(&p, ValidationType::TooFewPoints).len(), 1);
    }

    #[test]
    fn closed_ring_passes_unclosed_check() {
        let p = Polygon2D::from_rings(
            CoordinateFrame::Euclidean,
            square(),
            Vec::<Vec<[f64; 2]>>::new(),
        );
        assert_eq!(
            validate_one(
                &p,
                ValidationType::UnclosedRing,
                &ValidationParams::default()
            ),
            ValidationResult::Success
        );
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
        let positions = failures(&p, ValidationType::UnclosedRing);
        assert_eq!(positions.len(), 1);
        assert!(matches!(
            positions[0],
            crate::Geometry::Euclidean2D(crate::Euclidean2DGeometry::LineString(_))
        ));
    }

    #[test]
    fn unclosed_check_covers_holes() {
        // Closed exterior, open hole → one problem for the hole.
        let open_hole = vec![[1.0, 1.0], [2.0, 1.0], [2.0, 2.0]];
        let p = Polygon2D::from_rings(CoordinateFrame::Euclidean, square(), vec![open_hole]);
        assert_eq!(failures(&p, ValidationType::UnclosedRing).len(), 1);
    }

    #[test]
    fn ccw_exterior_is_oriented() {
        // `square()` winds counter-clockwise, Flow's convention.
        let p = Polygon2D::from_rings(
            CoordinateFrame::Euclidean,
            square(),
            Vec::<Vec<[f64; 2]>>::new(),
        );
        assert_eq!(
            validate_one(
                &p,
                ValidationType::Orientation,
                &ValidationParams::default()
            ),
            ValidationResult::Success
        );
    }

    #[test]
    fn cw_exterior_is_misoriented() {
        // The square reversed winds clockwise → one problem for the exterior ring.
        let cw = [[0.0, 0.0], [0.0, 4.0], [4.0, 4.0], [4.0, 0.0], [0.0, 0.0]];
        let p = Polygon2D::from_rings(CoordinateFrame::Euclidean, cw, Vec::<Vec<[f64; 2]>>::new());
        assert_eq!(failures(&p, ValidationType::Orientation).len(), 1);
    }

    #[test]
    fn cw_hole_is_oriented() {
        // CCW exterior with a clockwise hole is correctly oriented.
        let cw_hole = vec![[1.0, 1.0], [1.0, 2.0], [2.0, 2.0], [2.0, 1.0], [1.0, 1.0]];
        let p = Polygon2D::from_rings(CoordinateFrame::Euclidean, square(), vec![cw_hole]);
        assert_eq!(
            validate_one(
                &p,
                ValidationType::Orientation,
                &ValidationParams::default()
            ),
            ValidationResult::Success
        );
    }

    #[test]
    fn ccw_hole_is_misoriented() {
        // A hole winding counter-clockwise (same as the exterior) is wrong.
        let ccw_hole = vec![[1.0, 1.0], [2.0, 1.0], [2.0, 2.0], [1.0, 2.0], [1.0, 1.0]];
        let p = Polygon2D::from_rings(CoordinateFrame::Euclidean, square(), vec![ccw_hole]);
        assert_eq!(failures(&p, ValidationType::Orientation).len(), 1);
    }
}
