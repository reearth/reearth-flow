//! DE-9IM [`relate`] for the geometry model.
//!
//! Operands are flattened into [`Leaf2D`](super::view::Leaf2D) views wrapped in
//! a [`RelateOperand`] (see `operand`). Mesh leaves contribute their
//! **union-boundary rings** (see `boundary`) instead of raw faces, so faces
//! sharing edges relate as one area.
//!
//! `relate` does not support operands whose own members have overlapping areal
//! interiors **or share boundary edges** (an invalid mesh, or a collection of
//! overlapping or edge-adjacent polygons, invalid MultiPolygon topology) and
//! may mislabel them (a shared edge between two polygon members stays
//! *boundary* even though it is interior to the union) or panic on a
//! `debug_assert` in debug builds (a "side location conflict"). Mesh leaves are
//! exempt: their faces dissolve via the `boundary` pre-pass before graph
//! construction. The [`contains`](super::contains()) /
//! [`covers`](super::covers()) / [`intersects`](super::intersects()) fast paths
//! are exact point-set-union predicates even on such collections; prefer them
//! when the full matrix is not needed.

pub(crate) mod boundary;
mod edge_end_builder;
mod graph;
pub mod intersection_matrix;
pub(crate) mod operand;
mod relate_operation;
#[cfg(test)]
mod tests;

pub use intersection_matrix::{Dimensions, IntersectionMatrix};

use crate::predicates::view::require_common_frame_leaves;
use crate::{Euclidean2DGeometry, Geometry};

use super::Result;
use operand::RelateOperand;
use relate_operation::RelateOperation;

/// The DE-9IM intersection matrix of `a` against `b`.
///
/// Operands must share one coordinate frame
/// ([`MixedFrames`](super::PredicateError::MixedFrames) otherwise) and both be
/// 2D: a 2D × 3D pair is
/// [`CrossDimension`](super::PredicateError::CrossDimension), a 3D × 3D pair
/// [`UnsupportedPair`](super::PredicateError::UnsupportedPair). `Geometry::None`
/// and empty collections relate as the empty geometry (every predicate against
/// them is false except `is_disjoint`).
pub fn relate(a: &Geometry, b: &Geometry) -> Result<IntersectionMatrix> {
    let (a_leaves, b_leaves) = crate::predicates::flatten_2d_pair(a, b)?;
    require_common_frame_leaves(&a_leaves, &b_leaves)?;
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
    require_common_frame_leaves(&a_leaves, &b_leaves)?;
    let a = RelateOperand::new(a_leaves);
    let b = RelateOperand::new(b_leaves);
    Ok(RelateOperation::new(&a, &b).compute_intersection_matrix())
}
