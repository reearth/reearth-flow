use parking_lot::RwLock;

use crate::algorithm::GeoFloat;

use super::super::Edge;
use super::{EdgeSetIntersector, SegmentIntersector};

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

pub(crate) struct SimpleEdgeSetIntersector;

impl SimpleEdgeSetIntersector {
    pub fn new() -> Self {
        SimpleEdgeSetIntersector
    }

    fn compute_intersects<T: GeoFloat, Z: GeoFloat>(
        &mut self,
        edge0: &Arc<RwLock<Edge<T, Z>>>,
        edge1: &Arc<RwLock<Edge<T, Z>>>,
        segment_intersector: &mut SegmentIntersector<T, Z>,
    ) {
        let edge0_coords_len = edge0.read().coords().len() - 1;
        let edge1_coords_len = edge1.read().coords().len() - 1;
        for i0 in 0..edge0_coords_len {
            for i1 in 0..edge1_coords_len {
                segment_intersector.add_intersections(edge0, i0, edge1, i1);
            }
        }
    }
}

impl<T: GeoFloat, Z: GeoFloat> EdgeSetIntersector<T, Z> for SimpleEdgeSetIntersector {
    fn compute_intersections_within_set(
        &mut self,
        edges: &[Arc<RwLock<Edge<T, Z>>>],
        check_for_self_intersecting_edges: bool,
        segment_intersector: &mut SegmentIntersector<T, Z>,
    ) {
        for edge0 in edges.iter() {
            for edge1 in edges.iter() {
                if check_for_self_intersecting_edges || edge0.data_ptr() != edge1.data_ptr() {
                    self.compute_intersects(edge0, edge1, segment_intersector);
                }
            }
        }
    }

    fn compute_intersections_between_sets(
        &mut self,
        edges0: &[Arc<RwLock<Edge<T, Z>>>],
        edges1: &[Arc<RwLock<Edge<T, Z>>>],
        segment_intersector: &mut SegmentIntersector<T, Z>,
    ) {
        for edge0 in edges0 {
            for edge1 in edges1 {
                self.compute_intersects(edge0, edge1, segment_intersector);
            }
        }
    }
}
