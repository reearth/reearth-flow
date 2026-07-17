//! The geometry graph underlying [`relate`](super): nodes and edges labeled
//! with their topological positions relative to both operands.
//!
//! Follows the structure of JTS 1.18 `relate/geomgraph/`, using the
//! [`kernel`](crate::predicates::kernel) as the robust line intersector.

pub(crate) use super::intersection_matrix::IntersectionMatrix;
pub(crate) use edge::Edge;
pub(crate) use edge_end::{EdgeEnd, EdgeEndKey};
pub(crate) use edge_end_bundle::{EdgeEndBundle, LabeledEdgeEndBundle};
pub(crate) use edge_end_bundle_star::{EdgeEndBundleStar, LabeledEdgeEndBundleStar};
pub(crate) use edge_intersection::EdgeIntersection;
pub(crate) use geometry_graph::GeometryGraph;
pub(crate) use label::Label;
pub(crate) use node::CoordNode;
use planar_graph::PlanarGraph;
pub(crate) use quadrant::Quadrant;
use topology_position::TopologyPosition;

mod edge;
mod edge_end;
mod edge_end_bundle;
mod edge_end_bundle_star;
mod edge_intersection;
mod geometry_graph;
pub(crate) mod index;
mod label;
mod node;
pub(crate) mod node_map;
mod planar_graph;
mod quadrant;
mod topology_position;

/// Position relative to a point
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Direction {
    On,
    Left,
    Right,
}

/// The SFS "Mod-2 Rule": a point shared by an odd number of line endpoints is
/// on the boundary, an even number interior.
pub(crate) fn determine_boundary(boundary_count: usize) -> crate::predicates::kernel::CoordPos {
    if boundary_count % 2 == 1 {
        crate::predicates::kernel::CoordPos::OnBoundary
    } else {
        crate::predicates::kernel::CoordPos::Inside
    }
}
