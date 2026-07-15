use std::collections::BTreeSet;

use crate::predicates::kernel::SegmentIntersection;
use crate::predicates::relate::intersection_matrix::Dimensions;

use super::{Direction, EdgeIntersection, IntersectionMatrix, Label};

#[derive(Debug)]
pub(crate) struct Edge {
    /// `coordinates` of the line geometry
    coords: Vec<[f64; 2]>,

    /// an edge is "isolated" if no other edge touches it
    is_isolated: bool,

    /// other edges that this edge intersects with
    edge_intersections: BTreeSet<EdgeIntersection>,

    /// where the line's topological classification to the two geometries is recorded
    label: Label,
}

impl Edge {
    /// Create a new Edge.
    ///
    /// - `coords` a *non-empty* Vec of Coords
    /// - `label` an appropriately dimensioned topology label for the Edge. See
    ///   [`TopologyPosition`](super::TopologyPosition) for details
    pub(crate) fn new(mut coords: Vec<[f64; 2]>, label: Label) -> Edge {
        assert!(!coords.is_empty(), "Can't add empty edge");
        coords.shrink_to_fit();
        Edge {
            coords,
            label,
            is_isolated: true,
            edge_intersections: BTreeSet::new(),
        }
    }

    pub(crate) fn label(&self) -> &Label {
        &self.label
    }

    pub(crate) fn label_mut(&mut self) -> &mut Label {
        &mut self.label
    }

    pub fn coords(&self) -> &[[f64; 2]] {
        &self.coords
    }

    pub fn is_isolated(&self) -> bool {
        self.is_isolated
    }
    pub fn mark_as_unisolated(&mut self) {
        self.is_isolated = false;
    }

    pub fn edge_intersections(&self) -> &BTreeSet<EdgeIntersection> {
        &self.edge_intersections
    }

    pub fn edge_intersections_mut(&mut self) -> &mut BTreeSet<EdgeIntersection> {
        &mut self.edge_intersections
    }

    pub fn add_edge_intersection_list_endpoints(&mut self) {
        let max_segment_index = self.coords().len() - 1;
        let first_coord = self.coords()[0];
        let max_coord = self.coords()[max_segment_index];
        self.edge_intersections_mut()
            .insert(EdgeIntersection::new(first_coord, 0, 0.0));
        self.edge_intersections_mut().insert(EdgeIntersection::new(
            max_coord,
            max_segment_index,
            0.0,
        ));
    }

    pub fn is_closed(&self) -> bool {
        self.coords().first() == self.coords().last()
    }

    /// Adds EdgeIntersections for one or both intersections found for a segment of an edge to the
    /// edge intersection list.
    pub fn add_intersections(
        &mut self,
        intersection: SegmentIntersection,
        line: ([f64; 2], [f64; 2]),
        segment_index: usize,
    ) {
        match intersection {
            SegmentIntersection::SinglePoint { intersection, .. } => {
                self.add_intersection(intersection, line, segment_index);
            }
            SegmentIntersection::Collinear { start, end } => {
                self.add_intersection(start, line, segment_index);
                self.add_intersection(end, line, segment_index);
            }
        }
    }

    /// Add an EdgeIntersection for `intersection`.
    ///
    /// An intersection that falls exactly on a vertex of the edge is normalized to use the higher
    /// of the two possible `segment_index`
    pub fn add_intersection(
        &mut self,
        intersection_coord: [f64; 2],
        line: ([f64; 2], [f64; 2]),
        segment_index: usize,
    ) {
        let mut normalized_segment_index = segment_index;
        let mut distance = compute_edge_distance(intersection_coord, line.0, line.1);

        let next_segment_index = normalized_segment_index + 1;

        if next_segment_index < self.coords.len() {
            let next_coord = self.coords[next_segment_index];
            if intersection_coord == next_coord {
                normalized_segment_index = next_segment_index;
                distance = 0.0;
            }
        }
        self.edge_intersections.insert(EdgeIntersection::new(
            intersection_coord,
            normalized_segment_index,
            distance,
        ));
    }

    /// Update the IM with the contribution for this component.
    ///
    /// A component only contributes if it has a labelling for both parent geometries
    pub fn update_intersection_matrix(label: &Label, intersection_matrix: &mut IntersectionMatrix) {
        intersection_matrix.set_at_least_if_in_both(
            label.position(0, Direction::On),
            label.position(1, Direction::On),
            Dimensions::OneDimensional,
        );

        if label.is_area() {
            intersection_matrix.set_at_least_if_in_both(
                label.position(0, Direction::Left),
                label.position(1, Direction::Left),
                Dimensions::TwoDimensional,
            );
            intersection_matrix.set_at_least_if_in_both(
                label.position(0, Direction::Right),
                label.position(1, Direction::Right),
                Dimensions::TwoDimensional,
            );
        }
    }
}

/// The relative position of `intersection` along the segment `start -> end`,
/// measured on the segment's dominant axis. Not a true distance: only its
/// ordering along the segment matters (it keys [`EdgeIntersection`]'s sort).
fn compute_edge_distance(intersection: [f64; 2], start: [f64; 2], end: [f64; 2]) -> f64 {
    let dx = (end[0] - start[0]).abs();
    let dy = (end[1] - start[1]).abs();

    let mut dist: f64;
    if intersection == start {
        dist = 0.0;
    } else if intersection == end {
        if dx > dy {
            dist = dx;
        } else {
            dist = dy;
        }
    } else {
        let intersection_dx = (intersection[0] - start[0]).abs();
        let intersection_dy = (intersection[1] - start[1]).abs();
        if dx > dy {
            dist = intersection_dx;
        } else {
            dist = intersection_dy;
        }
        // hack to ensure that non-endpoints always have a non-zero distance
        if dist == 0.0 && intersection != start {
            dist = intersection_dx.max(intersection_dy);
        }
    }
    dist
}
