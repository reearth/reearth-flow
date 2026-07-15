use crate::predicates::kernel::{orient2d, CoordPos, Orientation};
use crate::predicates::relate::operand::RelateOperand;
use crate::predicates::view::Leaf2D;

use super::index::{
    compute_intersections_between_sets, compute_intersections_within_set, SegmentIntersector,
};
use super::{determine_boundary, CoordNode, Direction, Edge, Label, PlanarGraph, TopologyPosition};

use std::cell::RefCell;
use std::rc::Rc;

/// One operand's topology graph: its components as labeled nodes and edges.
///
/// Built from a [`RelateOperand`]: flattened leaves, with mesh leaves
/// contributing their union-boundary rings rather than raw faces.
pub(crate) struct GeometryGraph<'a> {
    arg_index: usize,
    parent_geometry: &'a RelateOperand<'a>,
    use_boundary_determination_rule: bool,
    planar_graph: PlanarGraph,
}

///  PlanarGraph delegations
///
/// In JTS, which is written in Java, GeometryGraph inherits from PlanarGraph. Here in Rust land we
/// use composition and delegation to the same effect.
impl GeometryGraph<'_> {
    pub fn edges(&self) -> &[Rc<RefCell<Edge>>] {
        self.planar_graph.edges()
    }

    pub fn insert_edge(&mut self, edge: Edge) {
        self.planar_graph.insert_edge(edge)
    }

    pub fn is_boundary_node(&self, coord: [f64; 2]) -> bool {
        self.planar_graph.is_boundary_node(self.arg_index, coord)
    }

    pub fn add_node_with_coordinate(&mut self, coord: [f64; 2]) -> &mut CoordNode {
        self.planar_graph.add_node_with_coordinate(coord)
    }

    pub fn nodes_iter(&self) -> impl Iterator<Item = &CoordNode> {
        self.planar_graph.nodes.iter()
    }
}

impl<'a> GeometryGraph<'a> {
    pub fn new(arg_index: usize, parent_geometry: &'a RelateOperand<'a>) -> Self {
        let mut graph = GeometryGraph {
            arg_index,
            parent_geometry,
            // Meshes are the MultiPolygon analog: in JTS every collection
            // except MultiPolygon obeys the mod-2 boundary rule.
            use_boundary_determination_rule: !parent_geometry.has_mesh(),
            planar_graph: PlanarGraph::new(),
        };
        graph.add_geometry();
        graph
    }

    pub fn geometry(&self) -> &RelateOperand<'a> {
        self.parent_geometry
    }

    fn boundary_nodes(&self) -> impl Iterator<Item = &CoordNode> {
        self.planar_graph.boundary_nodes(self.arg_index)
    }

    fn add_geometry(&mut self) {
        for (i, prepared) in self.parent_geometry.leaves().iter().enumerate() {
            match prepared.leaf {
                Leaf2D::Point(point) => self.add_point(point.position()),
                Leaf2D::Line(line) => self.add_line_string(line.coords()),
                Leaf2D::Polygon(polygon) => {
                    self.add_polygon_ring(polygon.exterior(), CoordPos::Outside, CoordPos::Inside);
                    // Holes are topologically labeled opposite to the shell, since
                    // the interior of the polygon lies on their opposite side
                    // (on the left, if the hole is oriented CW)
                    for hole in polygon.interiors() {
                        self.add_polygon_ring(hole, CoordPos::Inside, CoordPos::Outside)
                    }
                }
                Leaf2D::PolygonMesh(_) | Leaf2D::TriangularMesh(_) => {
                    let rings = self
                        .parent_geometry
                        .boundary_rings(i)
                        .expect("mesh leaves carry boundary rings")
                        .to_vec();
                    for ring in rings {
                        // Union-boundary rings preserve face-edge direction, so
                        // the union interior is on the left by construction.
                        self.insert_ring_edge(
                            dedup_consecutive(&ring),
                            CoordPos::Inside,
                            CoordPos::Outside,
                        );
                    }
                }
            }
        }
    }

    fn add_polygon_ring(
        &mut self,
        linear_ring: &[[f64; 2]],
        cw_left: CoordPos,
        cw_right: CoordPos,
    ) {
        debug_assert!(
            linear_ring.len() < 2 || linear_ring.first() == linear_ring.last(),
            "polygon rings are stored closed"
        );
        if linear_ring.is_empty() {
            return;
        }

        let coords = dedup_consecutive(linear_ring);

        let (left, right) = match ring_winding_order(&coords) {
            Some(WindingOrder::Clockwise) => (cw_left, cw_right),
            Some(WindingOrder::CounterClockwise) => (cw_right, cw_left),
            None => (cw_left, cw_right),
        };

        self.insert_ring_edge(coords, left, right);
    }

    /// Insert a closed area-boundary edge whose interior side is already
    /// resolved, and mark its start as a boundary node.
    fn insert_ring_edge(&mut self, coords: Vec<[f64; 2]>, left: CoordPos, right: CoordPos) {
        if coords.is_empty() {
            return;
        }
        let first_point = coords[0];

        let edge = Edge::new(
            coords,
            Label::new(
                self.arg_index,
                TopologyPosition::area(CoordPos::OnBoundary, left, right),
            ),
        );
        self.insert_edge(edge);

        // insert the endpoint as a node, to mark that it is on the boundary
        self.insert_point(self.arg_index, first_point, CoordPos::OnBoundary);
    }

    fn add_line_string(&mut self, line_string: &[[f64; 2]]) {
        if line_string.is_empty() {
            return;
        }

        let coords = dedup_consecutive(line_string);

        if coords.len() < 2 {
            self.add_point(coords[0]);
            return;
        }

        self.insert_boundary_point(*coords.first().unwrap());
        self.insert_boundary_point(*coords.last().unwrap());

        let edge = Edge::new(
            coords,
            Label::new(
                self.arg_index,
                TopologyPosition::line_or_point(CoordPos::Inside),
            ),
        );
        self.insert_edge(edge);
    }

    /// Add a point computed externally.  The point is assumed to be a
    /// Point Geometry part, which has a location of INTERIOR.
    fn add_point(&mut self, point: [f64; 2]) {
        self.insert_point(self.arg_index, point, CoordPos::Inside);
    }

    /// Compute self-nodes, taking advantage of the Geometry type to minimize the number of
    /// intersection tests.  (E.g. rings are not tested for self-intersection, since they are
    /// assumed to be valid).
    pub fn compute_self_nodes(&mut self) -> SegmentIntersector {
        let mut segment_intersector = SegmentIntersector::new(true);

        // optimize intersection search for valid Polygons and LinearRings
        let is_rings = self.geometry().is_rings();
        let check_for_self_intersecting_edges = !is_rings;

        compute_intersections_within_set(
            self.edges(),
            check_for_self_intersecting_edges,
            &mut segment_intersector,
        );

        self.add_self_intersection_nodes();

        segment_intersector
    }

    pub fn compute_edge_intersections(&self, other: &GeometryGraph<'_>) -> SegmentIntersector {
        let mut segment_intersector = SegmentIntersector::new(false);
        segment_intersector.set_boundary_nodes(
            self.boundary_nodes().cloned().collect(),
            other.boundary_nodes().cloned().collect(),
        );

        compute_intersections_between_sets(self.edges(), other.edges(), &mut segment_intersector);

        segment_intersector
    }

    fn insert_point(&mut self, arg_index: usize, coord: [f64; 2], position: CoordPos) {
        let node: &mut CoordNode = self.add_node_with_coordinate(coord);
        node.label_mut().set_on_position(arg_index, position);
    }

    /// Add the boundary points of 1-dim (line) geometries.
    fn insert_boundary_point(&mut self, coord: [f64; 2]) {
        let arg_index = self.arg_index;
        let node: &mut CoordNode = self.add_node_with_coordinate(coord);

        let label: &mut Label = node.label_mut();

        // determine the current location for the point (if any)
        let boundary_count = {
            #[allow(clippy::bool_to_int_with_if)]
            let prev_boundary_count =
                if Some(CoordPos::OnBoundary) == label.position(arg_index, Direction::On) {
                    1
                } else {
                    0
                };
            prev_boundary_count + 1
        };

        let new_position = determine_boundary(boundary_count);
        label.set_on_position(arg_index, new_position);
    }

    fn add_self_intersection_nodes(&mut self) {
        let positions_and_intersections: Vec<(CoordPos, Vec<[f64; 2]>)> = self
            .edges()
            .iter()
            .map(|cell| cell.borrow())
            .map(|edge| {
                let position = edge
                    .label()
                    .on_position(self.arg_index)
                    .expect("all edge labels should have an `on` position by now");
                let coordinates = edge
                    .edge_intersections()
                    .iter()
                    .map(|edge_intersection| edge_intersection.coordinate());

                (position, coordinates.collect())
            })
            .collect();

        for (position, edge_intersection_coordinates) in positions_and_intersections {
            for coordinate in edge_intersection_coordinates {
                self.add_self_intersection_node(coordinate, position)
            }
        }
    }

    /// Add a node for a self-intersection.
    ///
    /// If the node is a potential boundary node (e.g. came from an edge which is a boundary), then
    /// insert it as a potential boundary node.  Otherwise, just add it as a regular node.
    fn add_self_intersection_node(&mut self, coord: [f64; 2], position: CoordPos) {
        // if this node is already a boundary node, don't change it
        if self.is_boundary_node(coord) {
            return;
        }

        if position == CoordPos::OnBoundary && self.use_boundary_determination_rule {
            self.insert_boundary_point(coord)
        } else {
            self.insert_point(self.arg_index, coord, position)
        }
    }
}

/// Copy `coords` with consecutive duplicates removed (a closed ring keeps its
/// closing duplicate, which is not consecutive with itself).
fn dedup_consecutive(coords: &[[f64; 2]]) -> Vec<[f64; 2]> {
    let mut out: Vec<[f64; 2]> = Vec::with_capacity(coords.len());
    for coord in coords {
        if out.last() != Some(coord) {
            out.push(*coord);
        }
    }
    out
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WindingOrder {
    Clockwise,
    CounterClockwise,
}

/// The winding order of a closed ring, by robust orientation at its
/// lexicographically least vertex. `None` for open or degenerate rings.
fn ring_winding_order(ring: &[[f64; 2]]) -> Option<WindingOrder> {
    // If the ring has at most 3 coords, it is either not closed, or is at
    // most two distinct points. Either way, the winding is unspecified.
    if ring.len() < 4 || ring.first() != ring.last() {
        return None;
    }
    let least = ring
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| a.partial_cmp(b).expect("ring coordinates must not be NaN"))
        .map(|(i, _)| i)
        .expect("ring is non-empty");

    let n = ring.len();
    let increment = |x: usize| if x + 1 >= n { 0 } else { x + 1 };
    let decrement = |x: usize| if x == 0 { n - 1 } else { x - 1 };

    let mut next = increment(least);
    while ring[next] == ring[least] {
        if next == least {
            // We've looped too much. There aren't enough unique coords to
            // compute orientation.
            return None;
        }
        next = increment(next);
    }

    let mut prev = decrement(least);
    while ring[prev] == ring[least] {
        // We don't need to check if prev == least as the previous loop
        // succeeded, so there are at least two distinct elements.
        prev = decrement(prev);
    }

    match orient2d(ring[prev], ring[least], ring[next]) {
        Orientation::CounterClockwise => Some(WindingOrder::CounterClockwise),
        Orientation::Clockwise => Some(WindingOrder::Clockwise),
        Orientation::Collinear => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn winding_of_simple_rings() {
        let ccw = [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0], [0.0, 0.0]];
        assert_eq!(
            ring_winding_order(&ccw),
            Some(WindingOrder::CounterClockwise)
        );
        let cw: Vec<[f64; 2]> = ccw.iter().rev().copied().collect();
        assert_eq!(ring_winding_order(&cw), Some(WindingOrder::Clockwise));
        // Open chains and degenerate rings have no winding.
        assert_eq!(ring_winding_order(&ccw[..4]), None);
        assert_eq!(
            ring_winding_order(&[[1.0, 1.0], [1.0, 1.0], [1.0, 1.0], [1.0, 1.0]]),
            None
        );
    }

    #[test]
    fn dedup_keeps_closing_duplicate() {
        let ring = [
            [0.0, 0.0],
            [0.0, 0.0],
            [4.0, 0.0],
            [4.0, 4.0],
            [4.0, 4.0],
            [0.0, 0.0],
        ];
        assert_eq!(
            dedup_consecutive(&ring),
            vec![[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 0.0]]
        );
    }
}
