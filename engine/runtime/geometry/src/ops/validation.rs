//! Validate geometry types, reporting structural problems.
//!
//! Mirrors the `ops` pattern: a [`Validate`] trait carrying a parameterized
//! `validate` method, `#[enum_dispatch]`-ed so a call on `Geometry` /
//! `Euclidean{2,3}DGeometry` chains through to the concrete leaf. Each leaf
//! opts in from its own `{type}/validation.rs`, and the collection-like types
//! (`Geometry`, `GeometryCollection`, `Collection{2,3}D`) recurse by hand over
//! their children.
//!
//! **Scaffold.** This is the initial skeleton for new-geometry validation:
//! coordinate finiteness ([`ValidationProblem::NotFinite`]) is the only check
//! wired up so far, and it runs for every [`ValidationType`]. The remaining
//! checks (too-few-points, ring closure, duplicate / consecutive points,
//! self-intersection, ring containment, …) are represented in the enums and
//! selected by [`ValidationType`], but their detection is left as `TODO`s in
//! the per-leaf modules so the surface can grow without reshaping the API. This
//! is the new-geometry counterpart of the historic [`crate::validation`], which
//! validates the legacy `types::` geometry.
//!
//! # Validation matrix
//!
//! Rows are the geometry leaf types, columns are the checks: the always-on
//! finiteness check plus each [`ValidationType`]. Cell markers:
//!
//! - `✓` — applies, **core**: a failure means the geometry is invalid.
//! - `○` — applies, **optional**: an opt-in quality check whose failure is
//!   advisory, not an invalidity (see [`ValidationType::is_optional`]).
//! - `·` — not applicable to that type.
//! - `↻` — the type owns no coordinates and delegates to its children by
//!   recursion.
//!
//! Only the finiteness column is implemented today; every other `✓`/`○` is a
//! planned check (`TODO` in the per-leaf modules).
//!
//! | Leaf ╲ Check       | Finite | TooFew | Unclosed | SelfInt | Hole | Degen | Manifold | Connected | Dup | DupCons | Planar | Orient | Normal |
//! |--------------------|:------:|:------:|:--------:|:-------:|:----:|:-----:|:--------:|:---------:|:---:|:-------:|:------:|:------:|:------:|
//! | Point2D / 3D       |   ✓    |   ·    |    ·     |    ·    |  ·   |   ·   |    ·     |     ·     |  ·  |    ·    |   ·    |   ·    |   ·    |
//! | LineString2D / 3D  |   ✓    |   ✓    |    ·     |    ✓    |  ·   |   ✓   |    ·     |     ·     |  ○  |    ○    |   ·    |   ·    |   ·    |
//! | Polygon2D          |   ✓    |   ✓    |    ✓     |    ✓    |  ✓   |   ✓   |    ·     |     ·     |  ○  |    ○    |   ·    |   ○    |   ·    |
//! | Polygon3D          |   ✓    |   ✓    |    ✓     |    ✓    |  ✓   |   ✓   |    ·     |     ·     |  ○  |    ○    |   ○    |   ·    |   ·    |
//! | PolygonMesh2D      |   ✓    |   ✓    |    ✓     |    ✓    |  ✓   |   ✓   |    ·     |     ○     |  ○  |    ○    |   ·    |   ○    |   ·    |
//! | PolygonMesh3D      |   ✓    |   ✓    |    ✓     |    ✓    |  ✓   |   ✓   |    ·     |     ○     |  ○  |    ○    |   ○    |   ○    |   ○    |
//! | TriangularMesh2D   |   ✓    |   ·    |    ·     |    ·    |  ·   |   ✓   |    ·     |     ○     |  ○  |    ·    |   ·    |   ○    |   ·    |
//! | TriangularMesh3D   |   ✓    |   ·    |    ·     |    ·    |  ·   |   ✓   |    ·     |     ○     |  ○  |    ·    |   ·    |   ○    |   ○    |
//! | Solid              |   ✓    |   ✓    |    ✓     |    ✓    |  ·   |   ✓   |    ✓     |     ○     |  ○  |    ·    |   ○    |   ○    |   ○    |
//! | Csg                |   ↻    |   ↻    |    ↻     |    ↻    |  ↻   |   ↻   |    ↻     |     ↻     |  ↻  |    ↻    |   ↻    |   ↻    |   ↻    |
//! | PointCloud         |   ✓    |   ·    |    ·     |    ·    |  ·   |   ·   |    ·     |     ·     |  ○  |    ·    |   ·    |   ·    |   ·    |
//! | Collection2D / 3D  |   ↻    |   ↻    |    ↻     |    ↻    |  ↻   |   ↻   |    ↻     |     ↻     |  ↻  |    ↻    |   ↻    |   ↻    |   ↻    |
//!
//! Notes:
//! - **TooFew** is fixed at 3 for a triangle (never violable), so it is `·` for
//!   the triangular meshes.
//! - **Hole** (`InteriorRingContainment`) is a 2D ring-in-ring test over a
//!   polygon / mesh *face*; a `Solid`'s "interiors" are volumetric void *shells*,
//!   a different notion, so `Hole` is `·` for `Solid`.
//! - **Planar** applies only to a genuinely 3D face. A 2D or 2.5D face (2D
//!   coordinates plus a per-vertex elevation) is planar by construction, and a
//!   `TriangularMesh` face is a triangle, so `Planar` is `·` for all of them.
//! - **Orient** (consistency) has no absolute meaning for a lone 3D surface (a
//!   face in space can be viewed from either side), so a standalone `Polygon3D`
//!   is `·`. Where it applies it means: a 2D face → ring winding (exterior CCW,
//!   holes CW); a 3D mesh or solid → *consistent* winding across shared edges, so
//!   adjacent face normals agree (needs connectivity).
//! - **Normal** (direction) is the *absolute* normal orientation, and is only
//!   defined once a surface is closed and consistently oriented: a closed
//!   manifold mesh or a solid's exterior shell must have outward-pointing
//!   normals, and a solid's interior void shells must point inward (into the
//!   void). It is thus `·` for any open surface (a lone polygon, an open mesh);
//!   the `○` on `PolygonMesh3D` / `TriangularMesh3D` applies only when the mesh
//!   is a closed manifold.
//! - **Connected** asks whether the mesh / solid is a single connected component
//!   (shared vertices / edges). A point, a single line, or a single face is
//!   connected by construction, so it is `·` there.
//! - **Manifold** (watertight; every edge shared by exactly two faces) is
//!   meaningful only for the closed `Solid` boundary.
//!
//! # Check parameters
//!
//! Each column maps to a [`ValidationType`] variant (finiteness is always on and
//! has no variant). All distance / length parameters are in the geometry's own
//! coordinate units — metres in a projected CRS, degrees in a geographic CRS —
//! so a caller in a geographic frame must scale thresholds accordingly.
//!
//! | Column     | `ValidationType`              | Parameter                | Tier     | Meaning of the parameter |
//! |------------|-------------------------------|--------------------------|----------|--------------------------|
//! | Finite     | *(always on)*                 | —                        | core     | no parameter; every coordinate component must be finite |
//! | TooFew     | `TooFewPoints`                | —                        | core     | minimum count is intrinsic to the type (line ≥ 2, closed ring ≥ 4) |
//! | Unclosed   | `UnclosedRing`                | —                        | core     | a ring's first vertex must equal its last |
//! | SelfInt    | `SelfIntersection`            | `tolerance: Option<f64>` | core     | overlaps shorter than this are ignored; `None` = exact crossing test |
//! | Hole       | `InteriorRingContainment`     | `tolerance: Option<f64>` | core     | slack for shared-vertex touches when testing that a hole stays inside its exterior; `None` = exact |
//! | Degen      | `Degenerate`                  | `min_extent: f64`        | core     | length / area / volume floor below which the geometry is degenerate |
//! | Manifold   | `ShellManifold`               | —                        | core     | the solid boundary is a closed 2-manifold (watertight) |
//! | Connected  | `Connected`                   | —                        | optional | the mesh / solid is a single connected component |
//! | Dup        | `DuplicatePoints`             | `tolerance: Option<f64>` | optional | max distance under which two coordinates count as coincident; `None` = exact equality |
//! | DupCons    | `DuplicateConsecutivePoints`  | `threshold: f64`         | optional | max distance between adjacent coordinates before they are flagged as a near-duplicate |
//! | Planar     | `Planarity`                   | `max_deviation: f64`     | optional | greatest out-of-plane height a face vertex may have from the best-fit plane before the face is non-planar |
//! | Orient     | `Orientation`                 | —                        | optional | *consistent* winding across the surface (per-type meaning above) |
//! | Normal     | `NormalDirection`             | —                        | optional | absolute normal direction: outward for a closed manifold / solid exterior, inward for solid voids |
//!
//! # Check dependencies
//!
//! A check is only meaningful once the checks it builds on already hold, so each
//! [`CheckKind`] declares its immediate prerequisites via
//! [`CheckKind::dependencies`] (take the transitive closure for the full set).
//! A runner should skip a check — or treat its result as inconclusive — while
//! any prerequisite is failing. For example the outward/inward `Normal` check
//! needs a consistent `Orient` and a closed `Manifold` boundary before "outward"
//! is even defined, and `Orient` in turn needs `Connected`. The relation is a
//! DAG (verified in the unit tests):
//!
//! | Check                        | Immediate prerequisites                       |
//! |------------------------------|-----------------------------------------------|
//! | `Finiteness` (always on)     | — |
//! | `TooFewPoints`               | — |
//! | `Connected`                  | — |
//! | `UnclosedRing`               | `Finiteness` |
//! | `DuplicatePoints`            | `Finiteness` |
//! | `DuplicateConsecutivePoints` | `Finiteness` |
//! | `Planarity`                  | `Finiteness` |
//! | `Degenerate`                 | `Finiteness` |
//! | `SelfIntersection`           | `Finiteness`, `TooFewPoints`, `UnclosedRing` |
//! | `InteriorRingContainment`    | `Finiteness`, `SelfIntersection` |
//! | `ShellManifold`              | `Connected` |
//! | `Orientation`                | `Finiteness`, `Connected` |
//! | `NormalDirection`            | `Orientation`, `ShellManifold` |

use serde::Serialize;

/// Which validity check to run.
///
/// One variant per column of the [validation matrix](self#validation-matrix);
/// see [check parameters](self#check-parameters) for what each parameter means.
/// The always-on finiteness check has no variant — it runs for every
/// `ValidationType`. Detection for every variant here is currently a `TODO`;
/// only finiteness is implemented.
#[derive(Clone, Debug, PartialEq)]
pub enum ValidationType {
    /// A line or ring has fewer points than its type requires. The minimum is
    /// intrinsic to the type (line ≥ 2 points, closed ring ≥ 4), so there is no
    /// parameter.
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
    /// A ring is not closed (first vertex != last). No parameter.
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
    /// A genuinely 3D face's vertices are not coplanar. Not meaningful for a 2D
    /// or 2.5D face (planar by construction) or a triangle.
    Planarity {
        /// Greatest out-of-plane height a vertex may have from the face's
        /// best-fit plane before the face is flagged non-planar, in coordinate
        /// units.
        max_deviation: f64,
    },
    /// The geometry has zero or near-zero extent (length, area, or volume).
    Degenerate {
        /// Length / area / volume floor below which the geometry is degenerate,
        /// in coordinate units.
        min_extent: f64,
    },
    /// A mesh or solid is not a single connected component (by shared vertices /
    /// edges). No parameter.
    Connected,
    /// The surface is not *consistently* oriented — adjacent face normals
    /// disagree. No parameter. The meaning is type-dependent: a 2D face → ring
    /// winding (exterior CCW, holes CW); a 3D mesh or solid → coherent winding
    /// across shared edges (a lone 3D face has no intrinsic orientation). This is
    /// only *relative* consistency; see [`ValidationType::NormalDirection`] for
    /// the absolute outward/inward direction.
    Orientation,
    /// The absolute normal direction is wrong. No parameter. Defined only for a
    /// closed, consistently-oriented surface: a closed manifold mesh or a solid's
    /// exterior shell must have outward-pointing normals, and a solid's interior
    /// void shells must point inward (into the void). Depends on
    /// [`Orientation`](ValidationType::Orientation) and
    /// [`ShellManifold`](ValidationType::ShellManifold).
    NormalDirection,
    /// A solid's boundary is not a closed 2-manifold (watertight). No parameter.
    ShellManifold,
}

/// A parameter-free classifier for the validity checks, including the always-on
/// finiteness check (which has no [`ValidationType`] variant). It expresses the
/// cross-check relationships — optionality and prerequisites — independently of
/// a check's parameters.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CheckKind {
    /// The always-on finiteness check.
    Finiteness,
    /// See [`ValidationType::TooFewPoints`].
    TooFewPoints,
    /// See [`ValidationType::DuplicatePoints`].
    DuplicatePoints,
    /// See [`ValidationType::DuplicateConsecutivePoints`].
    DuplicateConsecutivePoints,
    /// See [`ValidationType::UnclosedRing`].
    UnclosedRing,
    /// See [`ValidationType::SelfIntersection`].
    SelfIntersection,
    /// See [`ValidationType::InteriorRingContainment`].
    InteriorRingContainment,
    /// See [`ValidationType::Planarity`].
    Planarity,
    /// See [`ValidationType::Degenerate`].
    Degenerate,
    /// See [`ValidationType::Connected`].
    Connected,
    /// See [`ValidationType::Orientation`].
    Orientation,
    /// See [`ValidationType::NormalDirection`].
    NormalDirection,
    /// See [`ValidationType::ShellManifold`].
    ShellManifold,
}

impl CheckKind {
    /// The checks that must already hold for this one to be meaningful. A
    /// failing prerequisite makes this check's result unreliable, so a runner
    /// should skip it (or treat it as inconclusive) until the prerequisites
    /// pass. These are *immediate* dependencies; take the transitive closure for
    /// the full prerequisite set. The relation is a DAG (see the unit tests) and
    /// is tabulated under [check dependencies](self#check-dependencies).
    pub fn dependencies(self) -> &'static [CheckKind] {
        use CheckKind::*;
        match self {
            Finiteness | TooFewPoints | Connected => &[],
            UnclosedRing
            | DuplicatePoints
            | DuplicateConsecutivePoints
            | Planarity
            | Degenerate => &[Finiteness],
            SelfIntersection => &[Finiteness, TooFewPoints, UnclosedRing],
            InteriorRingContainment => &[Finiteness, SelfIntersection],
            ShellManifold => &[Connected],
            Orientation => &[Finiteness, Connected],
            NormalDirection => &[Orientation, ShellManifold],
        }
    }

    /// Whether this is an optional, advisory quality check (`○` in the
    /// [validation matrix](self#validation-matrix)) rather than a core validity
    /// check (`✓`). An optional check's failure is a warning about geometry
    /// quality, not proof that the geometry is invalid.
    pub fn is_optional(self) -> bool {
        matches!(
            self,
            CheckKind::DuplicatePoints
                | CheckKind::DuplicateConsecutivePoints
                | CheckKind::Planarity
                | CheckKind::Connected
                | CheckKind::Orientation
                | CheckKind::NormalDirection
        )
    }
}

impl ValidationType {
    /// The parameter-free [`CheckKind`] this check belongs to.
    pub fn kind(&self) -> CheckKind {
        match self {
            ValidationType::TooFewPoints => CheckKind::TooFewPoints,
            ValidationType::DuplicatePoints { .. } => CheckKind::DuplicatePoints,
            ValidationType::DuplicateConsecutivePoints { .. } => {
                CheckKind::DuplicateConsecutivePoints
            }
            ValidationType::UnclosedRing => CheckKind::UnclosedRing,
            ValidationType::SelfIntersection { .. } => CheckKind::SelfIntersection,
            ValidationType::InteriorRingContainment { .. } => CheckKind::InteriorRingContainment,
            ValidationType::Planarity { .. } => CheckKind::Planarity,
            ValidationType::Degenerate { .. } => CheckKind::Degenerate,
            ValidationType::Connected => CheckKind::Connected,
            ValidationType::Orientation => CheckKind::Orientation,
            ValidationType::NormalDirection => CheckKind::NormalDirection,
            ValidationType::ShellManifold => CheckKind::ShellManifold,
        }
    }

    /// Immediate prerequisite check kinds; see [`CheckKind::dependencies`].
    pub fn dependencies(&self) -> &'static [CheckKind] {
        self.kind().dependencies()
    }

    /// Whether this is an optional, advisory quality check; see
    /// [`CheckKind::is_optional`].
    pub fn is_optional(&self) -> bool {
        self.kind().is_optional()
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

/// Where in a geometry a problem occurs.
///
/// Scaffold form: the concrete leaf type plus, when a single coordinate is
/// implicated, its index within that leaf's own coordinate buffer. The richer
/// positional enum of the historic validator (ring role, member index, …) can
/// replace this as the structural checks are filled in.
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

/// Validate a geometry against a chosen [`ValidationType`], returning the
/// problems found or `None` when the geometry is valid for that check.
///
/// The default body reports no problems, so a leaf that opts out (via
/// [`unsupported!`](crate::unsupported)) is treated as always valid; every leaf
/// currently provides its own `{type}/validation.rs` impl.
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
        // The `○` column of the validation matrix.
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
        // The `✓` column of the validation matrix, including the degenerate check.
        for core in [
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

    /// Every `CheckKind` variant, so the dependency graph can be walked in full.
    const ALL_KINDS: [CheckKind; 13] = [
        CheckKind::Finiteness,
        CheckKind::TooFewPoints,
        CheckKind::DuplicatePoints,
        CheckKind::DuplicateConsecutivePoints,
        CheckKind::UnclosedRing,
        CheckKind::SelfIntersection,
        CheckKind::InteriorRingContainment,
        CheckKind::Planarity,
        CheckKind::Degenerate,
        CheckKind::Connected,
        CheckKind::Orientation,
        CheckKind::NormalDirection,
        CheckKind::ShellManifold,
    ];

    #[test]
    fn documented_dependencies_hold() {
        // The example from the docs: the outward/inward normal check needs a
        // consistent orientation and a closed manifold; orientation needs
        // connectivity.
        assert_eq!(
            CheckKind::NormalDirection.dependencies(),
            &[CheckKind::Orientation, CheckKind::ShellManifold]
        );
        assert_eq!(
            CheckKind::Orientation.dependencies(),
            &[CheckKind::Finiteness, CheckKind::Connected]
        );
        assert!(CheckKind::Finiteness.dependencies().is_empty());
    }

    #[test]
    fn dependency_graph_is_acyclic() {
        // Depth-first cycle detection over the full dependency relation.
        fn reaches(from: CheckKind, target: CheckKind) -> bool {
            from.dependencies()
                .iter()
                .any(|&dep| dep == target || reaches(dep, target))
        }
        for kind in ALL_KINDS {
            assert!(!reaches(kind, kind), "{kind:?} depends on itself");
        }
    }
}
