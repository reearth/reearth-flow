use std::cell::RefCell;
use std::rc::Rc;

use rstar::RTree;

use super::super::Edge;
use super::SegmentIntersector;

struct Segment<'a> {
    i: usize,
    edge: &'a Rc<RefCell<Edge>>,
    envelope: rstar::AABB<[f64; 2]>,
}

impl<'a> Segment<'a> {
    fn new(i: usize, edge: &'a Rc<RefCell<Edge>>) -> Self {
        let p1 = edge.borrow().coords()[i];
        let p2 = edge.borrow().coords()[i + 1];
        Self {
            i,
            edge,
            envelope: rstar::AABB::from_corners(p1, p2),
        }
    }
}

impl rstar::RTreeObject for Segment<'_> {
    type Envelope = rstar::AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        self.envelope
    }
}

fn to_segments(edges: &[Rc<RefCell<Edge>>]) -> Vec<Segment<'_>> {
    edges
        .iter()
        .flat_map(|edge| {
            let start_of_final_segment: usize = edge.borrow().coords().len() - 1;
            (0..start_of_final_segment).map(move |segment_i| Segment::new(segment_i, edge))
        })
        .collect()
}

/// Compute all intersections between the edges within a set, recording those intersections on
/// the intersecting edges.
///
/// `edges`: the set of edges to check. Mutated to record any intersections.
/// `check_for_self_intersecting_edges`: if false, an edge is not checked for intersections with
/// itself.
/// `segment_intersector`: the SegmentIntersector to use
pub(crate) fn compute_intersections_within_set(
    edges: &[Rc<RefCell<Edge>>],
    check_for_self_intersecting_edges: bool,
    segment_intersector: &mut SegmentIntersector,
) {
    let tree = RTree::bulk_load(to_segments(edges));

    for (edge0, edge1) in tree.intersection_candidates_with_other_tree(&tree) {
        if check_for_self_intersecting_edges || !Rc::ptr_eq(edge0.edge, edge1.edge) {
            segment_intersector.add_intersections(edge0.edge, edge0.i, edge1.edge, edge1.i);
        }
    }
}

/// Compute all intersections between two sets of edges, recording those intersections on
/// the intersecting edges.
pub(crate) fn compute_intersections_between_sets(
    edges0: &[Rc<RefCell<Edge>>],
    edges1: &[Rc<RefCell<Edge>>],
    segment_intersector: &mut SegmentIntersector,
) {
    let tree_0 = RTree::bulk_load(to_segments(edges0));
    let tree_1 = RTree::bulk_load(to_segments(edges1));

    for (edge0, edge1) in tree_0.intersection_candidates_with_other_tree(&tree_1) {
        segment_intersector.add_intersections(edge0.edge, edge0.i, edge1.edge, edge1.i);
    }
}
