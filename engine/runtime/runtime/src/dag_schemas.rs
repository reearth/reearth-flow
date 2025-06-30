use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

use petgraph::dot::Dot;
use petgraph::graph::{DiGraph, EdgeIndex, NodeIndex};
use petgraph::visit::EdgeRef;
use petgraph::Direction;

use reearth_flow_types::workflow::{Graph, Node};

use crate::node::{
    EdgeId, GraphId, NodeHandle, NodeId, NodeKind, Port, INPUT_ROUTING_ACTION,
    OUTPUT_ROUTING_ACTION, ROUTING_PARAM_KEY,
};

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SchemaEdgeKind {
    FromSource,
    FromProcessor,
}

#[derive(Clone)]
pub struct SchemaNodeType {
    pub handle: NodeHandle,
    pub name: String,
    pub node: Node,
    pub kind: Option<NodeKind>,
    pub with: Option<HashMap<String, serde_json::Value>>,
}

impl Debug for SchemaNodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SchemaNodeType(id = {}, name = {})",
            self.handle.id, self.name
        )
    }
}

impl SchemaNodeType {
    pub fn new(
        id: NodeId,
        name: String,
        node: Node,
        kind: Option<NodeKind>,
        with: Option<HashMap<String, serde_json::Value>>,
    ) -> Self {
        Self {
            handle: NodeHandle::new(id),
            name,
            node,
            kind,
            with,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SchemaEdgeType {
    pub id: EdgeId,
    pub from: Port,
    pub to: Port,
    pub edge_kind: Option<SchemaEdgeKind>,
}

impl SchemaEdgeType {
    pub fn new(id: EdgeId, from: Port, to: Port, edge_kind: Option<SchemaEdgeKind>) -> Self {
        Self {
            id,
            from,
            to,
            edge_kind,
        }
    }
}

pub trait EdgeHavePorts {
    #[allow(clippy::wrong_self_convention)]
    fn from_port(&self) -> Port;
    fn to_port(&self) -> Port;
}

impl EdgeHavePorts for SchemaEdgeType {
    fn from_port(&self) -> Port {
        self.from.to_owned()
    }

    fn to_port(&self) -> Port {
        self.to.to_owned()
    }
}

pub struct DagSchemas {
    pub(crate) id: GraphId,
    graph: DiGraph<SchemaNodeType, SchemaEdgeType>,
    /// Lookup table for node indexes.
    node_lookup_table: HashMap<NodeId, NodeIndex>,
}

impl DagSchemas {
    pub fn from_graphs(
        entry_graph_id: GraphId,
        graphs: Vec<Graph>,
        factories: HashMap<String, NodeKind>,
        global_params: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Self {
        let entry_graph = graphs
            .iter()
            .find(|dag| dag.id == entry_graph_id)
            .unwrap_or_else(|| panic!("Entry graph not found. with id = {entry_graph_id}"));
        let other_graphs = graphs
            .iter()
            .filter(|graph| graph.id != entry_graph_id)
            .map(|graph| (graph.id, graph))
            .collect::<HashMap<_, _>>();

        let mut other_graph_schemas = HashMap::new();
        for (_, graph) in other_graphs.iter() {
            let mut graph_schema = DagSchemas::from_graph(graph, &factories, &global_params);
            let graph_nodes = graph_schema.collect_graph_nodes();
            for node in graph_nodes.iter() {
                let Node::SubGraph {
                    sub_graph_id,
                    entity,
                } = &node.node
                else {
                    continue;
                };
                if *sub_graph_id == graph.id {
                    panic!("Self reference subgraph is not allowed.");
                }
                let subgraph = other_graphs
                    .get(sub_graph_id)
                    .unwrap_or_else(|| panic!("Subgraph not found. with id = {sub_graph_id}"));
                let params = if let Some(with) = &entity.with {
                    if let Some(global_params) = &global_params {
                        let mut global_with = global_params.clone();
                        global_with.extend(with.clone());
                        Some(global_with)
                    } else {
                        Some(with.clone())
                    }
                } else {
                    global_params.clone()
                };
                let mut subgraph = DagSchemas::from_graph(subgraph, &factories, &params);
                for edge in subgraph.graph.edge_weights_mut() {
                    edge.id = EdgeId::new(format!("{}.{}", entity.id, edge.id));
                }
                graph_schema.add_subgraph_after_node(node.handle.id.clone(), &params, &subgraph);
                let Some(target_node) = graph_schema.node_index_by_node_id(node.handle.id.clone())
                else {
                    continue;
                };
                graph_schema.graph.remove_node(*target_node);
            }
            other_graph_schemas.insert(graph_schema.id, graph_schema);
        }
        let mut entry_graph = DagSchemas::from_graph(entry_graph, &factories, &global_params);
        let graph_nodes = entry_graph.collect_graph_nodes();
        for node in graph_nodes.iter() {
            let Node::SubGraph {
                sub_graph_id,
                entity,
            } = &node.node
            else {
                continue;
            };
            let params = if let Some(with) = &entity.with {
                if let Some(global_params) = &global_params {
                    let mut global_with = global_params.clone();
                    global_with.extend(with.clone());
                    Some(global_with)
                } else {
                    Some(with.clone())
                }
            } else {
                global_params.clone()
            };
            let subgraph = other_graph_schemas
                .get_mut(sub_graph_id)
                .unwrap_or_else(|| panic!("Subgraph not found. with id = {sub_graph_id}"));
            for edge in subgraph.graph.edge_weights_mut() {
                edge.id = EdgeId::new(format!("{}.{}", entity.id, edge.id));
            }
            entry_graph.add_subgraph_after_node(node.handle.id.clone(), &params, subgraph);
            let Some(target_node) = entry_graph.node_index_by_node_id(node.handle.id.clone())
            else {
                continue;
            };
            entry_graph.graph.remove_node(*target_node);
        }
        entry_graph
    }

    fn from_graph(
        graph: &Graph,
        factories: &HashMap<String, NodeKind>,
        global_params: &Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Self {
        let mut dag = Self {
            id: graph.id,
            graph: DiGraph::<SchemaNodeType, SchemaEdgeType>::new(),
            node_lookup_table: HashMap::new(),
        };
        let mut node_mappings = HashMap::<NodeIndex, NodeKind>::new();
        graph.nodes.iter().for_each(|node| {
            let mut with = HashMap::new();
            if let Some(global_params) = global_params {
                global_params.iter().for_each(|(k, v)| {
                    with.insert(k.clone(), v.clone());
                });
            }
            if let Some(params) = &node.with() {
                params.iter().for_each(|(k, v)| {
                    with.insert(k.clone(), v.clone());
                });
            }
            let kind = match node {
                Node::Action { .. } => {
                    let Some(kind) = factories.get(node.action()) else {
                        panic!("Action not found: {}", node.action());
                    };
                    Some(kind.clone())
                }
                Node::SubGraph { .. } => None,
            };
            let index = dag.add_node(SchemaNodeType::new(
                NodeId::new(node.id().to_string()),
                node.name().to_string(),
                node.clone(),
                kind.clone(),
                Some(with),
            ));
            if let Some(kind) = kind {
                node_mappings.insert(index, kind.clone());
            };
        });
        for edge in graph.edges.iter() {
            let from_node_index = dag
                .node_index_by_node_id(NodeId::new(edge.from.to_string()))
                .unwrap_or_else(|| panic!("From Node not found: {}", edge.from));
            let from_node_kind = node_mappings.get(from_node_index);
            let to_node_index = dag
                .node_index_by_node_id(NodeId::new(edge.to.to_string()))
                .unwrap_or_else(|| panic!("Edge Node not found: {}", edge.to));
            dag.connect(
                EdgeId::new(edge.id.to_string()),
                &Endpoint::new(*from_node_index, Port::new(edge.from_port.clone())),
                &Endpoint::new(*to_node_index, Port::new(edge.to_port.clone())),
                match from_node_kind {
                    Some(from_node_kind) => {
                        if let NodeKind::Source(_) = from_node_kind {
                            Some(SchemaEdgeKind::FromSource)
                        } else {
                            Some(SchemaEdgeKind::FromProcessor)
                        }
                    }
                    _ => None,
                },
            );
        }
        dag
    }

    pub fn into_graph(self) -> DiGraph<SchemaNodeType, SchemaEdgeType> {
        self.graph
    }

    pub fn graph(&self) -> &DiGraph<SchemaNodeType, SchemaEdgeType> {
        &self.graph
    }

    pub fn collect_ancestor_sources(
        &self,
        node_index: petgraph::graph::NodeIndex,
    ) -> HashSet<NodeHandle> {
        let mut sources = HashSet::new();
        collect_ancestor_sources_recursive(self, node_index, &mut sources);
        sources
    }

    pub fn collect_graph_nodes(&self) -> Vec<SchemaNodeType> {
        self.graph
            .node_indices()
            .map(|idx| self.graph[idx].clone())
            .filter(|node| matches!(node.node, Node::SubGraph { .. }))
            .collect()
    }

    pub fn remove_edge(&mut self, edge: petgraph::graph::EdgeIndex) {
        self.graph.remove_edge(edge);
    }

    pub fn to_dot(&self) -> String {
        format!("{:?}", Dot::new(&self.graph))
    }

    pub fn node_index_by_node_id(&self, node_id: NodeId) -> Option<&NodeIndex> {
        self.node_lookup_table.get(&node_id)
    }

    pub fn node_by_node_id(&self, node_id: NodeId) -> Option<&SchemaNodeType> {
        self.node_lookup_table
            .get(&node_id)
            .map(|node_index| &self.graph[*node_index])
    }

    pub fn entry_nodes(&self) -> Vec<SchemaNodeType> {
        self.graph
            .externals(Direction::Incoming)
            .map(|node_index| {
                let node = &self.graph[node_index].clone();
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

    pub fn add_node(&mut self, node_type: SchemaNodeType) -> NodeIndex {
        let node_id = node_type.handle.id.clone();
        let node_index = self.graph.add_node(node_type);
        self.node_lookup_table.insert(node_id, node_index);
        node_index
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

    pub fn connect(
        &mut self,
        id: EdgeId,
        from: &Endpoint,
        to: &Endpoint,
        edge_kind: Option<SchemaEdgeKind>,
    ) -> EdgeIndex {
        self.connect_with_index(id, from.node, &from.port, to.node, &to.port, edge_kind)
    }

    pub fn connect_with_index(
        &mut self,
        id: EdgeId,
        from_node_index: NodeIndex,
        from_port: &Port,
        to_node_index: NodeIndex,
        to_port: &Port,
        edge_kind: Option<SchemaEdgeKind>,
    ) -> EdgeIndex {
        self.graph.add_edge(
            from_node_index,
            to_node_index,
            SchemaEdgeType::new(id.clone(), from_port.clone(), to_port.clone(), edge_kind),
        )
    }

    pub fn edges_from_endpoint<'a>(
        &'a self,
        node_id: NodeId,
        port: &'a Port,
    ) -> impl Iterator<Item = (&'a SchemaNodeType, Port)> {
        self.graph
            .edges(*self.node_index_by_node_id(node_id).unwrap())
            .filter_map(move |edge| {
                if edge.weight().from_port() == *port {
                    let node = &self.graph[edge.target()];
                    Some((node, edge.weight().to_port().clone()))
                } else {
                    None
                }
            })
    }

    pub fn add_subgraph_after_node(
        &mut self,
        node_id: NodeId,
        params: &Option<serde_json::Map<String, serde_json::Value>>,
        subgraph: &DagSchemas,
    ) {
        let Some(target_node) = self.node_index_by_node_id(node_id) else {
            return;
        };
        // Find the next node after the target node
        let mut next_nodes = self
            .graph
            .neighbors_directed(*target_node, Direction::Outgoing)
            .detach();

        let mut pre_nodes = self
            .graph
            .neighbors_directed(*target_node, Direction::Incoming)
            .detach();

        // Store the next nodes to reattach later
        let mut next_node_indices = HashMap::<NodeIndex, Vec<(EdgeIndex, SchemaEdgeType)>>::new();
        while let Some((next_edge, next_node)) = next_nodes.next(&self.graph) {
            let target_edge = &self.graph()[next_edge];
            next_node_indices
                .entry(next_node)
                .or_insert_with(Vec::new)
                .push((next_edge, target_edge.clone()));
        }
        let mut pre_node_indices = HashMap::<NodeIndex, Vec<(EdgeIndex, SchemaEdgeType)>>::new();
        while let Some((pre_node_edge, pre_node)) = pre_nodes.next(&self.graph) {
            let target_edge = &self.graph()[pre_node_edge];
            pre_node_indices
                .entry(pre_node)
                .or_insert_with(Vec::new)
                .push((pre_node_edge, target_edge.clone()));
        }

        let mut next_old_edges = HashMap::<NodeIndex, Vec<SchemaEdgeType>>::new();
        let mut remove_edges = Vec::new();
        {
            let main_graph = &self.graph;
            // Remove the existing edges from target_node to next nodes
            for (next_node, edges) in &next_node_indices {
                for (edge, _) in edges {
                    let target_edge = &main_graph[*edge];
                    next_old_edges
                        .entry(*next_node)
                        .or_insert_with(Vec::new)
                        .push(target_edge.clone());
                    remove_edges.push(*edge);
                }
            }

            for edges in pre_node_indices.values() {
                for (edge, _) in edges {
                    remove_edges.push(*edge);
                }
            }
        }
        // Add the subgraph nodes to the main graph, mapping old indices to new ones
        {
            let main_graph = &mut self.graph;
            for edge in remove_edges {
                main_graph.remove_edge(edge);
            }
            let mut new_node_map = Vec::new();
            for node in subgraph.graph.node_indices() {
                let node_type = &subgraph.graph[node];
                let pre_subgraph_nodes = &mut subgraph
                    .graph
                    .neighbors_directed(node, Direction::Incoming)
                    .detach();
                let pre_subgraph_node = pre_subgraph_nodes.next_node(&subgraph.graph);
                let mut with = HashMap::new();
                if let Some(params) = &params {
                    params.iter().for_each(|(k, v)| {
                        with.insert(k.clone(), v.clone());
                    });
                }
                if let Some(params) = &node_type.with {
                    params.iter().for_each(|(k, v)| {
                        with.insert(k.clone(), v.clone());
                    });
                }
                let node_params = node_type.with.clone();
                let node_type_action = node_type.node.action();
                let node_type = SchemaNodeType::new(
                    node_type.handle.id.clone(),
                    node_type.name.clone(),
                    node_type.node.clone(),
                    node_type.kind.clone(),
                    Some(with),
                );
                let new_node = main_graph.add_node(node_type);
                new_node_map.push((node, new_node));
                if node_type_action != INPUT_ROUTING_ACTION {
                    continue;
                }
                let Some(with) = &node_params else {
                    continue;
                };
                let Some(serde_json::Value::String(routing_port)) = with.get(ROUTING_PARAM_KEY)
                else {
                    continue;
                };
                // Because it is an Input Router of another subgraph
                if pre_subgraph_node.is_some() {
                    continue;
                }
                for (pre_node, edges) in &pre_node_indices {
                    for (_, param) in edges {
                        if param.to != Port::new(routing_port.clone()) {
                            continue;
                        }
                        main_graph.add_edge(
                            *pre_node,
                            new_node,
                            SchemaEdgeType::new(
                                param.id.clone(),
                                param.from.clone(),
                                param.to.clone(),
                                Some(SchemaEdgeKind::FromProcessor),
                            ),
                        );
                    }
                }
            }
            for edge in subgraph.graph.edge_indices() {
                let (source, target) = subgraph.graph.edge_endpoints(edge).unwrap();
                let source = new_node_map
                    .iter()
                    .find(|&&(old, _)| old == source)
                    .unwrap()
                    .1;
                let target = new_node_map
                    .iter()
                    .find(|&&(old, _)| old == target)
                    .unwrap()
                    .1;
                let edge = &subgraph.graph[edge];
                main_graph.add_edge(source, target, edge.clone());
            }

            // Connect the last nodes of the subgraph to the original next nodes
            for &(old, new) in new_node_map.iter() {
                if subgraph
                    .graph
                    .neighbors_directed(old, Direction::Outgoing)
                    .next()
                    .is_some()
                {
                    continue;
                }
                let old_node = &subgraph.graph[old];
                let Some(with) = &old_node.with else {
                    continue;
                };
                let post_subgraph_nodes = &mut subgraph
                    .graph
                    .neighbors_directed(old, Direction::Outgoing)
                    .detach();
                let post_subgraph_node = post_subgraph_nodes.next_node(&subgraph.graph);
                if old_node.node.action() != OUTPUT_ROUTING_ACTION {
                    continue;
                }
                // Because it is an Output Router of another subgraph
                if post_subgraph_node.is_some() {
                    continue;
                }
                let Some(serde_json::Value::String(routing_port)) = with.get(ROUTING_PARAM_KEY)
                else {
                    continue;
                };
                for next_node in next_node_indices.keys() {
                    let Some(old_edge) = next_old_edges
                        .get(next_node)
                        .unwrap_or_else(|| panic!("next_node not found: {next_node:?}"))
                        .iter()
                        .find(|old_edge| old_edge.from_port() == Port::new(routing_port.clone()))
                    else {
                        continue;
                    };
                    main_graph.add_edge(
                        new,
                        *next_node,
                        SchemaEdgeType::new(
                            old_edge.id.clone(),
                            old_edge.from.clone(),
                            old_edge.to.clone(),
                            Some(SchemaEdgeKind::FromProcessor),
                        ),
                    );
                }
            }
        }
    }
}

fn collect_ancestor_sources_recursive(
    dag: &DagSchemas,
    node_index: NodeIndex,
    sources: &mut HashSet<NodeHandle>,
) {
    for edge in dag.graph().edges_directed(node_index, Direction::Incoming) {
        let source_node_index = edge.source();
        let source_node = &dag.graph()[source_node_index];
        let Some(ref kind) = source_node.kind else {
            continue;
        };
        if matches!(kind, NodeKind::Source(_)) {
            sources.insert(source_node.handle.clone());
        }
        collect_ancestor_sources_recursive(dag, source_node_index, sources);
    }
}
