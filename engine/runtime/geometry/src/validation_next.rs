//! Validations/predicates on geometry types.
//!
//! # Default validation per leaf type
//!
//! Which per-coordinate / per-ring / whole-surface checks run for each leaf type,
//! one row per [`ValidationType`]. `✓` runs, `·` does not (not applicable).
//! Finiteness (the `Finite` row) always runs. Leaf columns fold the 2D/3D pairs:
//! `Point` = Point2D/3D, `LineStr` = LineString2D/3D, `Coll` = Collection2D/3D.
//!
//! | Check ╲ Leaf            | Point | LineStr | Poly2D | Poly3D | PolyMesh2D | PolyMesh3D | TriMesh2D | TriMesh3D | Solid | Csg | PtCloud | Coll |
//! |-------------------------|:-----:|:-------:|:------:|:------:|:----------:|:----------:|:---------:|:---------:|:-----:|:---:|:-------:|:----:|
//! | Finite                  |   ✓   |    ✓    |   ✓    |   ✓    |     ✓      |     ✓      |     ✓     |     ✓     |   ✓   |  ✓  |    ✓    |  ✓   |
//! | TooFewPoints            |   ·   |    ✓    |   ✓    |   ✓    |     ✓      |     ✓      |     ·     |     ·     |   ✓   |  ✓  |    ·    |  ✓   |
//! | UnclosedRing            |   ·   |    ·    |   ✓    |   ✓    |     ✓      |     ✓      |     ·     |     ·     |   ✓   |  ✓  |    ·    |  ✓   |
//! | SelfIntersection        |   ·   |    ✓    |   ✓    |   ✓    |     ✓      |     ✓      |     ·     |     ·     |   ✓   |  ✓  |    ·    |  ✓   |
//! | InteriorRingContainment |   ·   |    ·    |   ✓    |   ✓    |     ✓      |     ✓      |     ·     |     ·     |   ·   |  ✓  |    ·    |  ✓   |
//! | Degenerate              |   ·   |    ✓    |   ✓    |   ✓    |     ✓      |     ✓      |     ✓     |     ✓     |   ✓   |  ✓  |    ·    |  ✓   |
//! | Planarity               |   ·   |    ·    |   ✓    |   ✓    |     ·      |     ·      |     ·     |     ·     |   ·   |  ✓  |    ·    |  ✓   |
//! | DuplicatePoints         |   ·   |    ✓    |   ✓    |   ✓    |     ✓      |     ✓      |     ✓     |     ✓     |   ✓   |  ✓  |    ✓    |  ✓   |
//! | Orientation             |   ·   |    ·    |   ✓    |   ·    |     ✓      |     ✓      |     ✓     |     ✓     |   ✓   |  ✓  |    ·    |  ✓   |
//! | Orientable              |   ·   |    ·    |   ·    |   ·    |     ·      |     ✓      |     ·     |     ✓     |   ✓   |  ✓  |    ·    |  ✓   |
//! | ShellManifold           |   ·   |    ·    |   ·    |   ·    |     ·      |     ·      |     ·     |     ·     |   ✓   |  ✓  |    ·    |  ✓   |
//! | ShellOrientation        |   ·   |    ·    |   ·    |   ·    |     ·      |     ·      |     ·     |     ·     |   ✓   |  ✓  |    ·    |  ✓   |
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
//! | `UnclosedRing`               | `Finite` |
//! | `DuplicatePoints`            | `Finite` |
//! | `Degenerate`                 | `Finite` |
//! | `Planarity`                  | `Finite` |
//! | `SelfIntersection`           | `Finite`, `TooFewPoints`, `UnclosedRing` |
//! | `InteriorRingContainment`    | `Finite`, `SelfIntersection` |
//! | `Orientable`                 | — |
//! | `Orientation`                | `Finite`, `Orientable` |
//! | `ShellManifold`              | — |
//! | `ShellOrientation`           | `Orientation`, `ShellManifold` |

use std::collections::{HashMap, HashSet};
use std::fmt;

use kiddo::{KdTree, SquaredEuclidean};
use serde::Serialize;

use crate::coordinate::CoordinateFrame;
use crate::csg::{Csg, ThreeDimensional};
use crate::line_string::{LineString2D, LineString3D};
use crate::point::{Point2D, Point3D};
use crate::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};

/// Which validity check to run — one variant per row of the
/// [validation matrix](self#default-validation-per-leaf-type).
///
/// `Finite`, `TooFewPoints`, `UnclosedRing`, `DuplicatePoints`, `Orientation`
/// (meshes / 2D faces), `Orientable` (3D meshes and solids), and `ShellManifold` /
/// `ShellOrientation` (solids) are implemented; the detection for every other
/// variant is a `TODO` that panics via `unimplemented!()` if a leaf lists it in
/// its [applicable checks](Validate::applicable_checks) and [`validate`] reaches
/// it. Checks carry no parameters: [`validate`] runs each with its default
/// tolerance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub enum ValidationType {
    /// Every coordinate component is finite (non-NaN, non-infinite). Always run.
    Finite,
    /// A line or ring has fewer points than its type requires (line ≥ 2, closed
    /// ring ≥ 4).
    TooFewPoints,
    /// Coordinates that coincide anywhere within a geometry.
    DuplicatePoints,
    /// A ring is not closed (first vertex != last).
    UnclosedRing,
    /// A ring or boundary crosses itself.
    SelfIntersection,
    /// An interior ring (hole) is not contained in its exterior ring.
    InteriorRingContainment,
    /// The geometry has zero or near-zero extent (length, area, or volume).
    Degenerate,
    /// A polygon's ring vertices do not all lie in a common plane (within
    /// tolerance). Meaningful for a face embedded in 3D — or a 2.5D face carrying
    /// per-vertex elevation. Polygon only.
    Planarity,
    /// The surface admits no consistent orientation at all — a Möbius-like
    /// contradiction or a non-manifold edge (shared by more than two faces) — so
    /// no assignment of face flips can make every shared edge agree. This is the
    /// topological prerequisite of [`Orientation`](ValidationType::Orientation),
    /// checked regardless of the surface's current winding.
    Orientable,
    /// The surface is not *consistently* oriented (adjacent face normals
    /// disagree). Type-dependent: a 2D face means ring winding (exterior CCW,
    /// holes CW); a 3D mesh or solid means coherent winding across shared edges
    /// (each shared edge traversed in opposite directions by its two faces).
    Orientation,
    /// A solid's boundary is not a closed 2-manifold (watertight): some shell is
    /// not a single connected component whose every edge is shared by exactly two
    /// faces. Solid only.
    ShellManifold,
    /// A solid's shell normals face the wrong way: the exterior shell must enclose
    /// positive volume (outward normals) and each void shell negative volume
    /// (normals into the void). Defined on a closed, consistently-oriented solid.
    /// Solid only.
    ShellOrientation,
}

impl ValidationType {
    /// Immediate prerequisite checks. [`validate`] marks a check
    /// [`Unvalidated`](ValidationResult::Unvalidated) while any *applicable*
    /// prerequisite did not end in [`Success`](ValidationResult::Success).
    /// Tabulated under [check dependencies](self#check-dependencies).
    pub fn dependencies(&self) -> &'static [ValidationType] {
        use ValidationType::*;
        match self {
            Finite | TooFewPoints | Orientable | ShellManifold => &[],
            UnclosedRing | DuplicatePoints | Degenerate | Planarity => &[Finite],
            SelfIntersection => &[Finite, TooFewPoints, UnclosedRing],
            InteriorRingContainment => &[Finite, SelfIntersection],
            Orientation => &[Finite, Orientable],
            ShellOrientation => &[Orientation, ShellManifold],
        }
    }

    /// Whether this is an optional, advisory quality check rather than a core
    /// validity check. An optional check's failure is a warning about geometry
    /// quality, not proof that the geometry is invalid.
    pub fn is_optional(&self) -> bool {
        matches!(
            self,
            ValidationType::DuplicatePoints
                | ValidationType::Orientable
                | ValidationType::Orientation
                | ValidationType::ShellOrientation
        )
    }
}

impl fmt::Display for ValidationType {
    /// The check's variant name.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            ValidationType::Finite => "Finite",
            ValidationType::TooFewPoints => "TooFewPoints",
            ValidationType::DuplicatePoints => "DuplicatePoints",
            ValidationType::UnclosedRing => "UnclosedRing",
            ValidationType::SelfIntersection => "SelfIntersection",
            ValidationType::InteriorRingContainment => "InteriorRingContainment",
            ValidationType::Degenerate => "Degenerate",
            ValidationType::Planarity => "Planarity",
            ValidationType::Orientable => "Orientable",
            ValidationType::Orientation => "Orientation",
            ValidationType::ShellManifold => "ShellManifold",
            ValidationType::ShellOrientation => "ShellOrientation",
        };
        f.write_str(name)
    }
}

/// The outcome of one [`ValidationType`] check on a geometry.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub enum ValidationResult {
    /// The check ran and found no problems.
    Success,
    /// The check could not run because a prerequisite check did not succeed. (A
    /// check that is applicable but unimplemented panics instead of reaching this
    /// state.)
    Unvalidated,
    /// The check ran and found problems; each [`Geometry`] pinpoints a failing
    /// position (the failed port).
    Failed(Vec<Geometry>),
}

/// Each applicable [`ValidationType`]'s [`ValidationResult`] for one geometry, as
/// returned by [`validate`].
pub type ValidationResults = std::collections::HashMap<ValidationType, ValidationResult>;

/// A validity problem together with where it occurred; the per-check accumulator
/// the `check_*` helpers push into. [`CheckOutcome::ran`] keeps only the positions.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct ValidationProblemAtPosition {
    /// The problem encountered.
    pub problem: String,
    /// The geometry pinpointing where the problem was found — typically a point
    /// leaf at the offending coordinate.
    pub position: Geometry,
}

/// The problems one `check_*` helper found, before it becomes a [`CheckOutcome`].
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
}

/// What running one check produced before dependency gating: the positions it
/// flagged, empty when the geometry passed.
///
/// A check that *does not run for a leaf type* never produces a `CheckOutcome` —
/// it is simply absent from that leaf's
/// [`applicable_checks`](Validate::applicable_checks), so the driver never calls
/// it. A check that is applicable but *not yet implemented* panics via
/// `unimplemented!()` rather than yielding an outcome. So a `CheckOutcome` always
/// means "this check genuinely ran".
pub(crate) struct CheckOutcome(Vec<Geometry>);

impl CheckOutcome {
    /// Run a `check_*` helper into a fresh report and keep only its positions.
    pub(crate) fn ran(fill: impl FnOnce(&mut ValidationReport)) -> Self {
        let mut report = ValidationReport::default();
        fill(&mut report);
        CheckOutcome(report.0.into_iter().map(|p| p.position).collect())
    }

    /// Reduce to a gated result: no positions →
    /// [`Success`](ValidationResult::Success), otherwise
    /// [`Failed`](ValidationResult::Failed).
    fn into_result(self) -> ValidationResult {
        if self.0.is_empty() {
            ValidationResult::Success
        } else {
            ValidationResult::Failed(self.0)
        }
    }
}

/// Run each of a leaf's `applicable` checks, honoring
/// [dependencies](ValidationType::dependencies), and collect every check's
/// [`ValidationResult`]. `run_one` executes a single check assuming its
/// prerequisites passed; it is invoked at most once per check.
pub(crate) fn run_checks(
    applicable: &[ValidationType],
    mut run_one: impl FnMut(ValidationType) -> CheckOutcome,
) -> ValidationResults {
    let applicable_set: HashSet<ValidationType> = applicable.iter().copied().collect();
    let mut results = ValidationResults::new();
    for &check in applicable {
        resolve(check, &applicable_set, &mut results, &mut run_one);
    }
    results
}

/// Resolve one check, recursing into its applicable prerequisites first. A check
/// is [`Unvalidated`](ValidationResult::Unvalidated) when any applicable
/// prerequisite did not end in [`Success`](ValidationResult::Success). A check
/// that is applicable but unimplemented panics (via the `Validate` default body)
/// rather than resolving.
fn resolve(
    check: ValidationType,
    applicable: &HashSet<ValidationType>,
    results: &mut ValidationResults,
    run_one: &mut impl FnMut(ValidationType) -> CheckOutcome,
) -> ValidationResult {
    if let Some(result) = results.get(&check) {
        return result.clone();
    }
    let mut blocked = false;
    for &dep in check.dependencies() {
        if applicable.contains(&dep)
            && resolve(dep, applicable, results, run_one) != ValidationResult::Success
        {
            blocked = true;
        }
    }
    let result = if blocked {
        ValidationResult::Unvalidated
    } else {
        run_one(check).into_result()
    };
    results.insert(check, result.clone());
    result
}

/// Fold one member's results into an aggregate's, combining each check: any
/// `Failed` wins (positions concatenated), then any `Unvalidated`, else `Success`.
pub(crate) fn merge_results(acc: &mut ValidationResults, other: ValidationResults) {
    for (check, incoming) in other {
        let combined = match acc.remove(&check) {
            Some(existing) => combine_results(existing, incoming),
            None => incoming,
        };
        acc.insert(check, combined);
    }
}

fn combine_results(a: ValidationResult, b: ValidationResult) -> ValidationResult {
    use ValidationResult::*;
    match (a, b) {
        (Failed(mut xs), Failed(ys)) => {
            xs.extend(ys);
            Failed(xs)
        }
        (Failed(xs), _) | (_, Failed(xs)) => Failed(xs),
        (Unvalidated, _) | (_, Unvalidated) => Unvalidated,
        (Success, Success) => Success,
    }
}

/// Generate the per-check [`Validate`] trait, its `Box` forwarding impl, and the
/// [`dispatch`] bridge from one `method => ValidationType` table, so the mapping
/// has a single source of truth. Each check method defaults to
/// [`CheckOutcome::NotImplemented`]; a leaf overrides only the ones it supports.
macro_rules! validation_checks {
    ($($(#[$m:meta])* $method:ident => $variant:ident),+ $(,)?) => {
        /// Per-leaf validation: one method per [`ValidationType`], plus
        /// [`applicable_checks`](Validate::applicable_checks) — the leaf's row of the
        /// [validation matrix](self#default-validation-per-leaf-type).
        ///
        /// A leaf overrides `applicable_checks` and the handful of check methods it
        /// implements; every other method falls to the
        /// [`NotImplemented`](CheckOutcome::NotImplemented) default. The driver
        /// ([`validate`]) only ever calls a method listed in `applicable_checks`, so
        /// an inapplicable check's default body is never observed. This trait is the
        /// dispatched primitive; the free function [`validate`] owns the dependency
        /// gating and aggregate recursion.
        #[enum_dispatch::enum_dispatch]
        pub(crate) trait Validate {
            /// This leaf's row of the matrix: every check that applies to it.
            /// Defaults to none — the aggregates ([`Csg`](crate::csg::Csg), the
            /// collections) validate by recursing into their members, not by direct
            /// checks.
            fn applicable_checks(&self) -> &'static [ValidationType] {
                &[]
            }
            $(
                $(#[$m])*
                fn $method(&self) -> CheckOutcome {
                    // Reached only when this check is listed in the leaf's
                    // `applicable_checks` but its detection is not yet written —
                    // a genuine `TODO`, made loud rather than silently skipped.
                    unimplemented!(concat!(
                        stringify!($variant),
                        " validation is not implemented for this geometry type"
                    ))
                }
            )+
        }

        // The boxed enum variants (`Box<Polygon2D>`, `Box<Solid>`, …) need the trait
        // on the `Box` itself: `enum_dispatch` forwards by UFCS, not auto-deref.
        impl<T: Validate + ?Sized> Validate for Box<T> {
            fn applicable_checks(&self) -> &'static [ValidationType] {
                (**self).applicable_checks()
            }
            $(
                fn $method(&self) -> CheckOutcome {
                    (**self).$method()
                }
            )+
        }

        /// Run a single check by routing a runtime [`ValidationType`] to the matching
        /// [`Validate`] method. Works on the enums (via `enum_dispatch`) and on a
        /// bare leaf such as a CSG operand's `&Solid`.
        fn dispatch<T: Validate + ?Sized>(leaf: &T, check: ValidationType) -> CheckOutcome {
            match check {
                $( ValidationType::$variant => leaf.$method(), )+
            }
        }
    };
}

validation_checks! {
    /// [`Finite`](ValidationType::Finite).
    check_finite => Finite,
    /// [`TooFewPoints`](ValidationType::TooFewPoints).
    check_too_few_points => TooFewPoints,
    /// [`DuplicatePoints`](ValidationType::DuplicatePoints).
    check_duplicate_points => DuplicatePoints,
    /// [`UnclosedRing`](ValidationType::UnclosedRing).
    check_unclosed_ring => UnclosedRing,
    /// [`SelfIntersection`](ValidationType::SelfIntersection).
    check_self_intersection => SelfIntersection,
    /// [`InteriorRingContainment`](ValidationType::InteriorRingContainment).
    check_interior_ring_containment => InteriorRingContainment,
    /// [`Degenerate`](ValidationType::Degenerate).
    check_degenerate => Degenerate,
    /// [`Planarity`](ValidationType::Planarity).
    check_planarity => Planarity,
    /// [`Orientable`](ValidationType::Orientable).
    check_orientable => Orientable,
    /// [`Orientation`](ValidationType::Orientation).
    check_orientation => Orientation,
    /// [`ShellManifold`](ValidationType::ShellManifold).
    check_shell_manifold => ShellManifold,
    /// [`ShellOrientation`](ValidationType::ShellOrientation).
    check_shell_orientation => ShellOrientation,
}

/// Validate a geometry by running every check that
/// [applies to its leaf type](self#default-validation-per-leaf-type), honoring
/// [dependencies](ValidationType::dependencies).
///
/// Each applicable check maps to a [`ValidationResult`]; checks whose
/// prerequisites failed (or whose detection is a `TODO`) are
/// [`Unvalidated`](ValidationResult::Unvalidated). Aggregates (the collections,
/// [`Csg`](crate::csg::Csg), [`GeometryCollection`](crate::GeometryCollection))
/// recurse into their members and merge the per-check results with
/// [`merge_results`].
pub fn validate(geometry: &Geometry) -> ValidationResults {
    match geometry {
        // An absent geometry has nothing to validate.
        Geometry::None => ValidationResults::new(),
        Geometry::Euclidean2D(g) => validate_2d(g),
        Geometry::Euclidean3D(g) => validate_3d(g),
        Geometry::GeometryCollection(c) => merge_members(c.members().iter().map(validate)),
    }
}

/// Run one leaf's applicable checks under dependency gating. The entry point for a
/// concrete leaf (and the per-leaf unit tests); [`validate`] routes enum variants
/// here after handling aggregates.
pub(crate) fn validate_leaf<T: Validate + ?Sized>(leaf: &T) -> ValidationResults {
    run_checks(leaf.applicable_checks(), |check| dispatch(leaf, check))
}

/// Resolve a single check for a leaf, running only its applicable prerequisites —
/// not the leaf's other checks. Lets the per-check unit tests exercise one check
/// without tripping an unrelated, still-`unimplemented!()` sibling on the same
/// leaf.
#[cfg(test)]
pub(crate) fn validate_one<T: Validate + ?Sized>(
    leaf: &T,
    check: ValidationType,
) -> ValidationResult {
    let applicable: HashSet<ValidationType> = leaf.applicable_checks().iter().copied().collect();
    let mut results = ValidationResults::new();
    resolve(check, &applicable, &mut results, &mut |c| dispatch(leaf, c))
}

fn validate_2d(g: &Euclidean2DGeometry) -> ValidationResults {
    match g {
        Euclidean2DGeometry::Collection(c) => merge_members(c.members().iter().map(validate_2d)),
        leaf => validate_leaf(leaf),
    }
}

fn validate_3d(g: &Euclidean3DGeometry) -> ValidationResults {
    match g {
        Euclidean3DGeometry::Collection(c) => merge_members(c.members().iter().map(validate_3d)),
        Euclidean3DGeometry::Csg(csg) => validate_csg(csg),
        leaf => validate_leaf(leaf),
    }
}

/// Recurse into a CSG tree, merging both operands' results — a `Csg` carries no
/// coordinates of its own.
fn validate_csg(csg: &Csg) -> ValidationResults {
    let (left, right) = match csg {
        Csg::Union(a, b) | Csg::Intersection(a, b) | Csg::Difference(a, b) => (a, b),
    };
    merge_members([left, right].into_iter().map(|op| validate_operand(op)))
}

fn validate_operand(operand: &ThreeDimensional) -> ValidationResults {
    match operand {
        ThreeDimensional::Solid(solid) => validate_leaf(solid.as_ref()),
        ThreeDimensional::Csg(csg) => validate_csg(csg),
    }
}

/// Merge a sequence of per-member results into one aggregate map.
fn merge_members(members: impl IntoIterator<Item = ValidationResults>) -> ValidationResults {
    let mut acc = ValidationResults::new();
    for member in members {
        merge_results(&mut acc, member);
    }
    acc
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
    let label = ValidationType::DuplicatePoints.to_string();
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
    let label = ValidationType::DuplicatePoints.to_string();
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

/// Twice the signed area of a 2D ring (shoelace), wrapping the last vertex back
/// to the first. Positive = counter-clockwise, negative = clockwise, zero =
/// degenerate / collinear.
fn signed_area_2d(ring: &[[f64; 2]]) -> f64 {
    let n = ring.len();
    let mut acc = 0.0;
    for i in 0..n {
        let a = ring[i];
        let b = ring[(i + 1) % n];
        acc += a[0] * b[1] - b[0] * a[1];
    }
    acc
}

/// Report a [`ValidationType::Orientation`] problem when a 2D ring winds the
/// wrong way: an exterior ring must be counter-clockwise, a hole clockwise.
/// A zero-area (degenerate / collinear) ring has no meaningful winding and is
/// left to the degeneracy check. The position is the offending ring.
pub(crate) fn check_ring_orientation_2d(
    frame: &CoordinateFrame,
    ring: &[[f64; 2]],
    is_exterior: bool,
    report: &mut ValidationReport,
) {
    let area = signed_area_2d(ring);
    let wrong = if is_exterior { area < 0.0 } else { area > 0.0 };
    if wrong {
        report.push(
            ValidationType::Orientation.to_string(),
            Geometry::Euclidean2D(Euclidean2DGeometry::LineString(LineString2D::from_coords(
                frame.clone(),
                ring.iter().copied(),
            ))),
        );
    }
}

/// Report a [`ValidationType::Orientation`] problem for each 3D face ring whose
/// winding disagrees with a neighbour: on a consistently-oriented surface every
/// shared edge is traversed in opposite directions by its two faces, so a
/// directed edge `(a, b)` seen from two faces means they wind the same way
/// across it. `rings` yields each face's vertex indices (closure optional; the
/// last vertex wraps to the first). The position is the offending ring.
pub(crate) fn check_edge_orientation_3d<R: AsRef<[u32]>>(
    frame: &CoordinateFrame,
    vertices: &[[f64; 3]],
    rings: impl IntoIterator<Item = R>,
    report: &mut ValidationReport,
) {
    let mut seen: HashSet<(u32, u32)> = HashSet::new();
    for ring in rings {
        let ring = ring.as_ref();
        let n = ring.len();
        if n < 2 {
            continue;
        }
        let mut conflict = false;
        for i in 0..n {
            let (a, b) = (ring[i], ring[(i + 1) % n]);
            if a == b {
                continue;
            }
            if !seen.insert((a, b)) {
                conflict = true;
                break;
            }
        }
        if conflict {
            let coords: Vec<[f64; 3]> = ring.iter().map(|&i| vertices[i as usize]).collect();
            report.push(
                ValidationType::Orientation.to_string(),
                Geometry::Euclidean3D(Euclidean3DGeometry::LineString(LineString3D::from_coords(
                    frame.clone(),
                    coords,
                ))),
            );
        }
    }
}

/// Six times the signed volume of the tetrahedron from the origin to triangle
/// `(a, b, c)`: the scalar triple product `a · (b × c)`. Summed over a closed
/// surface's triangles and divided by six it gives the enclosed signed volume,
/// whose sign follows the surface's orientation.
pub(crate) fn tetra_volume_6x(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> f64 {
    let cross = [
        b[1] * c[2] - b[2] * c[1],
        b[2] * c[0] - b[0] * c[2],
        b[0] * c[1] - b[1] * c[0],
    ];
    a[0] * cross[0] + a[1] * cross[1] + a[2] * cross[2]
}

/// Union-find with edge parity: each element carries a bit relative to its set
/// representative, so a "same" (`0`) or "different" (`1`) constraint between two
/// elements can be recorded and contradictions detected. Backs the connectivity
/// and orientability checks.
struct ParityUnionFind {
    parent: Vec<usize>,
    /// Parity of each element relative to its parent.
    parity: Vec<u8>,
    rank: Vec<u8>,
}

impl ParityUnionFind {
    fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
            parity: vec![0; n],
            rank: vec![0; n],
        }
    }

    /// The set root of `x` and `x`'s parity relative to that root, compressing
    /// the path on the way out.
    fn find(&mut self, x: usize) -> (usize, u8) {
        if self.parent[x] == x {
            return (x, 0);
        }
        let (root, p) = self.find(self.parent[x]);
        self.parent[x] = root;
        self.parity[x] ^= p;
        (root, self.parity[x])
    }

    /// Record that `x` and `y` differ by `rel` (`0` = same, `1` = opposite),
    /// merging their sets. Returns `false` if this contradicts an existing
    /// constraint (they are already related with the other parity).
    fn union(&mut self, x: usize, y: usize, rel: u8) -> bool {
        let (rx, px) = self.find(x);
        let (ry, py) = self.find(y);
        if rx == ry {
            return px ^ py == rel;
        }
        let new_parity = px ^ py ^ rel;
        if self.rank[rx] < self.rank[ry] {
            self.parent[rx] = ry;
            self.parity[rx] = new_parity;
        } else {
            self.parent[ry] = rx;
            self.parity[ry] = new_parity;
            if self.rank[rx] == self.rank[ry] {
                self.rank[rx] += 1;
            }
        }
        true
    }
}

/// Face-adjacency topology of a surface: which faces meet at each undirected edge
/// and in which direction, plus the face count. Built from the faces' vertex-index
/// rings and shared by the mesh modules' `pub(crate)` connectivity, `Orientable`,
/// and `ShellManifold` helpers.
pub(crate) struct FaceTopology {
    n_faces: usize,
    /// Undirected edge `(min, max)` → its incident faces, each flagged `true`
    /// when that face traverses the edge low → high.
    edges: HashMap<(u32, u32), Vec<(usize, bool)>>,
}

impl FaceTopology {
    /// Build from one vertex-index ring per face (closure optional; the last
    /// vertex wraps to the first). Self-loop edges (`a == b`) are skipped.
    pub(crate) fn from_faces<R: AsRef<[u32]>>(faces: impl IntoIterator<Item = R>) -> Self {
        let mut edges: HashMap<(u32, u32), Vec<(usize, bool)>> = HashMap::new();
        let mut n_faces = 0usize;
        for ring in faces {
            let ring = ring.as_ref();
            let f = n_faces;
            n_faces += 1;
            let n = ring.len();
            if n < 2 {
                continue;
            }
            for i in 0..n {
                let (a, b) = (ring[i], ring[(i + 1) % n]);
                if a == b {
                    continue;
                }
                let (key, forward) = if a < b {
                    ((a, b), true)
                } else {
                    ((b, a), false)
                };
                edges.entry(key).or_default().push((f, forward));
            }
        }
        Self { n_faces, edges }
    }

    /// Whether every edge is shared by exactly two faces — a closed 2-manifold
    /// boundary (watertight: no boundary edges, no non-manifold edges).
    pub(crate) fn is_closed_manifold(&self) -> bool {
        !self.edges.is_empty() && self.edges.values().all(|inc| inc.len() == 2)
    }

    /// Whether the faces form a single connected component through shared edges.
    pub(crate) fn is_connected(&self) -> bool {
        if self.n_faces == 0 {
            return false;
        }
        let mut uf = ParityUnionFind::new(self.n_faces);
        for inc in self.edges.values() {
            for w in inc.windows(2) {
                uf.union(w[0].0, w[1].0, 0);
            }
        }
        let root = uf.find(0).0;
        (1..self.n_faces).all(|f| uf.find(f).0 == root)
    }

    /// Whether a consistent orientation exists: no edge is shared by more than two
    /// faces, and the per-face flip constraints have no contradiction.
    pub(crate) fn is_orientable(&self) -> bool {
        let mut uf = ParityUnionFind::new(self.n_faces);
        for inc in self.edges.values() {
            if inc.len() > 2 {
                return false;
            }
            if inc.len() == 2 {
                let (f1, forward1) = inc[0];
                let (f2, forward2) = inc[1];
                // Consistent orientation traverses a shared edge in opposite
                // directions. If both faces traverse it the same way they must
                // flip oppositely (parity 1); otherwise they agree (parity 0).
                let rel = (forward1 == forward2) as u8;
                if !uf.union(f1, f2, rel) {
                    return false;
                }
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::CoordinateFrame;
    use crate::line_string::LineString3D;
    use crate::point::Point3D;
    use crate::{Euclidean3DGeometry, Geometry};

    /// The `[x, y, z]` of the 3D point leaf a failing position points at.
    fn offending_point(position: &Geometry) -> [f64; 3] {
        match position {
            Geometry::Euclidean3D(Euclidean3DGeometry::Point(p)) => p.position(),
            other => panic!("expected a 3D point position, got {other:?}"),
        }
    }

    /// The failing positions recorded for `check`, or a panic if it did not fail.
    fn failures(results: &ValidationResults, check: ValidationType) -> Vec<Geometry> {
        match results.get(&check) {
            Some(ValidationResult::Failed(positions)) => positions.clone(),
            other => panic!("expected {check} to fail, got {other:?}"),
        }
    }

    /// The failing positions of a single [`validate_one`] result, or a panic if it
    /// did not fail.
    fn one_failure(result: ValidationResult) -> Vec<Geometry> {
        match result {
            ValidationResult::Failed(positions) => positions,
            other => panic!("expected a failure, got {other:?}"),
        }
    }

    #[test]
    fn finite_point_is_valid() {
        let p = Point3D::new(CoordinateFrame::Euclidean, [1.0, 2.0, 3.0]);
        assert_eq!(
            validate_leaf(&p)[&ValidationType::Finite],
            ValidationResult::Success
        );
    }

    #[test]
    fn non_finite_point_reports_not_finite() {
        let p = Point3D::new(CoordinateFrame::Euclidean, [1.0, f64::NAN, 3.0]);
        let positions = failures(&validate_leaf(&p), ValidationType::Finite);
        assert_eq!(positions.len(), 1);
        assert!(matches!(
            positions[0],
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
        let positions = failures(&validate_leaf(&ls), ValidationType::Finite);
        assert_eq!(positions.len(), 2);
        // Each position is a 3D point leaf holding the offending coordinate.
        let first = offending_point(&positions[0]);
        assert!(first[0].is_infinite());
        assert_eq!(first[1], 1.0);
        let second = offending_point(&positions[1]);
        assert!(second[1].is_nan());
        assert_eq!(second[0], 2.0);
    }

    #[test]
    fn too_few_points_flags_single_point_line() {
        let ls = LineString3D::from_coords(CoordinateFrame::Euclidean, [[0.0, 0.0, 0.0]]);
        let positions = one_failure(validate_one(&ls, ValidationType::TooFewPoints));
        assert_eq!(positions.len(), 1);
        // The position is the offending line, not a single coordinate.
        assert!(matches!(
            positions[0],
            Geometry::Euclidean3D(Euclidean3DGeometry::LineString(_))
        ));
    }

    #[test]
    fn two_point_line_has_enough_points() {
        let ls = LineString3D::from_coords(
            CoordinateFrame::Euclidean,
            [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
        );
        assert_eq!(
            validate_one(&ls, ValidationType::TooFewPoints),
            ValidationResult::Success
        );
    }

    #[test]
    fn duplicate_points_exact_flags_repeated_coordinate() {
        let ls = LineString3D::from_coords(
            CoordinateFrame::Euclidean,
            [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 0.0]],
        );
        let positions = one_failure(validate_one(&ls, ValidationType::DuplicatePoints));
        assert_eq!(positions.len(), 1);
        assert_eq!(offending_point(&positions[0]), [0.0, 0.0, 0.0]);
    }

    #[test]
    fn dispatch_reaches_leaf_through_geometry() {
        let g = Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(
            CoordinateFrame::Euclidean,
            [f64::NAN, 0.0, 0.0],
        )));
        assert_eq!(failures(&validate(&g), ValidationType::Finite).len(), 1);
    }

    #[test]
    fn none_geometry_yields_no_checks() {
        assert!(validate(&Geometry::None).is_empty());
    }

    #[test]
    fn a_leaf_runs_exactly_its_applicable_checks() {
        // A point's only applicable check is finiteness.
        let keys: Vec<_> = validate_leaf(&Point3D::new(CoordinateFrame::Euclidean, [0.0; 3]))
            .into_keys()
            .collect();
        assert_eq!(keys, [ValidationType::Finite]);
    }

    #[test]
    #[should_panic(expected = "SelfIntersection validation is not implemented")]
    fn applicable_but_unimplemented_check_panics() {
        // `SelfIntersection` applies to a line string but is a TODO; reaching it
        // (its prerequisites hold) must panic rather than report `Unvalidated`.
        let ls = LineString3D::from_coords(
            CoordinateFrame::Euclidean,
            [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
        );
        let _ = validate_one(&ls, ValidationType::SelfIntersection);
    }

    #[test]
    fn failed_prerequisite_leaves_dependents_unvalidated() {
        // A non-finite coordinate fails `Finite`, so `DuplicatePoints` (which
        // depends on it) cannot run.
        let ls = LineString3D::from_coords(
            CoordinateFrame::Euclidean,
            [[0.0, 0.0, 0.0], [f64::NAN, 0.0, 0.0]],
        );
        let results = validate_leaf(&ls);
        assert!(matches!(
            results[&ValidationType::Finite],
            ValidationResult::Failed(_)
        ));
        assert_eq!(
            results[&ValidationType::DuplicatePoints],
            ValidationResult::Unvalidated
        );
    }

    #[test]
    fn optional_checks_are_flagged_optional() {
        // The optional, advisory checks.
        for optional in [
            ValidationType::DuplicatePoints,
            ValidationType::Orientable,
            ValidationType::Orientation,
            ValidationType::ShellOrientation,
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
            ValidationType::SelfIntersection,
            ValidationType::InteriorRingContainment,
            ValidationType::Degenerate,
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
        ValidationType::DuplicatePoints,
        ValidationType::UnclosedRing,
        ValidationType::SelfIntersection,
        ValidationType::InteriorRingContainment,
        ValidationType::Degenerate,
        ValidationType::Planarity,
        ValidationType::Orientable,
        ValidationType::Orientation,
        ValidationType::ShellManifold,
        ValidationType::ShellOrientation,
    ];

    #[test]
    fn documented_dependencies_hold() {
        assert_eq!(
            ValidationType::Orientation.dependencies(),
            &[ValidationType::Finite, ValidationType::Orientable]
        );
        assert_eq!(
            ValidationType::ShellOrientation.dependencies(),
            &[ValidationType::Orientation, ValidationType::ShellManifold]
        );
        assert_eq!(
            ValidationType::InteriorRingContainment.dependencies(),
            &[ValidationType::Finite, ValidationType::SelfIntersection]
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
