//! 2D boolean overlay and linear clipping for the geometry model.
//!
//! The *constructed* binary operations. They share the
//! [`predicates`](crate::predicates) substrate (flattened [`Leaf2D`] views, the
//! frame and dimension policy, [`PredicateError`]) but return geometry instead
//! of answering a question:
//!
//! - [`overlay()`] and its [`union()`] / [`intersection()`] / [`difference()`]
//!   / [`xor()`] shorthands: areal boolean overlay, backed by the `i_overlay`
//!   pure-Rust backend.
//! - [`clip()`]: the portion of a set of polylines inside (or, inverted,
//!   outside) an areal geometry.
//! - [`segment_intersections()`]: the pairwise segment × segment
//!   intersections between two polyline sets.
//!
//! The operand policy is the predicates': both operands in one coordinate
//! frame ([`MixedFrames`](PredicateError::MixedFrames) otherwise, reprojection
//! is the caller's step), a 2D × 3D pair is
//! [`CrossDimension`](PredicateError::CrossDimension), a purely 3D pair
//! [`UnsupportedPair`](PredicateError::UnsupportedPair). Collections flatten to
//! their leaves; `Geometry::None` and empty collections are the empty geometry.
//! Beyond that, each operation constrains the leaf kinds it accepts: an areal
//! operand takes `Polygon`, `PolygonMesh`, and `TriangularMesh` leaves, a
//! polyline operand takes `LineString` leaves, and any other leaf is an
//! [`UnsupportedPair`](PredicateError::UnsupportedPair) naming it.
//!
//! Semantics and caveats:
//!
//! - An operand is the point-set **union** of its leaves: the non-zero fill
//!   rule dissolves overlapping or edge-adjacent members exactly, and mesh
//!   leaves are pre-dissolved to their union-boundary rings (the
//!   [`relate`](crate::predicates::relate()) boundary pre-pass) so shared face
//!   edges cancel before the backend ever snaps a coordinate. Unlike
//!   [`relate`](crate::predicates::relate()), such operands are *not* a
//!   limitation here.
//! - Valid ring winding is assumed (exteriors CCW, holes CW, Flow's
//!   convention, checked by `Validate`): under the non-zero rule a mis-wound
//!   hole reads as filled area. Results follow the same convention.
//! - Constructed output is **not exact**: `i_overlay` snaps input to an
//!   adaptive integer grid, so vertices can move by a relative epsilon,
//!   degenerate slivers are dropped, collinear vertices are removed, and
//!   chained overlays can drift. Segment intersection points are f64-rounded
//!   robust-kernel constructions (no grid snap).
//! - Output is pure 2D: any per-vertex elevation on the inputs is ignored and
//!   dropped, and appearance does not propagate.

mod segments;
mod shapes;
#[cfg(test)]
mod tests;

use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::float::clip::FloatClip;
use i_overlay::float::single::SingleFloatOverlay;
use i_overlay::string::clip::ClipRule;

use crate::coordinate::CoordinateFrame;
use crate::line_string::LineString2D;
use crate::polygon::Polygon2D;
use crate::predicates::view::{flatten_2d, require_common_frame_leaves, Leaf2D};
use crate::predicates::{flatten_2d_pair, PredicateError, Result};
use crate::{Euclidean2DGeometry, Geometry};

pub use crate::predicates::kernel::SegmentIntersection;

/// The boolean overlay operation to apply.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverlayOp {
    /// Points in `a`, `b`, or both.
    Union,
    /// Points in both `a` and `b`.
    Intersection,
    /// Points in `a` but not in `b`.
    Difference,
    /// Points in exactly one of `a` and `b`.
    Xor,
}

impl From<OverlayOp> for OverlayRule {
    fn from(op: OverlayOp) -> Self {
        match op {
            OverlayOp::Union => OverlayRule::Union,
            OverlayOp::Intersection => OverlayRule::Intersect,
            OverlayOp::Difference => OverlayRule::Difference,
            OverlayOp::Xor => OverlayRule::Xor,
        }
    }
}

/// The boolean overlay `a <op> b` of two areal geometries, as disjoint
/// polygons in the operands' common frame (empty when the result is empty).
pub fn overlay(a: &Geometry, b: &Geometry, op: OverlayOp) -> Result<Vec<Polygon2D>> {
    let (a_leaves, b_leaves) = flatten_2d_pair(a, b)?;
    overlay_leaves(&a_leaves, &b_leaves, op)
}

/// [`overlay`] with [`OverlayOp::Union`]: points in `a`, `b`, or both.
pub fn union(a: &Geometry, b: &Geometry) -> Result<Vec<Polygon2D>> {
    overlay(a, b, OverlayOp::Union)
}

/// [`overlay`] with [`OverlayOp::Intersection`]: points in both `a` and `b`.
pub fn intersection(a: &Geometry, b: &Geometry) -> Result<Vec<Polygon2D>> {
    overlay(a, b, OverlayOp::Intersection)
}

/// [`overlay`] with [`OverlayOp::Difference`]: points in `a` but not in `b`.
pub fn difference(a: &Geometry, b: &Geometry) -> Result<Vec<Polygon2D>> {
    overlay(a, b, OverlayOp::Difference)
}

/// [`overlay`] with [`OverlayOp::Xor`]: points in exactly one of `a` and `b`.
pub fn xor(a: &Geometry, b: &Geometry) -> Result<Vec<Polygon2D>> {
    overlay(a, b, OverlayOp::Xor)
}

/// [`overlay`] over two 2D geometries.
pub fn overlay_2d(
    a: &Euclidean2DGeometry,
    b: &Euclidean2DGeometry,
    op: OverlayOp,
) -> Result<Vec<Polygon2D>> {
    let mut a_leaves = Vec::new();
    flatten_2d(a, &mut a_leaves);
    let mut b_leaves = Vec::new();
    flatten_2d(b, &mut b_leaves);
    overlay_leaves(&a_leaves, &b_leaves, op)
}

/// The portion of the polylines of `lines` inside the areal geometry `area`,
/// or, with `invert`, the portion outside it. Points exactly on `area`'s
/// boundary count as inside either way.
pub fn clip(lines: &Geometry, area: &Geometry, invert: bool) -> Result<Vec<LineString2D>> {
    let (line_leaves, area_leaves) = flatten_2d_pair(lines, area)?;
    clip_leaves(&line_leaves, &area_leaves, invert)
}

/// [`clip`] over two 2D geometries.
pub fn clip_2d(
    lines: &Euclidean2DGeometry,
    area: &Euclidean2DGeometry,
    invert: bool,
) -> Result<Vec<LineString2D>> {
    let mut line_leaves = Vec::new();
    flatten_2d(lines, &mut line_leaves);
    let mut area_leaves = Vec::new();
    flatten_2d(area, &mut area_leaves);
    clip_leaves(&line_leaves, &area_leaves, invert)
}

/// Every segment × segment intersection between the polylines of `a` and the
/// polylines of `b`: proper crossings, endpoint touches, and collinear
/// overlaps, deduplicated and in a deterministic order. Segments *within* one
/// operand are never paired with each other, so consecutive segments of one
/// polyline do not report their shared vertex.
pub fn segment_intersections(a: &Geometry, b: &Geometry) -> Result<Vec<SegmentIntersection>> {
    let (a_leaves, b_leaves) = flatten_2d_pair(a, b)?;
    segment_intersections_leaves(&a_leaves, &b_leaves)
}

/// [`segment_intersections`] over two 2D geometries.
pub fn segment_intersections_2d(
    a: &Euclidean2DGeometry,
    b: &Euclidean2DGeometry,
) -> Result<Vec<SegmentIntersection>> {
    let mut a_leaves = Vec::new();
    flatten_2d(a, &mut a_leaves);
    let mut b_leaves = Vec::new();
    flatten_2d(b, &mut b_leaves);
    segment_intersections_leaves(&a_leaves, &b_leaves)
}

// --- leaf-level implementations ------------------------------------------------

fn overlay_leaves(a: &[Leaf2D<'_>], b: &[Leaf2D<'_>], op: OverlayOp) -> Result<Vec<Polygon2D>> {
    require_common_frame_leaves(a, b)?;
    let unsupported = || unsupported_pair(a, b, is_areal);
    let subject = shapes::areal_shapes(a).map_err(|_| unsupported())?;
    let clip = shapes::areal_shapes(b).map_err(|_| unsupported())?;
    let Some(frame) = common_frame(a, b) else {
        return Ok(Vec::new());
    };
    let result = subject.overlay(&clip, op.into(), FillRule::NonZero);
    Ok(shapes::shapes_to_polygons(result, frame))
}

fn clip_leaves(
    lines: &[Leaf2D<'_>],
    area: &[Leaf2D<'_>],
    invert: bool,
) -> Result<Vec<LineString2D>> {
    require_common_frame_leaves(lines, area)?;
    let unsupported = || PredicateError::UnsupportedPair {
        left: operand_name(lines, is_line),
        right: operand_name(area, is_areal),
    };
    let subject = shapes::line_paths(lines).map_err(|_| unsupported())?;
    let clip = shapes::areal_shapes(area).map_err(|_| unsupported())?;
    let Some(frame) = common_frame(lines, area) else {
        return Ok(Vec::new());
    };
    // An empty clip area contains nothing: the inside portion is empty and the
    // outside portion is the polylines verbatim.
    if clip.is_empty() {
        let outside = if invert { subject } else { Vec::new() };
        return Ok(shapes::paths_to_line_strings(outside, frame));
    }
    let clip_rule = ClipRule {
        invert,
        boundary_included: true,
    };
    let result = subject.clip_by(&clip, FillRule::NonZero, clip_rule);
    Ok(shapes::paths_to_line_strings(result, frame))
}

fn segment_intersections_leaves(
    a: &[Leaf2D<'_>],
    b: &[Leaf2D<'_>],
) -> Result<Vec<SegmentIntersection>> {
    require_common_frame_leaves(a, b)?;
    let unsupported = || unsupported_pair(a, b, is_line);
    let a_segments = segments::leaf_segments(a).map_err(|_| unsupported())?;
    let b_segments = segments::leaf_segments(b).map_err(|_| unsupported())?;
    Ok(segments::intersections(a_segments, b_segments))
}

// --- operand policy helpers ----------------------------------------------------

fn is_areal(leaf: &Leaf2D<'_>) -> bool {
    matches!(
        leaf,
        Leaf2D::Polygon(_) | Leaf2D::PolygonMesh(_) | Leaf2D::TriangularMesh(_)
    )
}

fn is_line(leaf: &Leaf2D<'_>) -> bool {
    matches!(leaf, Leaf2D::Line(_))
}

/// The concrete 2D leaf name, for `UnsupportedPair` diagnostics.
fn leaf_type_name(leaf: &Leaf2D<'_>) -> &'static str {
    match leaf {
        Leaf2D::Point(_) => "Point2D",
        Leaf2D::Line(_) => "LineString2D",
        Leaf2D::Polygon(_) => "Polygon2D",
        Leaf2D::PolygonMesh(_) => "PolygonMesh2D",
        Leaf2D::TriangularMesh(_) => "TriangularMesh2D",
    }
}

/// Describe an operand for an `UnsupportedPair` diagnostic: its first leaf
/// that fails `ok` if any, else its first leaf, else "empty geometry".
fn operand_name(leaves: &[Leaf2D<'_>], ok: fn(&Leaf2D<'_>) -> bool) -> &'static str {
    leaves
        .iter()
        .find(|leaf| !ok(leaf))
        .or_else(|| leaves.first())
        .map(leaf_type_name)
        .unwrap_or("empty geometry")
}

/// The `UnsupportedPair` for an operation whose two operands both require
/// `ok` leaves.
fn unsupported_pair(
    a: &[Leaf2D<'_>],
    b: &[Leaf2D<'_>],
    ok: fn(&Leaf2D<'_>) -> bool,
) -> PredicateError {
    PredicateError::UnsupportedPair {
        left: operand_name(a, ok),
        right: operand_name(b, ok),
    }
}

/// The operands' shared frame (the first leaf's, all being equal after
/// [`require_common_frame_leaves`]), or `None` when both are empty.
fn common_frame<'l>(a: &[Leaf2D<'l>], b: &[Leaf2D<'l>]) -> Option<&'l CoordinateFrame> {
    a.first().or_else(|| b.first()).map(Leaf2D::frame)
}
