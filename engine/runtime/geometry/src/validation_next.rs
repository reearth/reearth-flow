//! Validations/predicates on geometry types.

use std::collections::{HashMap, HashSet};
use std::fmt;

use kiddo::{KdTree, SquaredEuclidean};
use serde::{Deserialize, Serialize};

use crate::coordinate::CoordinateFrame;
use crate::csg::{Csg, ThreeDimensional};
use crate::line_string::{LineString2D, LineString3D};
use crate::point::{Point2D, Point3D};
use crate::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};

/// Type of validity check. One variant per row of the
/// [validation matrix](validate#default-validation-per-leaf-type).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub enum ValidationType {
    /// Whether every coordinate component is finite (non-NaN, non-infinite).
    Finite,
    /// Whether a line or ring has fewer points than its type requires (line ≥ 2,
    /// closed ring ≥ 4).
    TooFewPoints,
    /// Whether coordinates coincide anywhere within a geometry.
    DuplicatePoints,
    /// Whether a ring is closed (first vertex == last).
    UnclosedRing,
    /// Whether a ring or boundary crosses itself.
    SelfIntersection,
    /// Whether an interior ring (hole) is contained in its exterior ring.
    InteriorRingContainment,
    /// Whether the geometry has zero or near-zero measure (length, area, or volume).
    Degenerate,
    /// Whether a polygon's ring vertices do all lie in a common plane (within
    /// tolerance).
    Planarity,
    /// Whether the surface admits a consistent orientation, so assignment of face
    /// flips can make every shared edge agree. This is the topological prerequisite
    /// of [`Orientation`](ValidationType::Orientation),
    /// checked regardless of the surface's current winding.
    Orientable,
    /// Whether the surface is consistently oriented. Type-dependent: a 2D face means
    /// ring winding. Flow's orientation convention for 2D geometry is
    /// counter-clockwise (CCW): an exterior ring must wind CCW (positive signed
    /// area) and each interior ring (hole) clockwise. A 3D face has no absolute
    /// winding, so its convention is relative: each hole winds opposite the exterior.
    /// A 3D mesh or solid means coherent winding across shared edges (each shared
    /// edge traversed in opposite directions by its two faces).
    Orientation,
    /// Whether a solid's boundary is not a closed 2-manifold (watertight). Solid only.
    ShellManifold,
    /// Whether a solid's shell normals face the correct way: the exterior shell must enclose
    /// positive volume (outward normals) and each void shell negative volume
    /// (normals into the void). Defined on a closed, consistently-oriented solid.
    /// Solid only.
    ShellOrientation,
}

impl ValidationType {
    /// Immediate prerequisite checks. [`validate`] marks a check
    /// [`Unvalidated`](ValidationResult::Unvalidated) while any applicable
    /// prerequisite did not end in [`Success`](ValidationResult::Success).
    /// Tabulated under [check dependencies](validate#check-dependencies).
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
    /// The check's variant name, as produced by the derived [`Debug`].
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

/// Tunable thresholds for the checks that admit them. Which checks *run* is fixed
/// per leaf type ([`applicable_checks`](Validate::applicable_checks)); these only
/// tune *how* a check decides. [`Default`] gives the strictest sensible behavior
/// (exact-equality duplicates, zero-tolerance planarity and degeneracy), so an
/// omitted field means "no slack".
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ValidationParams {
    /// [`DuplicatePoints`](ValidationType::DuplicatePoints): `None` = exact bit
    /// equality; `Some(t)` = two coordinates coincide when within distance `t`.
    pub duplicate_tolerance: Option<f64>,
    /// [`Planarity`](ValidationType::Planarity): the greatest distance a ring
    /// vertex may sit off the ring's best-fit plane before the ring is non-planar.
    pub planarity_tolerance: f64,
    /// [`Degenerate`](ValidationType::Degenerate): the smallest measure a geometry
    /// may have before it counts as degenerate.
    pub degenerate: DegenerateThresholds,
}

/// The per-dimension measures below which a geometry is
/// [`Degenerate`](ValidationType::Degenerate). Each applies to the geometries of
/// its dimension: `min_length` to lines, `min_area` to faces, `min_volume` to
/// solids.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct DegenerateThresholds {
    /// Minimum length of a 1D geometry (line / ring edge).
    pub min_length: f64,
    /// Minimum area of a 2D geometry (face / ring).
    pub min_area: f64,
    /// Minimum volume of a 3D geometry (solid).
    pub min_volume: f64,
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

/// What running one check produced: the positions it flagged, empty when the
/// geometry passed. Doubles as the mutable buffer a `check_*` helper pushes into
/// and as the check's outcome before dependency gating. Each [`Geometry`]
/// pinpoints where a problem was found, typically a point leaf at the offending
/// coordinate.
///
/// This is a two-state value (empty / non-empty); it cannot express
/// [`Unvalidated`](ValidationResult::Unvalidated), which is a gating decision
/// owned by the driver ([`resolve`]), not something a leaf check can report.
#[derive(Serialize, Clone, Debug, PartialEq, Default)]
pub struct ValidationReport(pub Vec<Geometry>);

impl ValidationReport {
    /// Run a `check_*` helper into a fresh report and collect the positions it
    /// flagged.
    pub(crate) fn ran(fill: impl FnOnce(&mut ValidationReport)) -> Self {
        let mut report = ValidationReport::default();
        fill(&mut report);
        report
    }

    /// Whether any problems were recorded.
    #[inline]
    pub fn problem_recorded(&self) -> bool {
        !self.0.is_empty()
    }

    /// Record a problem at a position.
    #[inline]
    pub fn push(&mut self, position: Geometry) {
        self.0.push(position);
    }

    /// Reduce to a gated result: no positions →
    /// [`Success`](ValidationResult::Success), otherwise
    /// [`Failed`](ValidationResult::Failed).
    #[inline]
    fn into_result(self) -> ValidationResult {
        if self.problem_recorded() {
            ValidationResult::Failed(self.0)
        } else {
            ValidationResult::Success
        }
    }
}

/// Run each of a leaf's `applicable` checks, honoring
/// [dependencies](ValidationType::dependencies), and collect every check's
/// [`ValidationResult`]. `run_one` executes a single check assuming its
/// prerequisites passed; it is invoked at most once per check.
pub(crate) fn run_checks(
    applicable: &[ValidationType],
    mut run_one: impl FnMut(ValidationType) -> ValidationReport,
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
/// prerequisite did not end in [`Success`](ValidationResult::Success).
fn resolve(
    check: ValidationType,
    applicable: &HashSet<ValidationType>,
    results: &mut ValidationResults,
    run_one: &mut impl FnMut(ValidationType) -> ValidationReport,
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
/// has a single source of truth. Each check method defaults to a
/// `unimplemented!()` panic; a leaf overrides only the ones it supports.
macro_rules! validation_checks {
    ($($(#[$m:meta])* $method:ident => $variant:ident),+ $(,)?) => {
        /// Per-leaf validation: one method per [`ValidationType`], plus
        /// [`applicable_checks`](Validate::applicable_checks): the leaf's row of the
        /// [validation matrix](validate#default-validation-per-leaf-type).
        ///
        /// A leaf overrides `applicable_checks` and the handful of check methods it
        /// implements; every other method falls to the panicking default. The driver
        /// ([`validate`]) only ever calls a method listed in `applicable_checks`, so
        /// an inapplicable check's default body is never observed. This trait is the
        /// dispatched primitive; the free function [`validate`] owns the dependency
        /// gating and aggregate recursion.
        #[enum_dispatch::enum_dispatch]
        pub(crate) trait Validate {
            /// This leaf's row of the matrix: every check that applies to it.
            /// Defaults to none: the aggregates ([`Csg`](crate::csg::Csg), the
            /// collections) validate by recursing into their members, not by direct
            /// checks.
            fn applicable_checks(&self) -> &'static [ValidationType] {
                &[]
            }
            $(
                $(#[$m])*
                fn $method(&self, params: &ValidationParams) -> ValidationReport {
                    // Reached only when this check is listed in the leaf's
                    // `applicable_checks` but its detection is not yet written:
                    // a genuine `TODO`, made loud rather than silently skipped.
                    let _ = params;
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
                fn $method(&self, params: &ValidationParams) -> ValidationReport {
                    (**self).$method(params)
                }
            )+
        }

        /// Run a single check by routing a runtime [`ValidationType`] to the matching
        /// [`Validate`] method. Works on the enums (via `enum_dispatch`) and on a
        /// bare leaf such as a CSG operand's `&Solid`.
        fn dispatch<T: Validate + ?Sized>(
            leaf: &T,
            check: ValidationType,
            params: &ValidationParams,
        ) -> ValidationReport {
            match check {
                $( ValidationType::$variant => leaf.$method(params), )+
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
/// [applies to its leaf type](#default-validation-per-leaf-type), honoring
/// [dependencies](#check-dependencies).
///
/// Each applicable check maps to a [`ValidationResult`]; checks whose
/// prerequisites failed are
/// [`Unvalidated`](ValidationResult::Unvalidated). Aggregates (the collections,
/// [`Csg`](crate::csg::Csg), [`GeometryCollection`](crate::GeometryCollection))
/// recurse into their members and merge the per-check results with
/// [`merge_results`].
///
/// # Default validation per leaf type
///
/// Which per-coordinate / per-ring / whole-surface checks run for each leaf type,
/// one row per [`ValidationType`]. `✓` runs, `·` does not (not applicable).
/// Finiteness (the `Finite` row) always runs. Leaf columns fold the 2D/3D pairs:
/// `Point` = Point2D/3D, `LineStr` = LineString2D/3D, `Coll` = Collection2D/3D.
///
/// | Check ╲ Leaf            | Point | LineStr | Poly2D | Poly3D | PolyMesh2D | PolyMesh3D | TriMesh2D | TriMesh3D | Solid | Csg | PtCloud | Coll |
/// |-------------------------|:-----:|:-------:|:------:|:------:|:----------:|:----------:|:---------:|:---------:|:-----:|:---:|:-------:|:----:|
/// | Finite                  |   ✓   |    ✓    |   ✓    |   ✓    |     ✓      |     ✓      |     ✓     |     ✓     |   ✓   |  ✓  |    ✓    |  ✓   |
/// | TooFewPoints            |   ·   |    ✓    |   ✓    |   ✓    |     ✓      |     ✓      |     ·     |     ·     |   ✓   |  ✓  |    ·    |  ✓   |
/// | UnclosedRing            |   ·   |    ·    |   ✓    |   ✓    |     ✓      |     ✓      |     ·     |     ·     |   ✓   |  ✓  |    ·    |  ✓   |
/// | SelfIntersection        |   ·   |    ✓    |   ✓    |   ✓    |     ✓      |     ✓      |     ·     |     ·     |   ✓   |  ✓  |    ·    |  ✓   |
/// | InteriorRingContainment |   ·   |    ·    |   ✓    |   ✓    |     ✓      |     ✓      |     ·     |     ·     |   ·   |  ·  |    ·    |  ✓   |
/// | Degenerate              |   ·   |    ✓    |   ✓    |   ✓    |     ✓      |     ✓      |     ✓     |     ✓     |   ✓   |  ✓  |    ·    |  ✓   |
/// | Planarity               |   ·   |    ·    |   ·    |   ✓    |     ·      |     ·      |     ·     |     ·     |   ·   |  ·  |    ·    |  ✓   |
/// | DuplicatePoints         |   ·   |    ✓    |   ✓    |   ✓    |     ✓      |     ✓      |     ✓     |     ✓     |   ✓   |  ✓  |    ✓    |  ✓   |
/// | Orientation             |   ·   |    ·    |   ✓    |   ✓    |     ✓      |     ✓      |     ✓     |     ✓     |   ✓   |  ✓  |    ·    |  ✓   |
/// | Orientable              |   ·   |    ·    |   ·    |   ·    |     ·      |     ✓      |     ·     |     ✓     |   ✓   |  ✓  |    ·    |  ✓   |
/// | ShellManifold           |   ·   |    ·    |   ·    |   ·    |     ·      |     ·      |     ·     |     ·     |   ✓   |  ✓  |    ·    |  ✓   |
/// | ShellOrientation        |   ·   |    ·    |   ·    |   ·    |     ·      |     ·      |     ·     |     ·     |   ✓   |  ✓  |    ·    |  ✓   |
///
/// # Check dependencies
///
/// A check is only meaningful once the checks it depends on hold, so each
/// [`ValidationType`] lists its immediate prerequisites via
/// [`ValidationType::dependencies`] (transitive-close for the full set). A
/// runner should skip a check while any prerequisite fails. The relation is a
/// DAG (checked in the unit tests):
///
/// | Check                        | Immediate prerequisites                  |
/// |------------------------------|------------------------------------------|
/// | `Finite` (always on)         | (none) |
/// | `TooFewPoints`               | (none) |
/// | `UnclosedRing`               | `Finite` |
/// | `DuplicatePoints`            | `Finite` |
/// | `Degenerate`                 | `Finite` |
/// | `Planarity`                  | `Finite` |
/// | `SelfIntersection`           | `Finite`, `TooFewPoints`, `UnclosedRing` |
/// | `InteriorRingContainment`    | `Finite`, `SelfIntersection` |
/// | `Orientable`                 | (none) |
/// | `Orientation`                | `Finite`, `Orientable` |
/// | `ShellManifold`              | (none) |
/// | `ShellOrientation`           | `Orientation`, `ShellManifold` |
pub fn validate(geometry: &Geometry) -> ValidationResults {
    validate_with(geometry, &ValidationParams::default())
}

/// Like [`validate`], but with caller-supplied [`ValidationParams`] instead of the
/// defaults.
pub fn validate_with(geometry: &Geometry, params: &ValidationParams) -> ValidationResults {
    match geometry {
        // An absent geometry has nothing to validate.
        Geometry::None => ValidationResults::new(),
        Geometry::Euclidean2D(g) => validate_2d(g, params),
        Geometry::Euclidean3D(g) => validate_3d(g, params),
        Geometry::GeometryCollection(c) => {
            merge_members(c.members().iter().map(|m| validate_with(m, params)))
        }
    }
}

/// Run one leaf's applicable checks under dependency gating. The entry point for a
/// concrete leaf (and the per-leaf unit tests); [`validate`] routes enum variants
/// here after handling aggregates.
pub(crate) fn validate_leaf<T: Validate + ?Sized>(
    leaf: &T,
    params: &ValidationParams,
) -> ValidationResults {
    run_checks(leaf.applicable_checks(), |check| {
        dispatch(leaf, check, params)
    })
}

/// Resolve a single check for a leaf, running only its applicable prerequisites,
/// not the leaf's other checks. Lets the per-check unit tests exercise one check
/// without tripping an unrelated, still-`unimplemented!()` sibling on the same
/// leaf.
#[cfg(test)]
pub(crate) fn validate_one<T: Validate + ?Sized>(
    leaf: &T,
    check: ValidationType,
    params: &ValidationParams,
) -> ValidationResult {
    let applicable: HashSet<ValidationType> = leaf.applicable_checks().iter().copied().collect();
    let mut results = ValidationResults::new();
    resolve(check, &applicable, &mut results, &mut |c| {
        dispatch(leaf, c, params)
    })
}

fn validate_2d(g: &Euclidean2DGeometry, params: &ValidationParams) -> ValidationResults {
    match g {
        Euclidean2DGeometry::Collection(c) => {
            merge_members(c.members().iter().map(|m| validate_2d(m, params)))
        }
        leaf => validate_leaf(leaf, params),
    }
}

fn validate_3d(g: &Euclidean3DGeometry, params: &ValidationParams) -> ValidationResults {
    match g {
        Euclidean3DGeometry::Collection(c) => {
            merge_members(c.members().iter().map(|m| validate_3d(m, params)))
        }
        Euclidean3DGeometry::Csg(csg) => validate_csg(csg, params),
        leaf => validate_leaf(leaf, params),
    }
}

/// Recurse into a CSG tree, merging both operands' results; a `Csg` carries no
/// coordinates of its own.
fn validate_csg(csg: &Csg, params: &ValidationParams) -> ValidationResults {
    let (left, right) = match csg {
        Csg::Union(a, b) | Csg::Intersection(a, b) | Csg::Difference(a, b) => (a, b),
    };
    merge_members(
        [left, right]
            .into_iter()
            .map(|op| validate_operand(op, params)),
    )
}

fn validate_operand(operand: &ThreeDimensional, params: &ValidationParams) -> ValidationResults {
    match operand {
        ThreeDimensional::Solid(solid) => validate_leaf(solid.as_ref(), params),
        ThreeDimensional::Csg(csg) => validate_csg(csg, params),
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

/// A ring stored closed (first == last) with its trailing closing vertex
/// dropped, so the mandatory closure is not treated as a real element (e.g. a
/// duplicate point or an extra fan corner). Open rings — and anything too short
/// to be closed — pass through unchanged.
pub(crate) fn open_ring<T: PartialEq>(ring: &[T]) -> &[T] {
    match ring.split_last() {
        Some((last, head)) if !head.is_empty() && ring.first() == Some(last) => head,
        _ => ring,
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
        let zi = z.and_then(|zs| zs.get(i)).copied();
        let z_not_finite = zi.is_some_and(|v| !v.is_finite());
        if !c[0].is_finite() || !c[1].is_finite() || z_not_finite {
            // When the elevation is the offending component, report a 3D point
            // carrying it so the non-finite value is visible in the position;
            // otherwise the finite [x, y] alone would hide where the fault is.
            if z_not_finite {
                report.push(Geometry::Euclidean3D(Euclidean3DGeometry::Point(
                    Point3D::new(frame.clone(), [c[0], c[1], zi.unwrap()]),
                )));
            } else {
                report.push(Geometry::Euclidean2D(Euclidean2DGeometry::Point(
                    Point2D::new(frame.clone(), *c),
                )));
            }
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
            report.push(Geometry::Euclidean3D(Euclidean3DGeometry::Point(
                Point3D::new(frame.clone(), c),
            )));
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
        report.push(Geometry::Euclidean2D(Euclidean2DGeometry::LineString(
            LineString2D::from_coords(frame.clone(), coords.iter().copied()),
        )));
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
        report.push(Geometry::Euclidean3D(Euclidean3DGeometry::LineString(
            LineString3D::from_coords(frame.clone(), coords.iter().copied()),
        )));
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
        report.push(Geometry::Euclidean2D(Euclidean2DGeometry::LineString(
            LineString2D::from_coords(frame.clone(), ring.iter().copied()),
        )));
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
        report.push(Geometry::Euclidean3D(Euclidean3DGeometry::LineString(
            LineString3D::from_coords(frame.clone(), ring.iter().copied()),
        )));
    }
}

/// The bit pattern of a coordinate component, normalizing `-0.0` to `+0.0` so the
/// two hash and compare equal in the exact duplicate scan.
#[inline]
fn norm_bits(x: f64) -> u64 {
    (x + 0.0).to_bits()
}

/// A coordinate whose dimension selects the point-leaf `Geometry` that reports
/// it, letting [`check_duplicate_points`] stay generic over 2D/3D while still
/// pinpointing each failure at a point of the matching dimension.
pub(crate) trait DuplicateCoord: Copy {
    /// This coordinate as a point-leaf `Geometry` in `frame`.
    fn into_point(self, frame: &CoordinateFrame) -> Geometry;
}

impl DuplicateCoord for [f64; 2] {
    fn into_point(self, frame: &CoordinateFrame) -> Geometry {
        Geometry::Euclidean2D(Euclidean2DGeometry::Point(Point2D::new(
            frame.clone(),
            self,
        )))
    }
}

impl DuplicateCoord for [f64; 3] {
    fn into_point(self, frame: &CoordinateFrame) -> Geometry {
        Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(
            frame.clone(),
            self,
        )))
    }
}

/// Report a [`ValidationType::DuplicatePoints`] problem per coordinate that
/// coincides with an earlier one, positioned at the offending coordinate as a
/// point. Exact bit-equality when `tolerance` is `None`; otherwise two coords are
/// coincident when within `tolerance` distance.
///
/// # Precondition
///
/// Every coordinate must be finite. `DuplicatePoints` depends on
/// [`Finite`](ValidationType::Finite) (see
/// [`dependencies`](ValidationType::dependencies)), so the gated driver never
/// reaches this check until finiteness has passed, and this routine relies on
/// that rather than re-checking. A non-finite coordinate would corrupt
/// detection — [`norm_bits`] collides distinct NaNs into a false duplicate, and a
/// NaN poisons the k-d tree — so any caller outside the gated driver must uphold
/// it.
pub(crate) fn check_duplicate_points<const N: usize>(
    frame: &CoordinateFrame,
    coords: impl IntoIterator<Item = [f64; N]>,
    tolerance: Option<f64>,
    report: &mut ValidationReport,
) where
    [f64; N]: DuplicateCoord,
{
    let mut push = |c: [f64; N]| report.push(c.into_point(frame));
    match tolerance {
        None => {
            let mut seen = HashSet::new();
            for c in coords {
                let key: [u64; N] = c.map(norm_bits);
                if !seen.insert(key) {
                    push(c);
                }
            }
        }
        Some(t) => {
            let radius = t * t;
            let mut tree: KdTree<f64, N> = KdTree::new();
            let mut n: u64 = 0;
            for c in coords {
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
/// wrong way. Flow's convention is counter-clockwise: an exterior ring must be
/// counter-clockwise, a hole clockwise. A zero-area (degenerate / collinear) ring
/// has no meaningful winding and is left to the degeneracy check. The position is
/// the offending ring.
pub(crate) fn check_ring_orientation_2d(
    frame: &CoordinateFrame,
    ring: &[[f64; 2]],
    is_exterior: bool,
    report: &mut ValidationReport,
) {
    let area = signed_area_2d(ring);
    let wrong = if is_exterior { area < 0.0 } else { area > 0.0 };
    if wrong {
        report.push(Geometry::Euclidean2D(Euclidean2DGeometry::LineString(
            LineString2D::from_coords(frame.clone(), ring.iter().copied()),
        )));
    }
}

/// Right-hand-rule unit normal of a planar 3D ring, `None` if degenerate.
fn face_normal_3d(ring: &[[f64; 3]]) -> Option<[f64; 3]> {
    crate::ops::triangulation::normal(open_ring(ring))
}

/// Report `hole` when its winding agrees with a face's exterior normal (dot > 0)
/// instead of opposing it; a degenerate hole is skipped.
fn report_aligned_hole(
    frame: &CoordinateFrame,
    exterior_normal: [f64; 3],
    hole: &[[f64; 3]],
    report: &mut ValidationReport,
) {
    let Some(n) = face_normal_3d(hole) else {
        return;
    };
    let dot = exterior_normal[0] * n[0] + exterior_normal[1] * n[1] + exterior_normal[2] * n[2];
    if dot > 0.0 {
        report.push(Geometry::Euclidean3D(Euclidean3DGeometry::LineString(
            LineString3D::from_coords(frame.clone(), hole.iter().copied()),
        )));
    }
}

/// Report a [`ValidationType::Orientation`] problem for each hole of a planar 3D
/// face whose winding agrees with the exterior instead of opposing it. A 3D face
/// has no absolute winding, so orientation is relative: a valid hole's right-hand
/// normal opposes the exterior's (dot < 0). A ring with no normal (degenerate) is
/// left to the degeneracy check. The position is the offending hole ring.
pub(crate) fn check_face_orientation_3d<'a>(
    frame: &CoordinateFrame,
    exterior: &[[f64; 3]],
    interiors: impl IntoIterator<Item = &'a [[f64; 3]]>,
    report: &mut ValidationReport,
) {
    let Some(n_ext) = face_normal_3d(exterior) else {
        return;
    };
    for hole in interiors {
        report_aligned_hole(frame, n_ext, hole, report);
    }
}

/// Streaming form of [`check_face_orientation_3d`] for meshes: fed each ring in
/// face order (exterior first, then that face's holes), it checks each hole winds
/// opposite its face's exterior. Mirrors [`EdgeOrientation`]'s streaming shape.
pub(crate) struct FaceOrientation {
    exterior_normal: Option<[f64; 3]>,
}

impl FaceOrientation {
    pub(crate) fn new() -> Self {
        Self {
            exterior_normal: None,
        }
    }

    pub(crate) fn check_ring(
        &mut self,
        frame: &CoordinateFrame,
        coords: &[[f64; 3]],
        is_exterior: bool,
        report: &mut ValidationReport,
    ) {
        if is_exterior {
            self.exterior_normal = face_normal_3d(coords);
        } else if let Some(n_ext) = self.exterior_normal {
            report_aligned_hole(frame, n_ext, coords, report);
        }
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
    let mut checker = EdgeOrientation::new();
    for ring in rings {
        checker.check_ring(frame, vertices, ring.as_ref(), report);
    }
}

/// The directed edges seen so far while checking that a set of face rings winds
/// coherently: a running accumulator so rings can be fed one at a time (e.g.
/// streamed from a decoder into a reused buffer) without collecting them all.
pub(crate) struct EdgeOrientation {
    /// Directed edges `(from, to)` already traversed by an earlier ring.
    seen: HashSet<(u32, u32)>,
}

impl EdgeOrientation {
    pub(crate) fn new() -> Self {
        Self {
            seen: HashSet::new(),
        }
    }

    /// Fold one face ring into the accumulator, reporting it when it traverses a
    /// shared edge in the same direction as an earlier ring (an orientation
    /// conflict). Self-loop edges (`a == b`) are ignored.
    pub(crate) fn check_ring(
        &mut self,
        frame: &CoordinateFrame,
        vertices: &[[f64; 3]],
        ring: &[u32],
        report: &mut ValidationReport,
    ) {
        let n = ring.len();
        if n < 2 {
            return;
        }
        let mut conflict = false;
        for i in 0..n {
            let (a, b) = (ring[i], ring[(i + 1) % n]);
            if a == b {
                continue;
            }
            if !self.seen.insert((a, b)) {
                conflict = true;
            }
        }
        if conflict {
            let coords: Vec<[f64; 3]> = ring.iter().map(|&i| vertices[i as usize]).collect();
            report.push(Geometry::Euclidean3D(Euclidean3DGeometry::LineString(
                LineString3D::from_coords(frame.clone(), coords),
            )));
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
    /// An empty topology, to be populated with [`add_face`](Self::add_face).
    pub(crate) fn new() -> Self {
        Self {
            n_faces: 0,
            edges: HashMap::new(),
        }
    }

    /// Add one face from its vertex-index ring (closure optional; the last vertex
    /// wraps to the first). Self-loop edges (`a == b`) are skipped. Lets faces be
    /// fed one at a time (e.g. streamed from a decoder into a reused buffer).
    pub(crate) fn add_face(&mut self, ring: &[u32]) {
        let f = self.n_faces;
        self.n_faces += 1;
        let n = ring.len();
        if n < 2 {
            return;
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
            self.edges.entry(key).or_default().push((f, forward));
        }
    }

    /// Build from one vertex-index ring per face (closure optional; the last
    /// vertex wraps to the first). Self-loop edges (`a == b`) are skipped.
    pub(crate) fn from_faces<R: AsRef<[u32]>>(faces: impl IntoIterator<Item = R>) -> Self {
        let mut topology = Self::new();
        for ring in faces {
            topology.add_face(ring.as_ref());
        }
        topology
    }

    /// Whether every edge is shared by exactly two faces: a closed 2-manifold
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

    /// Default thresholds for the tests that don't care about tuning.
    fn params() -> ValidationParams {
        ValidationParams::default()
    }

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
            validate_leaf(&p, &params())[&ValidationType::Finite],
            ValidationResult::Success
        );
    }

    #[test]
    fn non_finite_point_reports_not_finite() {
        let p = Point3D::new(CoordinateFrame::Euclidean, [1.0, f64::NAN, 3.0]);
        let positions = failures(&validate_leaf(&p, &params()), ValidationType::Finite);
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
        let positions = failures(&validate_leaf(&ls, &params()), ValidationType::Finite);
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
        let positions = one_failure(validate_one(&ls, ValidationType::TooFewPoints, &params()));
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
            validate_one(&ls, ValidationType::TooFewPoints, &params()),
            ValidationResult::Success
        );
    }

    #[test]
    fn duplicate_points_exact_flags_repeated_coordinate() {
        let ls = LineString3D::from_coords(
            CoordinateFrame::Euclidean,
            [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 0.0]],
        );
        let positions = one_failure(validate_one(
            &ls,
            ValidationType::DuplicatePoints,
            &params(),
        ));
        assert_eq!(positions.len(), 1);
        assert_eq!(offending_point(&positions[0]), [0.0, 0.0, 0.0]);
    }

    #[test]
    fn duplicate_tolerance_flags_near_coincident_points() {
        // Two vertices 0.001 apart. With the default (exact) params they are
        // distinct; a `duplicate_tolerance` of 0.01 makes them coincide.
        let ls = LineString3D::from_coords(
            CoordinateFrame::Euclidean,
            [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.001, 0.0, 0.0]],
        );
        assert_eq!(
            validate_one(&ls, ValidationType::DuplicatePoints, &params()),
            ValidationResult::Success
        );
        let lenient = ValidationParams {
            duplicate_tolerance: Some(0.01),
            ..Default::default()
        };
        let positions = one_failure(validate_one(&ls, ValidationType::DuplicatePoints, &lenient));
        assert_eq!(positions.len(), 1);
        assert_eq!(offending_point(&positions[0]), [0.001, 0.0, 0.0]);
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
        let keys: Vec<_> = validate_leaf(
            &Point3D::new(CoordinateFrame::Euclidean, [0.0; 3]),
            &params(),
        )
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
        let _ = validate_one(&ls, ValidationType::SelfIntersection, &params());
    }

    #[test]
    fn failed_prerequisite_leaves_dependents_unvalidated() {
        // A non-finite coordinate fails `Finite`, so `DuplicatePoints` (which
        // depends on it) cannot run.
        let ls = LineString3D::from_coords(
            CoordinateFrame::Euclidean,
            [[0.0, 0.0, 0.0], [f64::NAN, 0.0, 0.0]],
        );
        let results = validate_leaf(&ls, &params());
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
