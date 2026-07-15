//! DE-9IM [`relate`] for the new geometry model.
//!
//! A port of the in-tree JTS geomgraph (`algorithm/relate/`, itself a
//! georust/geo port of JTS 1.18) onto the flat `[f64; 2]` layout and the
//! [`view`](super::view) layer, per phase 3 of the predicates plan. The legacy
//! copy stays in-tree as the differential-testing oracle until legacy removal.
//!
//! Structural differences from the legacy copy:
//!
//! - the `GeometryCow` input layer is replaced by `RelateOperand` (see
//!   `operand`) over flattened [`Leaf2D`] views;
//! - mesh leaves contribute their **union-boundary rings** (see `boundary`)
//!   instead of raw faces, so faces sharing edges relate as one area;
//! - the robust line intersector is the shared phase-1
//!   [`kernel`](super::kernel);
//! - graphs are call-local, so edges are `Rc<RefCell<..>>`, not `Arc<RwLock<..>>`.
//!
//! Like JTS, `relate` does not support operands whose own members have
//! overlapping areal interiors **or share boundary edges** (an invalid mesh,
//! or a collection of overlapping or edge-adjacent polygons — invalid
//! MultiPolygon topology) and may mislabel them: e.g. a shared edge between
//! two polygon members stays *boundary* even though it is interior to the
//! union. Mesh leaves are exempt — their faces dissolve via the `boundary`
//! pre-pass before graph construction. The phase-2 [`contains`](super::contains()) /
//! [`covers`](super::covers()) / [`intersects`](super::intersects()) fast
//! paths are exact point-set-union predicates even on such collections;
//! prefer them when the full matrix is not needed.

pub(crate) mod boundary;
mod edge_end_builder;
mod graph;
pub mod intersection_matrix;
pub(crate) mod operand;
mod relate_operation;
#[cfg(test)]
mod tests;

pub use intersection_matrix::{Dimensions, IntersectionMatrix};

use crate::predicates::contains::flatten_geometry;
use crate::predicates::view::Leaf2D;
use crate::{Euclidean2DGeometry, Geometry};

use super::{PredicateError, Result};
use operand::RelateOperand;
use relate_operation::RelateOperation;

/// The DE-9IM intersection matrix of `a` against `b`.
///
/// Operands must share one coordinate frame
/// ([`MixedFrames`](PredicateError::MixedFrames) otherwise) and both be 2D —
/// a 2D × 3D pair is
/// [`CrossDimension`](PredicateError::CrossDimension), a 3D × 3D pair
/// [`UnsupportedPair`](PredicateError::UnsupportedPair) until the 3D phases
/// land. `Geometry::None` and empty collections relate as the empty geometry
/// (every predicate against them is false except `is_disjoint`).
pub fn relate(a: &Geometry, b: &Geometry) -> Result<IntersectionMatrix> {
    let (a_leaves, a_3d) = flatten_geometry(a);
    let (b_leaves, b_3d) = flatten_geometry(b);
    match (a_3d, b_3d) {
        (None, None) => {}
        (Some(left), Some(right)) if a_leaves.is_empty() && b_leaves.is_empty() => {
            return Err(PredicateError::UnsupportedPair { left, right })
        }
        _ => return Err(PredicateError::CrossDimension),
    }
    require_common_frame(&a_leaves, &b_leaves)?;
    let a = RelateOperand::new(a_leaves);
    let b = RelateOperand::new(b_leaves);
    Ok(RelateOperation::new(&a, &b).compute_intersection_matrix())
}

/// [`relate`] over two 2D geometries.
pub fn relate_2d(a: &Euclidean2DGeometry, b: &Euclidean2DGeometry) -> Result<IntersectionMatrix> {
    let mut a_leaves = Vec::new();
    crate::predicates::view::flatten_2d(a, &mut a_leaves);
    let mut b_leaves = Vec::new();
    crate::predicates::view::flatten_2d(b, &mut b_leaves);
    require_common_frame(&a_leaves, &b_leaves)?;
    let a = RelateOperand::new(a_leaves);
    let b = RelateOperand::new(b_leaves);
    Ok(RelateOperation::new(&a, &b).compute_intersection_matrix())
}

/// Require every leaf across both operands to share one coordinate frame.
fn require_common_frame(a: &[Leaf2D<'_>], b: &[Leaf2D<'_>]) -> Result<()> {
    let mut frames = a.iter().chain(b.iter()).map(|leaf| leaf.frame());
    let Some(first) = frames.next() else {
        return Ok(());
    };
    for frame in frames {
        super::require_same_frame(first, frame)?;
    }
    Ok(())
}
