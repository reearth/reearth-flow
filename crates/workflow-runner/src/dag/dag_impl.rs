use std::collections::{HashMap, HashSet};

use anyhow::Result;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use petgraph::Direction;

use reearth_flow_action::action::Port;
use reearth_flow_workflow::graph::Node;
use reearth_flow_workflow::id::Id;

use super::error::Error;

pub type NodeId = Id;
pub type GraphId = Id;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Endpoint {
    pub node: NodeIndex,
    pub port: Port,
}

impl Endpoint {
    pub fn new(node: NodeIndex, port: Port) -> Self {
        Self { node, port }
    }
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub from: Endpoint,
    pub to: Endpoint,
}

impl Edge {
    pub fn new(from: Endpoint, to: Endpoint) -> Self {
        Self { from, to }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeType {
    pub id: NodeId,
}

pub struct EdgeType {
    pub from: Port,
    pub to: Port,
}

impl EdgeType {
    pub fn new(from: Port, to: Port) -> Self {
        Self { from, to }
    }
}

pub trait EdgeHavePorts {
    #[allow(clippy::wrong_self_convention)]
    fn from_port(&self) -> Port;
    fn to_port(&self) -> Port;
}

impl EdgeHavePorts for EdgeType {
    fn from_port(&self) -> Port {
        self.from.to_owned()
    }

    fn to_port(&self) -> Port {
        self.to.to_owned()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EdgeIndex {
    pub from_node: NodeIndex,
    pub from_port: Port,
    pub to_node: NodeIndex,
    pub to_port: Port,
}

pub struct Dag {
    pub id: GraphId,
    /// The graph.
    graph: DiGraph<NodeType, EdgeType>,
    /// Lookup table for node indexes.
    node_lookup_table: HashMap<NodeType, NodeIndex>,
    /// All edge indexes.
    edge_indexes: HashSet<EdgeIndex>,
    /// All nodes.
    nodes: HashMap<NodeId, Node>,
}

impl Dag {
    pub fn from_graph(graph: &reearth_flow_workflow::graph::Graph) -> Result<Self> {
        let mut dag = Self {
            id: graph.id,
            graph: DiGraph::<NodeType, EdgeType>::new(),
            node_lookup_table: HashMap::new(),
            edge_indexes: HashSet::new(),
            nodes: HashMap::new(),
        };
        graph.nodes.iter().for_each(|node| {
            dag.add_node(node.clone());
        });
        for edge in graph.edges.iter() {
            let from = dag.node(edge.from).ok_or(Error::Init(format!(
                "Failed to get from node with edge = {:?}",
                edge
            )))?;
            let to = dag.node(edge.to).ok_or(Error::Init(format!(
                "Failed to get to nodes with edge = {:?}",
                edge
            )))?;
            let from_node_index = dag.node_index(from)?;
            let to_node_index = dag.node_index(to)?;
            dag.connect(
                &Endpoint::new(from_node_index, edge.from_port.to_owned()),
                &Endpoint::new(to_node_index, edge.to_port.to_owned()),
            )?;
        }
        Ok(dag)
    }

    pub fn graph(&self) -> &DiGraph<NodeType, EdgeType> {
        &self.graph
    }

    pub fn entry_nodes(&self) -> Vec<Node> {
        self.graph
            .externals(Direction::Incoming)
            .map(|node_index| {
                let node = &self.graph[node_index];
                let node = self
                    .node(node.id)
                    .ok_or(Error::node_not_found(format!("node_id = {}", node.id)))
                    .unwrap();
                node.clone()
            })
            .collect()
    }

    pub fn is_last_node_index(&self, idx: NodeIndex) -> bool {
        self.graph
            .edges_directed(idx, Direction::Outgoing)
            .next()
            .is_none()
    }

    pub fn add_node(&mut self, node: Node) -> NodeIndex {
        let node_type = NodeType { id: node.id() };
        let node_index = self.graph.add_node(node_type.clone());
        self.nodes.insert(node.id(), node);
        self.node_lookup_table.insert(node_type.clone(), node_index);
        node_index
    }

    pub fn node_index(&self, node: &Node) -> Result<NodeIndex> {
        self.node_lookup_table
            .get(&NodeType { id: node.id() })
            .copied()
            .ok_or_else(|| Error::node_not_found(format!("node_id = {}", node.id())).into())
    }

    pub fn is_ready_node(&self, idx: NodeIndex, finish_ports: Vec<Port>) -> bool {
        let to_all_ports = self
            .graph
            .edges_directed(idx, Direction::Incoming)
            .map(|edge| edge.weight().to_port())
            .collect::<Vec<_>>();
        let mut finish_ports = finish_ports.clone();
        let mut to_all_ports = to_all_ports.clone();
        finish_ports.sort();
        to_all_ports.sort();
        finish_ports == to_all_ports
    }

    pub fn connect(&mut self, from: &Endpoint, to: &Endpoint) -> Result<()> {
        self.connect_with_index(from.node, &from.port, to.node, &to.port)
    }

    pub fn connect_with_index(
        &mut self,
        from_node_index: NodeIndex,
        from_port: &Port,
        to_node_index: NodeIndex,
        to_port: &Port,
    ) -> Result<()> {
        let edge_index = self.graph.add_edge(
            from_node_index,
            to_node_index,
            EdgeType::new(from_port.clone(), to_port.clone()),
        );

        if !self.edge_indexes.insert(EdgeIndex {
            from_node: from_node_index,
            from_port: from_port.clone(),
            to_node: to_node_index,
            to_port: to_port.clone(),
        }) {
            Err(Error::edge_already_exists(format!("edge = {:?}", edge_index)).into())
        } else {
            Ok(())
        }
    }

    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.values()
    }

    pub fn node(&self, node_id: NodeId) -> Option<&Node> {
        self.nodes.get(&node_id)
    }

    pub fn node_from_index(&self, idx: NodeIndex) -> Option<&Node> {
        let node_type = &self.graph[idx];
        self.nodes.get(&node_type.id)
    }

    pub fn node_mut(&mut self, node_id: &NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(node_id)
    }

    pub fn edges(&self) -> Vec<Edge> {
        let get_endpoint = |node_index: NodeIndex, port: &Port| Endpoint {
            node: node_index,
            port: port.clone(),
        };

        self.edge_indexes
            .iter()
            .map(|edge_index| {
                let from = get_endpoint(edge_index.from_node, &edge_index.from_port);
                let to = get_endpoint(edge_index.to_node, &edge_index.to_port);
                Edge::new(from, to)
            })
            .collect()
    }

    pub fn edges_from_node(&self, node: &Node) -> impl Iterator<Item = EdgeIndex> + '_ {
        let node_index = self.node_index(node).unwrap();
        self.edges_from_node_index(node_index)
    }

    pub fn edges_from_node_index(
        &self,
        node_index: NodeIndex,
    ) -> impl Iterator<Item = EdgeIndex> + '_ {
        self.graph.edges(node_index).map(|edge| {
            let from_node = edge.source();
            let to_node = edge.target();
            let weight = edge.weight();
            let from_port = weight.from_port().clone();
            let to_port = weight.to_port().clone();
            EdgeIndex {
                from_node,
                from_port,
                to_node,
                to_port,
            }
        })
    }

    pub fn edges_from_endpoint<'a>(
        &'a self,
        node: &'a Node,
        port: &'a Port,
    ) -> impl Iterator<Item = (&Node, Port)> {
        self.graph
            .edges(self.node_index(node).unwrap())
            .filter_map(move |edge| {
                if edge.weight().from_port() == *port {
                    let node = &self.graph[edge.target()];
                    let node = self
                        .node(node.id)
                        .ok_or(Error::node_not_found(format!("node_id = {}", node.id)))
                        .unwrap();
                    Some((node, edge.weight().to_port().clone()))
                } else {
                    None
                }
            })
    }
}
