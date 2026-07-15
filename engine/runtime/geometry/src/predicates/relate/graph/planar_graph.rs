use super::{
    node_map::{NodeFactory, NodeMap},
    CoordNode, Edge,
};
use crate::predicates::kernel::CoordPos;

use std::cell::RefCell;
use std::rc::Rc;

pub(crate) struct PlanarGraphNode;

/// The basic node constructor does not allow for incident edges
impl NodeFactory for PlanarGraphNode {
    type Node = CoordNode;
    fn create_node(coordinate: [f64; 2]) -> Self::Node {
        CoordNode::new(coordinate)
    }
}

pub(crate) struct PlanarGraph {
    pub(crate) nodes: NodeMap<PlanarGraphNode>,
    edges: Vec<Rc<RefCell<Edge>>>,
}

impl PlanarGraph {
    pub fn edges(&self) -> &[Rc<RefCell<Edge>>] {
        &self.edges
    }

    pub fn new() -> Self {
        PlanarGraph {
            nodes: NodeMap::new(),
            edges: vec![],
        }
    }

    pub fn is_boundary_node(&self, geom_index: usize, coord: [f64; 2]) -> bool {
        self.nodes
            .find(coord)
            .and_then(|node| node.label().on_position(geom_index))
            .map(|position| position == CoordPos::OnBoundary)
            .unwrap_or(false)
    }

    pub fn insert_edge(&mut self, edge: Edge) {
        self.edges.push(Rc::new(RefCell::new(edge)));
    }

    pub fn add_node_with_coordinate(&mut self, coord: [f64; 2]) -> &mut CoordNode {
        self.nodes.insert_node_with_coordinate(coord)
    }

    pub fn boundary_nodes(&self, geom_index: usize) -> impl Iterator<Item = &CoordNode> {
        self.nodes.iter().filter(move |node| {
            matches!(
                node.label().on_position(geom_index),
                Some(CoordPos::OnBoundary)
            )
        })
    }
}
