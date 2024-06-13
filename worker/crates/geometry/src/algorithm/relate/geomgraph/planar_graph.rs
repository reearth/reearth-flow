use super::{
    node_map::{NodeFactory, NodeMap},
    CoordNode, Edge, Label,
};
use crate::{
    algorithm::{coordinate_position::CoordPos, GeoFloat},
    types::coordinate::Coordinate,
};

use std::cell::RefCell;
use std::rc::Rc;

pub(crate) struct PlanarGraphNode;

/// The basic node constructor does not allow for incident edges
impl<T, Z> NodeFactory<T, Z> for PlanarGraphNode
where
    T: GeoFloat,
    Z: GeoFloat,
{
    type Node = CoordNode<T, Z>;
    fn create_node(coordinate: Coordinate<T, Z>) -> Self::Node {
        CoordNode::new(coordinate)
    }
}

pub(crate) struct PlanarGraph<T: GeoFloat, Z: GeoFloat> {
    pub(crate) nodes: NodeMap<T, Z, PlanarGraphNode>,
    edges: Vec<Rc<RefCell<Edge<T, Z>>>>,
}

impl<T: GeoFloat, Z: GeoFloat> PlanarGraph<T, Z> {
    pub fn edges(&self) -> &[Rc<RefCell<Edge<T, Z>>>] {
        &self.edges
    }

    pub fn new() -> Self {
        PlanarGraph {
            nodes: NodeMap::new(),
            edges: vec![],
        }
    }

    pub fn is_boundary_node(&self, geom_index: usize, coord: Coordinate<T, Z>) -> bool {
        self.nodes
            .find(coord)
            .and_then(|node| node.label().on_position(geom_index))
            .map(|position| position == CoordPos::OnBoundary)
            .unwrap_or(false)
    }

    pub fn insert_edge(&mut self, edge: Edge<T, Z>) {
        self.edges.push(Rc::new(RefCell::new(edge)));
    }

    pub fn add_node_with_coordinate(&mut self, coord: Coordinate<T, Z>) -> &mut CoordNode<T, Z> {
        self.nodes.insert_node_with_coordinate(coord)
    }

    pub fn boundary_nodes(&self, geom_index: usize) -> impl Iterator<Item = &CoordNode<T, Z>> {
        self.nodes.iter().filter(move |node| {
            matches!(
                node.label().on_position(geom_index),
                Some(CoordPos::OnBoundary)
            )
        })
    }
}
