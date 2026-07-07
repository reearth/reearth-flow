//! Validations/predicates on geometry types.
//!
//! # Default validation per leaf type
//!
//! Which checks `GeometryValidator` runs for each leaf type, one column per
//! [`ValidationType`]. `✓` runs, `·` does not (not applicable). Finiteness (the
//! `Finite` column) always runs.
//!
//! | Leaf ╲ Check      | Finite | TooFewPoints | UnclosedRing | SelfIntersection | InteriorRingContainment | Degenerate | DuplicatePoints | DuplicateConsecutivePoints | Orientation | NormalDirection |
//! |-------------------|:------:|:------------:|:------------:|:----------------:|:-----------------------:|:----------:|:---------------:|:--------------------------:|:-----------:|:---------------:|
//! | Point2D / 3D      |   ✓    |      ·       |      ·       |        ·         |            ·            |     ·      |        ·        |             ·              |      ·      |        ·        |
//! | LineString2D / 3D |   ✓    |      ✓       |      ·       |        ✓         |            ·            |     ✓      |        ✓        |             ✓              |      ·      |        ·        |
//! | Polygon2D         |   ✓    |      ✓       |      ✓       |        ✓         |            ✓            |     ✓      |        ✓        |             ✓              |      ✓      |        ·        |
//! | Polygon3D         |   ✓    |      ✓       |      ✓       |        ✓         |            ✓            |     ✓      |        ✓        |             ✓              |      ·      |        ·        |
//! | PolygonMesh2D     |   ✓    |      ✓       |      ✓       |        ✓         |            ✓            |     ✓      |        ✓        |             ✓              |      ✓      |        ·        |
//! | PolygonMesh3D     |   ✓    |      ✓       |      ✓       |        ✓         |            ✓            |     ✓      |        ✓        |             ✓              |      ✓      |        ✓        |
//! | TriangularMesh2D  |   ✓    |      ·       |      ·       |        ·         |            ·            |     ✓      |        ✓        |             ·              |      ✓      |        ·        |
//! | TriangularMesh3D  |   ✓    |      ·       |      ·       |        ·         |            ·            |     ✓      |        ✓        |             ·              |      ✓      |        ✓        |
//! | Solid             |   ✓    |      ✓       |      ✓       |        ✓         |            ·            |     ✓      |        ✓        |             ·              |      ✓      |        ✓        |
//! | Csg               |   ✓    |      ✓       |      ✓       |        ✓         |            ✓            |     ✓      |        ✓        |             ✓              |      ✓      |        ✓        |
//! | PointCloud        |   ✓    |      ·       |      ·       |        ·         |            ·            |     ·      |        ✓        |             ·              |      ·      |        ·        |
//! | Collection2D / 3D |   ✓    |      ✓       |      ✓       |        ✓         |            ✓            |     ✓      |        ✓        |             ✓              |      ✓      |        ✓        |
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
//! | `DuplicateConsecutivePoints` | `Finite` |
//! | `Planarity`                  | `Finite` |
//! | `Degenerate`                 | `Finite` |
//! | `SelfIntersection`           | `Finite`, `TooFewPoints`, `UnclosedRing` |
//! | `InteriorRingContainment`    | `Finite`, `SelfIntersection` |
//! | `ShellManifold`              | `Connected` |
//! | `Orientation`                | `Finite`, `Connected` |
//! | `NormalDirection`            | `Orientation`, `ShellManifold` |

use serde::Serialize;

/// Which validity check to run.
///
/// One variant per column of the
/// [validation matrix](self#default-validation-per-leaf-type). Only finiteness
/// is implemented; every other variant's detection is a `TODO`.
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
    /// Adjacent coordinates that fall within a distance threshold.
    DuplicateConsecutivePoints {
        /// Max distance between neighbours before they are flagged, in
        /// coordinate units.
        threshold: f64,
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
            UnclosedRing
            | DuplicatePoints { .. }
            | DuplicateConsecutivePoints { .. }
            | Planarity { .. }
            | Degenerate { .. } => &[Finite],
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
                | ValidationType::DuplicateConsecutivePoints { .. }
                | ValidationType::Planarity { .. }
                | ValidationType::Connected
                | ValidationType::Orientation
                | ValidationType::NormalDirection
        )
    }
}

/// A single kind of validity problem.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub enum ValidationProblem {
    /// A coordinate component is NaN or infinite.
    NotFinite,
    /// A line or ring has fewer points than its type requires. (`TODO`)
    TooFewPoints,
    /// A polygon ring is not closed (first vertex != last). (`TODO`)
    UnclosedRing,
    /// Two coordinates of a geometry coincide. (`TODO`)
    IdenticalCoords,
    /// Consecutive coordinates fall within the duplicate-distance threshold.
    /// (`TODO`)
    DuplicateConsecutivePoints,
    /// A ring or boundary intersects itself. (`TODO`)
    SelfIntersection,
    /// An interior ring is not contained in its exterior ring. (`TODO`)
    InteriorRingNotContainedInExteriorRing,
    /// A 3D face's vertices are not coplanar. (`TODO`)
    NonPlanar,
    /// The geometry has zero extent or is otherwise degenerate. (`TODO`)
    DegenerateGeometry,
    /// A mesh or solid has more than one connected component. (`TODO`)
    Disconnected,
    /// A surface is not consistently oriented (adjacent normals disagree).
    /// (`TODO`)
    WrongOrientation,
    /// A closed surface's normals point the wrong way (a solid exterior facing
    /// inward, or a void facing outward). (`TODO`)
    WrongNormalDirection,
    /// A solid's boundary is not a closed 2-manifold. (`TODO`)
    NonManifold,
}

/// Where in a geometry a problem occurs: the concrete leaf type plus, when a
/// single coordinate is implicated, its index in that leaf's coordinate buffer.
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct ValidationProblemPosition {
    /// Concrete geometry type the problem was found in.
    pub geometry: &'static str,
    /// Index of the offending coordinate within the leaf's coordinate buffer,
    /// when a single coordinate is implicated; `None` otherwise.
    pub coordinate_index: Option<usize>,
}

/// A [`ValidationProblem`] together with where it occurred.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct ValidationProblemAtPosition {
    /// The problem encountered.
    pub problem: ValidationProblem,
    /// Where in the geometry it was found.
    pub position: ValidationProblemPosition,
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
    pub fn push(&mut self, problem: ValidationProblem, position: ValidationProblemPosition) {
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
/// non-finite values, pushing one [`ValidationProblem::NotFinite`] per
/// offending coordinate into `report`.
pub(crate) fn check_finite_2d(
    geometry: &'static str,
    coords: &[[f64; 2]],
    z: Option<&[f64]>,
    report: &mut ValidationReport,
) {
    for (i, c) in coords.iter().enumerate() {
        let z_not_finite = z.and_then(|zs| zs.get(i)).is_some_and(|v| !v.is_finite());
        if !c[0].is_finite() || !c[1].is_finite() || z_not_finite {
            report.push(
                ValidationProblem::NotFinite,
                ValidationProblemPosition {
                    geometry,
                    coordinate_index: Some(i),
                },
            );
        }
    }
}

/// Scan a 3D coordinate buffer for non-finite values, pushing one
/// [`ValidationProblem::NotFinite`] per offending coordinate into `report`.
pub(crate) fn check_finite_3d(
    geometry: &'static str,
    coords: &[[f64; 3]],
    report: &mut ValidationReport,
) {
    for (i, c) in coords.iter().enumerate() {
        if !c[0].is_finite() || !c[1].is_finite() || !c[2].is_finite() {
            report.push(
                ValidationProblem::NotFinite,
                ValidationProblemPosition {
                    geometry,
                    coordinate_index: Some(i),
                },
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::Coordinate;
    use crate::line_string::LineString3D;
    use crate::point::Point3D;
    use crate::{Euclidean3DGeometry, Geometry};

    #[test]
    fn finite_point_is_valid() {
        let p = Point3D::new(Coordinate::Euclidean, [1.0, 2.0, 3.0]);
        assert!(p
            .validate(ValidationType::DuplicatePoints { tolerance: None })
            .is_none());
    }

    #[test]
    fn non_finite_point_reports_not_finite() {
        let p = Point3D::new(Coordinate::Euclidean, [1.0, f64::NAN, 3.0]);
        let report = p
            .validate(ValidationType::DuplicatePoints { tolerance: None })
            .unwrap();
        assert_eq!(report.error_count(), 1);
        assert_eq!(report.0[0].problem, ValidationProblem::NotFinite);
    }

    #[test]
    fn linestring_reports_each_non_finite_coordinate() {
        let ls = LineString3D::from_coords(
            Coordinate::Euclidean,
            [
                [0.0, 0.0, 0.0],
                [f64::INFINITY, 1.0, 0.0],
                [2.0, f64::NAN, 0.0],
            ],
        );
        let report = ls.validate(ValidationType::TooFewPoints).unwrap();
        assert_eq!(report.error_count(), 2);
        assert_eq!(report.0[0].position.coordinate_index, Some(1));
        assert_eq!(report.0[1].position.coordinate_index, Some(2));
    }

    #[test]
    fn dispatch_reaches_leaf_through_geometry() {
        let g = Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(
            Coordinate::Euclidean,
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
            ValidationType::DuplicateConsecutivePoints { threshold: 0.01 },
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
    const ALL_TYPES: [ValidationType; 13] = [
        ValidationType::Finite,
        ValidationType::TooFewPoints,
        ValidationType::DuplicatePoints { tolerance: None },
        ValidationType::DuplicateConsecutivePoints { threshold: 0.0 },
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
