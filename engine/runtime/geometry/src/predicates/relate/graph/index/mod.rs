//! Noding support: finding the intersections among the graph's edges.
//!
//! The legacy `EdgeSetIntersector` trait and its `Simple` implementation are
//! gone — the rstar-backed edge-set sweep is the only implementation, exposed
//! as free functions — and [`SegmentIntersector`] calls the phase-1
//! [`kernel`](crate::predicates::kernel) directly instead of going through a
//! boxed `LineIntersector`.

mod rstar_edge_set_intersector;
mod segment_intersector;

pub(crate) use rstar_edge_set_intersector::{
    compute_intersections_between_sets, compute_intersections_within_set,
};
pub(crate) use segment_intersector::SegmentIntersector;
