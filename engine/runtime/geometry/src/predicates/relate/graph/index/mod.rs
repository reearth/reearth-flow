//! Noding support: finding the intersections among the graph's edges.
//!
//! The rstar-backed edge-set sweep is exposed as free functions, and
//! [`SegmentIntersector`] calls the
//! [`kernel`](crate::predicates::kernel) directly.

mod rstar_edge_set_intersector;
mod segment_intersector;

pub(crate) use rstar_edge_set_intersector::{
    compute_intersections_between_sets, compute_intersections_within_set,
};
pub(crate) use segment_intersector::SegmentIntersector;
