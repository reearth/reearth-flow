use std::cell::RefCell;
use std::rc::Rc;

use super::graph::{Edge, EdgeEnd, EdgeIntersection};

/// Computes the [`EdgeEnd`]s which arise from a noded [`Edge`]: one stub per
/// direction at every intersection point along the edge.
pub(crate) struct EdgeEndBuilder;

impl EdgeEndBuilder {
    pub fn new() -> Self {
        EdgeEndBuilder
    }

    pub fn compute_ends_for_edges(&self, edges: &[Rc<RefCell<Edge>>]) -> Vec<EdgeEnd> {
        let mut list = vec![];
        for edge in edges {
            self.compute_ends_for_edge(&mut edge.borrow_mut(), &mut list);
        }
        list
    }

    fn compute_ends_for_edge(&self, edge: &mut Edge, list: &mut Vec<EdgeEnd>) {
        edge.add_edge_intersection_list_endpoints();

        let mut ei_iter = edge.edge_intersections().iter();
        let mut ei_prev;
        let mut ei_curr = None;
        let mut ei_next = ei_iter.next();
        if ei_next.is_none() {
            return;
        }

        loop {
            ei_prev = ei_curr;
            ei_curr = ei_next;
            ei_next = ei_iter.next();
            if let Some(ei_curr) = ei_curr {
                self.create_edge_end_for_prev(edge, list, ei_curr, ei_prev);
                self.create_edge_end_for_next(edge, list, ei_curr, ei_next);
            }

            if ei_curr.is_none() {
                break;
            }
        }
    }

    /// Adds a `EdgeEnd`, if any, to `list` for the edge before the intersection ei_curr.
    ///
    /// The previous intersection is provided in case it is the endpoint for the stub edge.
    /// Otherwise, the previous point from the parent edge will be the endpoint.
    fn create_edge_end_for_prev(
        &self,
        edge: &Edge,
        list: &mut Vec<EdgeEnd>,
        ei_curr: &EdgeIntersection,
        ei_prev: Option<&EdgeIntersection>,
    ) {
        let mut i_prev = ei_curr.segment_index();
        if ei_curr.distance() == 0.0 {
            // if at the start of the edge there is no previous edge
            if i_prev == 0 {
                return;
            }
            i_prev -= 1;
        }

        let mut coord_prev = edge.coords()[i_prev];
        // if prev intersection is past the previous vertex, use it instead
        if let Some(ei_prev) = ei_prev {
            if ei_prev.segment_index() >= i_prev {
                coord_prev = ei_prev.coordinate();
            }
        }

        let mut label = edge.label().clone();
        // since edgeStub is oriented opposite to its parent edge, have to flip sides for edge label
        label.flip();

        let edge_end = EdgeEnd::new(ei_curr.coordinate(), coord_prev, label);
        list.push(edge_end);
    }

    /// Adds a `EdgeEnd`, if any, to `list` for the edge after the intersection ei_curr.
    ///
    /// The next intersection is provided in case it is the endpoint for the stub edge.
    /// Otherwise, the next point from the parent edge will be the endpoint.
    fn create_edge_end_for_next(
        &self,
        edge: &Edge,
        list: &mut Vec<EdgeEnd>,
        ei_curr: &EdgeIntersection,
        ei_next: Option<&EdgeIntersection>,
    ) {
        let i_next = ei_curr.segment_index() + 1;

        // if there is no next edge there is nothing to do
        if i_next >= edge.coords().len() && ei_next.is_none() {
            return;
        }

        let mut coord_next = edge.coords()[i_next];

        // if the next intersection is in the same segment as the current, use it as the endpoint
        if let Some(ei_next) = ei_next {
            if ei_next.segment_index() == ei_curr.segment_index() {
                coord_next = ei_next.coordinate();
            }
        }

        let label = edge.label().clone();
        let edge_end = EdgeEnd::new(ei_curr.coordinate(), coord_next, label);
        list.push(edge_end);
    }
}
