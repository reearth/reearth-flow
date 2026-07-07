//! Validations/predicates on geometry types.
//!
//! # Default validation per leaf type
//!
//! Which checks `GeometryValidator` runs for each leaf type, one column per
//! [`ValidationType`]. `✓` runs, `·` does not (not applicable). Finiteness (the
//! `Finite` column) always runs.
//!
//! | Leaf ╲ Check      | Finite | TooFewPoints | UnclosedRing | SelfIntersection | InteriorRingContainment | Degenerate | DuplicatePoints | Orientation | NormalDirection |
//! |-------------------|:------:|:------------:|:------------:|:----------------:|:-----------------------:|:----------:|:---------------:|:-----------:|:---------------:|
//! | Point2D / 3D      |   ✓    |      ·       |      ·       |        ·         |            ·            |     ·      |        ·        |      ·      |        ·        |
//! | LineString2D / 3D |   ✓    |      ✓       |      ·       |        ✓         |            ·            |     ✓      |        ✓        |      ·      |        ·        |
//! | Polygon2D         |   ✓    |      ✓       |      ✓       |        ✓         |            ✓            |     ✓      |        ✓        |      ✓      |        ·        |
//! | Polygon3D         |   ✓    |      ✓       |      ✓       |        ✓         |            ✓            |     ✓      |        ✓        |      ·      |        ·        |
//! | PolygonMesh2D     |   ✓    |      ✓       |      ✓       |        ✓         |            ✓            |     ✓      |        ✓        |      ✓      |        ·        |
//! | PolygonMesh3D     |   ✓    |      ✓       |      ✓       |        ✓         |            ✓            |     ✓      |        ✓        |      ✓      |        ✓        |
//! | TriangularMesh2D  |   ✓    |      ·       |      ·       |        ·         |            ·            |     ✓      |        ✓        |      ✓      |        ·        |
//! | TriangularMesh3D  |   ✓    |      ·       |      ·       |        ·         |            ·            |     ✓      |        ✓        |      ✓      |        ✓        |
//! | Solid             |   ✓    |      ✓       |      ✓       |        ✓         |            ·            |     ✓      |        ✓        |      ✓      |        ✓        |
//! | Csg               |   ✓    |      ✓       |      ✓       |        ✓         |            ✓            |     ✓      |        ✓        |      ✓      |        ✓        |
//! | PointCloud        |   ✓    |      ·       |      ·       |        ·         |            ·            |     ·      |        ✓        |      ·      |        ·        |
//! | Collection2D / 3D |   ✓    |      ✓       |      ✓       |        ✓         |            ✓            |     ✓      |        ✓        |      ✓      |        ✓        |
//!
//! # Check dependencies
//!
//! A check is only meaningful once the checks it depends on hold, so each
//! [`ValidationType`] lists its immediate prerequisites via
//! [`ValidationType::dependencies`] (transitive-close for the full set). A
//! runner should skip a check while any prerequisite fails. The relation is a
//! DAG (checked in the unit tests):
//!
//! | Check                        | Immediate prerequisites                  |
//! |------------------------------|------------------------------------------|
//! | `Finite` (always on)         | — |
//! | `TooFewPoints`               | — |
//! | `Connected`                  | — |
//! | `UnclosedRing`               | `Finite` |
//! | `DuplicatePoints`            | `Finite` |
//! | `Planarity`                  | `Finite` |
//! | `Degenerate`                 | `Finite` |
//! | `SelfIntersection`           | `Finite`, `TooFewPoints`, `UnclosedRing` |
//! | `InteriorRingContainment`    | `Finite`, `SelfIntersection` |
//! | `ShellManifold`              | `Connected` |
//! | `Orientation`                | `Finite`, `Connected` |
//! | `NormalDirection`            | `Orientation`, `ShellManifold` |

use std::collections::HashSet;
use std::fmt;

use kiddo::{KdTree, SquaredEuclidean};
use serde::Serialize;

use crate::coordinate::CoordinateFrame;
use crate::line_string::{LineString2D, LineString3D};
use crate::point::{Point2D, Point3D};
use crate::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};

/// Which validity check to run.
///
/// One variant per column of the
/// [validation matrix](self#default-validation-per-leaf-type). `Finite`,
/// `TooFewPoints`, `UnclosedRing`, and `DuplicatePoints` are implemented; every
/// other variant's detection is a `TODO`.
#[derive(Clone, Debug, PartialEq)]
pub enum ValidationType {
    /// Every coordinate component is finite (non-NaN, non-infinite). Always run,
    /// regardless of which check is selected.
    Finite,
    /// A line or ring has fewer points than its type requires (line ≥ 2, closed
    /// ring ≥ 4).
    TooFewPoints,
    /// Coordinates that coincide anywhere within a geometry.
    DuplicatePoints {
        /// Max distance under which two coordinates are treated as coincident;
        /// `None` = exact equality.
        tolerance: Option<f64>,
    },
    /// A ring is not closed (first vertex != last).
    UnclosedRing,
    /// A ring or boundary crosses itself.
    SelfIntersection {
        /// Overlaps shorter than this are ignored; `None` = exact crossing test.
        tolerance: Option<f64>,
    },
    /// An interior ring (hole) is not contained in its exterior ring.
    InteriorRingContainment {
        /// Slack for shared-vertex touches between the rings; `None` = exact.
        tolerance: Option<f64>,
    },
    /// A 3D face's vertices are not coplanar. Not meaningful for a 2D or 2.5D
    /// face (planar by construction) or a triangle.
    Planarity {
        /// Max out-of-plane distance a vertex may have from the face's best-fit
        /// plane, in coordinate units.
        max_deviation: f64,
    },
    /// The geometry has zero or near-zero extent (length, area, or volume).
    Degenerate {
        /// Length / area / volume floor below which the geometry is degenerate,
        /// in coordinate units.
        min_extent: f64,
    },
    /// A mesh or solid is not a single connected component (by shared vertices /
    /// edges).
    Connected,
    /// The surface is not *consistently* oriented (adjacent face normals
    /// disagree). Type-dependent: a 2D face means ring winding (exterior CCW,
    /// holes CW); a 3D mesh or solid means coherent winding across shared edges.
    /// Relative consistency only; see
    /// [`NormalDirection`](ValidationType::NormalDirection) for absolute
    /// direction.
    Orientation,
    /// The absolute normal direction is wrong. Defined only for a closed,
    /// consistently-oriented surface: outward for a closed manifold mesh or a
    /// solid's exterior shell, inward for a solid's void shells. Depends on
    /// [`Orientation`](ValidationType::Orientation) and
    /// [`ShellManifold`](ValidationType::ShellManifold).
    NormalDirection,
    /// A solid's boundary is not a closed 2-manifold (watertight).
    ShellManifold,
}

impl ValidationType {
    /// Immediate prerequisite checks; a runner should skip this check while any
    /// of them fails. Transitive-close for the full set. Tabulated under
    /// [check dependencies](self#check-dependencies). Parameters on the returned
    /// checks are placeholders and carry no meaning.
    pub fn dependencies(&self) -> &'static [ValidationType] {
        use ValidationType::*;
        match self {
            Finite | TooFewPoints | Connected => &[],
            UnclosedRing | DuplicatePoints { .. } | Planarity { .. } | Degenerate { .. } => {
                &[Finite]
            }
            SelfIntersection { .. } => &[Finite, TooFewPoints, UnclosedRing],
            InteriorRingContainment { .. } => &[Finite, SelfIntersection { tolerance: None }],
            ShellManifold => &[Connected],
            Orientation => &[Finite, Connected],
            NormalDirection => &[Orientation, ShellManifold],
        }
    }

    /// Whether this is an optional, advisory quality check rather than a core
    /// validity check. An optional check's failure is a warning about geometry
    /// quality, not proof that the geometry is invalid.
    pub fn is_optional(&self) -> bool {
        matches!(
            self,
            ValidationType::DuplicatePoints { .. }
                | ValidationType::Planarity { .. }
                | ValidationType::Connected
                | ValidationType::Orientation
                | ValidationType::NormalDirection
        )
    }
}

impl fmt::Display for ValidationType {
    /// The check's variant name and its parameters, used as the `problem` label
    /// on a reported [`ValidationProblemAtPosition`].
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationType::Finite => f.write_str("Finite"),
            ValidationType::TooFewPoints => f.write_str("TooFewPoints"),
            ValidationType::DuplicatePoints { tolerance } => {
                write!(f, "DuplicatePoints(tolerance: {tolerance:?})")
            }
            ValidationType::UnclosedRing => f.write_str("UnclosedRing"),
            ValidationType::SelfIntersection { tolerance } => {
                write!(f, "SelfIntersection(tolerance: {tolerance:?})")
            }
            ValidationType::InteriorRingContainment { tolerance } => {
                write!(f, "InteriorRingContainment(tolerance: {tolerance:?})")
            }
            ValidationType::Planarity { max_deviation } => {
                write!(f, "Planarity(max_deviation: {max_deviation})")
            }
            ValidationType::Degenerate { min_extent } => {
                write!(f, "Degenerate(min_extent: {min_extent})")
            }
            ValidationType::Connected => f.write_str("Connected"),
            ValidationType::Orientation => f.write_str("Orientation"),
            ValidationType::NormalDirection => f.write_str("NormalDirection"),
            ValidationType::ShellManifold => f.write_str("ShellManifold"),
        }
    }
}

/// A validity problem together with where it occurred.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct ValidationProblemAtPosition {
    /// The problem encountered.
    pub problem: String,
    /// The geometry pinpointing where the problem was found — typically a point
    /// leaf at the offending coordinate.
    pub position: Geometry,
}

/// Every problem found while validating a geometry.
///
/// A `None` return from [`Validate::validate`] means "no problems"; a
/// `Some(report)` is always non-empty (see [`ValidationReport::into_option`]).
#[derive(Serialize, Clone, Debug, PartialEq, Default)]
pub struct ValidationReport(pub Vec<ValidationProblemAtPosition>);

impl ValidationReport {
    /// The number of problems recorded.
    pub fn error_count(&self) -> usize {
        self.0.len()
    }

    /// Whether no problems were recorded.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Record a problem at a position.
    pub fn push(&mut self, problem: String, position: Geometry) {
        self.0
            .push(ValidationProblemAtPosition { problem, position });
    }

    /// Absorb another report's problems.
    pub fn extend(&mut self, other: ValidationReport) {
        self.0.extend(other.0);
    }

    /// Wrap in `Some` when non-empty, else `None` — the [`Validate::validate`]
    /// return convention.
    pub fn into_option(self) -> Option<Self> {
        if self.0.is_empty() {
            None
        } else {
            Some(self)
        }
    }
}

/// Validate a geometry against a chosen [`ValidationType`].
///
/// Returns the problems found, or `None` when valid. The default body reports
/// no problems (always valid); each leaf provides its own impl.
#[enum_dispatch::enum_dispatch]
pub trait Validate {
    /// Run `valid_type`'s check (plus the always-on finiteness check) over the
    /// geometry.
    fn validate(&self, valid_type: ValidationType) -> Option<ValidationReport> {
        let _ = valid_type;
        None
    }
}

// The boxed enum variants (`Box<Polygon2D>`, `Box<Solid>`, …) need the trait on
// the `Box` itself: `enum_dispatch` forwards by UFCS, not auto-deref.
impl<T: Validate + ?Sized> Validate for Box<T> {
    fn validate(&self, valid_type: ValidationType) -> Option<ValidationReport> {
        (**self).validate(valid_type)
    }
}

/// Scan a 2D coordinate buffer (with an optional parallel elevation buffer) for
/// non-finite values, pushing one [`ValidationType::Finite`] problem per
/// offending coordinate into `report`, positioned at a 2D point leaf in `frame`.
pub(crate) fn check_finite_2d(
    frame: &CoordinateFrame,
    coords: &[[f64; 2]],
    z: Option<&[f64]>,
    report: &mut ValidationReport,
) {
    for (i, c) in coords.iter().enumerate() {
        let z_not_finite = z.and_then(|zs| zs.get(i)).is_some_and(|v| !v.is_finite());
        if !c[0].is_finite() || !c[1].is_finite() || z_not_finite {
            report.push(
                ValidationType::Finite.to_string(),
                Geometry::Euclidean2D(Euclidean2DGeometry::Point(Point2D::new(frame.clone(), *c))),
            );
        }
    }
}

/// Scan 3D coordinates for non-finite values, pushing one
/// [`ValidationType::Finite`] problem per offending coordinate into `report`,
/// positioned at a 3D point leaf in `frame`. Takes an iterator so streamed
/// sources (e.g. a point cloud's decoded positions) need not be materialized.
pub(crate) fn check_finite_3d(
    frame: &CoordinateFrame,
    coords: impl IntoIterator<Item = [f64; 3]>,
    report: &mut ValidationReport,
) {
    for c in coords {
        if !c[0].is_finite() || !c[1].is_finite() || !c[2].is_finite() {
            report.push(
                ValidationType::Finite.to_string(),
                Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(frame.clone(), c))),
            );
        }
    }
}

/// Report a [`ValidationType::TooFewPoints`] problem when a line (`is_ring =
/// false`, needs ≥ 2) or closed ring (`is_ring = true`, needs ≥ 4) has too few
/// coordinates, positioned at the offending line / ring as a 2D `LineString`.
pub(crate) fn check_too_few_points_2d(
    frame: &CoordinateFrame,
    coords: &[[f64; 2]],
    is_ring: bool,
    report: &mut ValidationReport,
) {
    if coords.len() < if is_ring { 4 } else { 2 } {
        report.push(
            ValidationType::TooFewPoints.to_string(),
            Geometry::Euclidean2D(Euclidean2DGeometry::LineString(LineString2D::from_coords(
                frame.clone(),
                coords.iter().copied(),
            ))),
        );
    }
}

/// Report a [`ValidationType::TooFewPoints`] problem when a line (`is_ring =
/// false`, needs ≥ 2) or closed ring (`is_ring = true`, needs ≥ 4) has too few
/// coordinates, positioned at the offending line / ring as a 3D `LineString`.
pub(crate) fn check_too_few_points_3d(
    frame: &CoordinateFrame,
    coords: &[[f64; 3]],
    is_ring: bool,
    report: &mut ValidationReport,
) {
    if coords.len() < if is_ring { 4 } else { 2 } {
        report.push(
            ValidationType::TooFewPoints.to_string(),
            Geometry::Euclidean3D(Euclidean3DGeometry::LineString(LineString3D::from_coords(
                frame.clone(),
                coords.iter().copied(),
            ))),
        );
    }
}

/// Report a [`ValidationType::UnclosedRing`] problem when a ring's first and last
/// coordinates differ, positioned at the offending ring as a 2D `LineString`. An
/// empty ring has nothing to close.
pub(crate) fn check_unclosed_ring_2d(
    frame: &CoordinateFrame,
    ring: &[[f64; 2]],
    report: &mut ValidationReport,
) {
    if ring.first().is_some_and(|first| Some(first) != ring.last()) {
        report.push(
            ValidationType::UnclosedRing.to_string(),
            Geometry::Euclidean2D(Euclidean2DGeometry::LineString(LineString2D::from_coords(
                frame.clone(),
                ring.iter().copied(),
            ))),
        );
    }
}

/// Report a [`ValidationType::UnclosedRing`] problem when a ring's first and last
/// coordinates differ, positioned at the offending ring as a 3D `LineString`. An
/// empty ring has nothing to close.
pub(crate) fn check_unclosed_ring_3d(
    frame: &CoordinateFrame,
    ring: &[[f64; 3]],
    report: &mut ValidationReport,
) {
    if ring.first().is_some_and(|first| Some(first) != ring.last()) {
        report.push(
            ValidationType::UnclosedRing.to_string(),
            Geometry::Euclidean3D(Euclidean3DGeometry::LineString(LineString3D::from_coords(
                frame.clone(),
                ring.iter().copied(),
            ))),
        );
    }
}

/// The bit pattern of a coordinate component, normalizing `-0.0` to `+0.0` so the
/// two hash and compare equal in the exact duplicate scan.
#[inline]
fn norm_bits(x: f64) -> u64 {
    (x + 0.0).to_bits()
}

/// Report a [`ValidationType::DuplicatePoints`] problem per coordinate that
/// coincides with an earlier one, positioned at the offending coordinate as a 2D
/// point. Exact bit-equality when `tolerance` is `None`; otherwise two coords are
/// coincident when within `tolerance` distance. Non-finite coords are skipped
/// (already covered by the finiteness check).
pub(crate) fn check_duplicate_points_2d(
    frame: &CoordinateFrame,
    coords: impl IntoIterator<Item = [f64; 2]>,
    tolerance: Option<f64>,
    report: &mut ValidationReport,
) {
    let label = ValidationType::DuplicatePoints { tolerance }.to_string();
    let mut push = |c: [f64; 2]| {
        report.push(
            label.clone(),
            Geometry::Euclidean2D(Euclidean2DGeometry::Point(Point2D::new(frame.clone(), c))),
        );
    };
    match tolerance {
        None => {
            let mut seen = HashSet::new();
            for c in coords {
                if !c[0].is_finite() || !c[1].is_finite() {
                    continue;
                }
                if !seen.insert([norm_bits(c[0]), norm_bits(c[1])]) {
                    push(c);
                }
            }
        }
        Some(t) => {
            let radius = t * t;
            let mut tree: KdTree<f64, 2> = KdTree::new();
            let mut n: u64 = 0;
            for c in coords {
                if !c[0].is_finite() || !c[1].is_finite() {
                    continue;
                }
                if n > 0 && tree.nearest_one::<SquaredEuclidean>(&c).distance <= radius {
                    push(c);
                } else {
                    tree.add(&c, n);
                    n += 1;
                }
            }
        }
    }
}

/// Report a [`ValidationType::DuplicatePoints`] problem per coordinate that
/// coincides with an earlier one, positioned at the offending coordinate as a 3D
/// point. Matching semantics mirror [`check_duplicate_points_2d`].
pub(crate) fn check_duplicate_points_3d(
    frame: &CoordinateFrame,
    coords: impl IntoIterator<Item = [f64; 3]>,
    tolerance: Option<f64>,
    report: &mut ValidationReport,
) {
    let label = ValidationType::DuplicatePoints { tolerance }.to_string();
    let mut push = |c: [f64; 3]| {
        report.push(
            label.clone(),
            Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(frame.clone(), c))),
        );
    };
    match tolerance {
        None => {
            let mut seen = HashSet::new();
            for c in coords {
                if !c[0].is_finite() || !c[1].is_finite() || !c[2].is_finite() {
                    continue;
                }
                if !seen.insert([norm_bits(c[0]), norm_bits(c[1]), norm_bits(c[2])]) {
                    push(c);
                }
            }
        }
        Some(t) => {
            let radius = t * t;
            let mut tree: KdTree<f64, 3> = KdTree::new();
            let mut n: u64 = 0;
            for c in coords {
                if !c[0].is_finite() || !c[1].is_finite() || !c[2].is_finite() {
                    continue;
                }
                if n > 0 && tree.nearest_one::<SquaredEuclidean>(&c).distance <= radius {
                    push(c);
                } else {
                    tree.add(&c, n);
                    n += 1;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::CoordinateFrame;
    use crate::line_string::LineString3D;
    use crate::point::Point3D;
    use crate::{Euclidean3DGeometry, Geometry};

    #[test]
    fn finite_point_is_valid() {
        let p = Point3D::new(CoordinateFrame::Euclidean, [1.0, 2.0, 3.0]);
        assert!(p
            .validate(ValidationType::DuplicatePoints { tolerance: None })
            .is_none());
    }

    #[test]
    fn non_finite_point_reports_not_finite() {
        let p = Point3D::new(CoordinateFrame::Euclidean, [1.0, f64::NAN, 3.0]);
        let report = p
            .validate(ValidationType::DuplicatePoints { tolerance: None })
            .unwrap();
        assert_eq!(report.error_count(), 1);
        assert_eq!(report.0[0].problem, "Finite");
        assert!(matches!(
            report.0[0].position,
            Geometry::Euclidean3D(Euclidean3DGeometry::Point(_))
        ));
    }

    #[test]
    fn linestring_reports_each_non_finite_coordinate() {
        let ls = LineString3D::from_coords(
            CoordinateFrame::Euclidean,
            [
                [0.0, 0.0, 0.0],
                [f64::INFINITY, 1.0, 0.0],
                [2.0, f64::NAN, 0.0],
            ],
        );
        let report = ls.validate(ValidationType::TooFewPoints).unwrap();
        assert_eq!(report.error_count(), 2);
        // Each problem is positioned at a 3D point leaf holding the offending
        // coordinate.
        let first = offending_point(&report.0[0].position);
        assert!(first[0].is_infinite());
        assert_eq!(first[1], 1.0);
        let second = offending_point(&report.0[1].position);
        assert!(second[1].is_nan());
        assert_eq!(second[0], 2.0);
    }

    /// The `[x, y, z]` of the 3D point leaf a problem is positioned at.
    fn offending_point(position: &Geometry) -> [f64; 3] {
        match position {
            Geometry::Euclidean3D(Euclidean3DGeometry::Point(p)) => p.position(),
            other => panic!("expected a 3D point position, got {other:?}"),
        }
    }

    #[test]
    fn too_few_points_flags_single_point_line() {
        let ls = LineString3D::from_coords(CoordinateFrame::Euclidean, [[0.0, 0.0, 0.0]]);
        let report = ls.validate(ValidationType::TooFewPoints).unwrap();
        assert_eq!(report.error_count(), 1);
        assert_eq!(report.0[0].problem, "TooFewPoints");
        // The position is the offending line, not a single coordinate.
        assert!(matches!(
            report.0[0].position,
            Geometry::Euclidean3D(Euclidean3DGeometry::LineString(_))
        ));
    }

    #[test]
    fn two_point_line_has_enough_points() {
        let ls = LineString3D::from_coords(
            CoordinateFrame::Euclidean,
            [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
        );
        assert!(ls.validate(ValidationType::TooFewPoints).is_none());
    }

    #[test]
    fn duplicate_points_exact_flags_repeated_coordinate() {
        let ls = LineString3D::from_coords(
            CoordinateFrame::Euclidean,
            [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 0.0]],
        );
        let report = ls
            .validate(ValidationType::DuplicatePoints { tolerance: None })
            .unwrap();
        assert_eq!(report.error_count(), 1);
        assert_eq!(report.0[0].problem, "DuplicatePoints(tolerance: None)");
        assert_eq!(offending_point(&report.0[0].position), [0.0, 0.0, 0.0]);
    }

    #[test]
    fn duplicate_points_tolerance_uses_distance() {
        let ls = LineString3D::from_coords(
            CoordinateFrame::Euclidean,
            [[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [0.001, 0.0, 0.0]],
        );
        // The third point is within 0.01 of the first.
        let report = ls
            .validate(ValidationType::DuplicatePoints {
                tolerance: Some(0.01),
            })
            .unwrap();
        assert_eq!(report.error_count(), 1);
        assert_eq!(offending_point(&report.0[0].position), [0.001, 0.0, 0.0]);
        // Under a tighter tolerance nothing coincides.
        assert!(ls
            .validate(ValidationType::DuplicatePoints {
                tolerance: Some(1e-6)
            })
            .is_none());
    }

    #[test]
    fn dispatch_reaches_leaf_through_geometry() {
        let g = Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(
            CoordinateFrame::Euclidean,
            [f64::NAN, 0.0, 0.0],
        )));
        assert_eq!(
            g.validate(ValidationType::DuplicatePoints { tolerance: None })
                .unwrap()
                .error_count(),
            1
        );
    }

    #[test]
    fn none_geometry_is_valid() {
        assert!(Geometry::None
            .validate(ValidationType::DuplicatePoints { tolerance: None })
            .is_none());
    }

    #[test]
    fn optional_checks_are_flagged_optional() {
        // The optional, advisory checks.
        for optional in [
            ValidationType::DuplicatePoints { tolerance: None },
            ValidationType::Planarity { max_deviation: 0.1 },
            ValidationType::Connected,
            ValidationType::Orientation,
            ValidationType::NormalDirection,
        ] {
            assert!(optional.is_optional(), "{optional:?} should be optional");
        }
    }

    #[test]
    fn core_checks_are_not_optional() {
        // The core validity checks, including the degenerate check.
        for core in [
            ValidationType::Finite,
            ValidationType::TooFewPoints,
            ValidationType::UnclosedRing,
            ValidationType::SelfIntersection { tolerance: None },
            ValidationType::InteriorRingContainment { tolerance: None },
            ValidationType::Degenerate { min_extent: 0.0 },
            ValidationType::ShellManifold,
        ] {
            assert!(!core.is_optional(), "{core:?} should be core");
        }
    }

    /// Every `ValidationType` variant, so the dependency graph can be walked in
    /// full.
    const ALL_TYPES: [ValidationType; 12] = [
        ValidationType::Finite,
        ValidationType::TooFewPoints,
        ValidationType::DuplicatePoints { tolerance: None },
        ValidationType::UnclosedRing,
        ValidationType::SelfIntersection { tolerance: None },
        ValidationType::InteriorRingContainment { tolerance: None },
        ValidationType::Planarity { max_deviation: 0.0 },
        ValidationType::Degenerate { min_extent: 0.0 },
        ValidationType::Connected,
        ValidationType::Orientation,
        ValidationType::NormalDirection,
        ValidationType::ShellManifold,
    ];

    #[test]
    fn documented_dependencies_hold() {
        assert_eq!(
            ValidationType::NormalDirection.dependencies(),
            &[ValidationType::Orientation, ValidationType::ShellManifold]
        );
        assert_eq!(
            ValidationType::Orientation.dependencies(),
            &[ValidationType::Finite, ValidationType::Connected]
        );
        assert!(ValidationType::Finite.dependencies().is_empty());
    }

    #[test]
    fn dependency_graph_is_acyclic() {
        // Depth-first cycle detection over the full dependency relation.
        fn reaches(from: &ValidationType, target: &ValidationType) -> bool {
            from.dependencies()
                .iter()
                .any(|dep| dep == target || reaches(dep, target))
        }
        for kind in &ALL_TYPES {
            assert!(!reaches(kind, kind), "{kind:?} depends on itself");
        }
    }
}
