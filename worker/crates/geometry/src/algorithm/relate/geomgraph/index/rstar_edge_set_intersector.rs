use super::super::Edge;
use super::{EdgeSetIntersector, SegmentIntersector};
use crate::algorithm::GeoFloat;
use crate::types::coordinate::Coordinate;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use parking_lot::RwLock;
use rstar::RTree;

pub(crate) struct RstarEdgeSetIntersector;

impl RstarEdgeSetIntersector {
    pub fn new() -> Self {
        RstarEdgeSetIntersector
    }
}

struct Segment<'a, T: GeoFloat + rstar::RTreeNum, Z: GeoFloat + rstar::RTreeNum> {
    i: usize,
    edge: &'a RwLock<Edge<T, Z>>,
    envelope: rstar::AABB<Coordinate<T, Z>>,
}

impl<'a, T, Z> Segment<'a, T, Z>
where
    T: GeoFloat + rstar::RTreeNum,
    Z: GeoFloat + rstar::RTreeNum,
{
    fn new(i: usize, edge: &'a RwLock<Edge<T, Z>>) -> Self {
        use rstar::RTreeObject;
        let p1 = edge.read().coords()[i];
        let p2 = edge.read().coords()[i + 1];
        Self {
            i,
            edge,
            envelope: rstar::AABB::from_corners(p1, p2),
        }
    }
}

impl<'a, T, Z> rstar::RTreeObject for Segment<'a, T, Z>
where
    T: GeoFloat + rstar::RTreeNum,
    Z: GeoFloat + rstar::RTreeNum,
{
    type Envelope = rstar::AABB<Coordinate<T, Z>>;

    fn envelope(&self) -> Self::Envelope {
        self.envelope
    }
}

impl<T, Z> EdgeSetIntersector<T, Z> for RstarEdgeSetIntersector
where
    T: GeoFloat + rstar::RTreeNum,
    Z: GeoFloat + rstar::RTreeNum,
{
    fn compute_intersections_within_set(
        &mut self,
        edges: &[Arc<RwLock<Edge<T, Z>>>],
        check_for_self_intersecting_edges: bool,
        segment_intersector: &mut SegmentIntersector<T, Z>,
    ) {
        let segments: Vec<Segment<T, Z>> = edges
            .iter()
            .flat_map(|edge| {
                let start_of_final_segment: usize = edge.read().coords().len() - 1;
                (0..start_of_final_segment).map(|segment_i| Segment::new(segment_i, edge))
            })
            .collect();
        let tree = RTree::bulk_load(segments);

        for (edge0, edge1) in tree.intersection_candidates_with_other_tree(&tree) {
            if check_for_self_intersecting_edges || edge0.edge.data_ptr() != edge1.edge.data_ptr() {
                segment_intersector.add_intersections(edge0.edge, edge0.i, edge1.edge, edge1.i);
            }
        }
    }

    fn compute_intersections_between_sets(
        &mut self,
        edges0: &[Arc<RwLock<Edge<T, Z>>>],
        edges1: &[Arc<RwLock<Edge<T, Z>>>],
        segment_intersector: &mut SegmentIntersector<T, Z>,
    ) {
        let segments0: Vec<Segment<T, Z>> = edges0
            .iter()
            .flat_map(|edge| {
                let start_of_final_segment: usize = edge.read().coords().len() - 1;
                (0..start_of_final_segment).map(|segment_i| Segment::new(segment_i, edge))
            })
            .collect();
        let tree_0 = RTree::bulk_load(segments0);

        let segments1: Vec<Segment<T, Z>> = edges1
            .iter()
            .flat_map(|edge| {
                let start_of_final_segment: usize = edge.read().coords().len() - 1;
                (0..start_of_final_segment).map(|segment_i| Segment::new(segment_i, edge))
            })
            .collect();
        let tree_1 = RTree::bulk_load(segments1);

        for (edge0, edge1) in tree_0.intersection_candidates_with_other_tree(&tree_1) {
            segment_intersector.add_intersections(edge0.edge, edge0.i, edge1.edge, edge1.i);
        }
    }
}
