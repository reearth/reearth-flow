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
    /// Accumulated dotted prefix for subgraph nodes, e.g. "G1_id.G2_id".
    /// `None` for top-level nodes.
    pub subgraph_prefix: Option<String>,
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
        subgraph_prefix: Option<String>,
    ) -> Self {
        Self {
            handle: NodeHandle::new(id),
            name,
            node,
            kind,
            with,
            subgraph_prefix,
        }
    }
}

/// Prepend `parent_id` to every node's `subgraph_prefix` inside a subgraph.
fn prepend_subgraph_prefix(graph: &mut DiGraph<SchemaNodeType, SchemaEdgeType>, parent_id: &str) {
    for node_weight in graph.node_weights_mut() {
        node_weight.subgraph_prefix = Some(match &node_weight.subgraph_prefix {
            Some(existing) => format!("{}.{}", parent_id, existing),
            None => parent_id.to_string(),
        });
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

#[derive(Clone)]
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
    ) -> Result<Self, crate::errors::ExecutionError> {
        let entry_graph = graphs
            .iter()
            .find(|dag| dag.id == entry_graph_id)
            .unwrap_or_else(|| panic!("Entry graph not found. with id = {entry_graph_id}"));
        let graphs_by_id: HashMap<_, _> = graphs.iter().map(|graph| (graph.id, graph)).collect();

        let mut dag = DagSchemas::from_graph(entry_graph, &factories, &global_params);

        // Expand subgraphs top-down.
        const MAX_EXPANSION_DEPTH: usize = 1000;
        for _ in 0..MAX_EXPANSION_DEPTH {
            let found = dag.graph.node_indices().find_map(|idx| {
                let node = &dag.graph[idx];
                if let Node::SubGraph {
                    sub_graph_id,
                    entity,
                } = &node.node
                {
                    Some((
                        idx,
                        *sub_graph_id,
                        entity.id,
                        entity.with.clone(),
                        node.subgraph_prefix.clone(),
                    ))
                } else {
                    None
                }
            });
            let Some((target_idx, sub_graph_id, entity_id, entity_with, parent_prefix)) = found
            else {
                break;
            };

            let subgraph_def = graphs_by_id
                .get(&sub_graph_id)
                .unwrap_or_else(|| panic!("Subgraph not found. with id = {sub_graph_id}"));

            let params = if let Some(with) = &entity_with {
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

            let mut subgraph = DagSchemas::from_graph(subgraph_def, &factories, &params);

            // Accumulated prefix: inherit parent's prefix + this entity's id.
            let entity_id_str = entity_id.to_string();
            let full_prefix = match &parent_prefix {
                Some(p) => format!("{}.{}", p, entity_id_str),
                None => entity_id_str,
            };

            for edge in subgraph.graph.edge_weights_mut() {
                edge.id = EdgeId::new(format!("{}.{}", full_prefix, edge.id));
            }
            prepend_subgraph_prefix(&mut subgraph.graph, &full_prefix);
            dag.add_subgraph_after_node(target_idx, &params, &subgraph);
            dag.graph.remove_node(target_idx);
        }

        // Verify all subgraphs were fully expanded to check for a possible cycle or
        // if the expansion limit was exceeded.
        let remaining: Vec<_> = dag
            .graph
            .node_weights()
            .filter_map(|n| match &n.node {
                Node::SubGraph { sub_graph_id, .. } => Some(sub_graph_id.to_string()),
                _ => None,
            })
            .collect();
        if !remaining.is_empty() {
            return Err(crate::errors::ExecutionError::SubgraphCycle {
                max_iterations: MAX_EXPANSION_DEPTH,
                ids: remaining,
            });
        }

        Ok(dag)
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
                None,
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

    /// For every node, the set of `Source` nodes that are its strict ancestors,
    /// indexed by `NodeIndex::index()`.
    ///
    /// Single iterative pass in topological order (mirrors
    /// `schema_infer::infer_and_validate`). Replaces the former per-node
    /// recursive `collect_ancestor_sources`, which recursed to depth = ancestor
    /// chain length (stack overflow on long workflows) and cost O(N^2). A
    /// predecessor whose `kind` is `None` is a barrier: it and its ancestors are
    /// excluded, exactly as the old recursive `continue`.
    pub fn affecting_sources_per_node(
        &self,
    ) -> Result<Vec<HashSet<NodeHandle>>, crate::errors::ExecutionError> {
        let graph = self.graph();
        let order = petgraph::algo::toposort(graph, None)
            .map_err(|_| crate::errors::ExecutionError::SchemaInferenceCycle)?;

        let mut result: Vec<HashSet<NodeHandle>> = vec![HashSet::new(); graph.node_count()];
        for node_index in order {
            let mut acc: HashSet<NodeHandle> = HashSet::new();
            for edge in graph.edges_directed(node_index, Direction::Incoming) {
                let pred_index = edge.source();
                let pred = &graph[pred_index];
                let Some(ref kind) = pred.kind else {
                    continue; // kind == None barrier
                };
                acc.extend(result[pred_index.index()].iter().cloned());
                if matches!(kind, NodeKind::Source(_)) {
                    acc.insert(pred.handle.clone());
                }
            }
            result[node_index.index()] = acc;
        }
        Ok(result)
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

    pub fn add_node(&mut self, node_type: SchemaNodeType) -> NodeIndex {
        let node_id = node_type.handle.id.clone();
        let node_index = self.graph.add_node(node_type);
        self.node_lookup_table.insert(node_id, node_index);
        node_index
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

    pub fn add_subgraph_after_node(
        &mut self,
        target_node: NodeIndex,
        params: &Option<serde_json::Map<String, serde_json::Value>>,
        subgraph: &DagSchemas,
    ) {
        // Find the next node after the target node
        let mut next_nodes = self
            .graph
            .neighbors_directed(target_node, Direction::Outgoing)
            .detach();

        let mut pre_nodes = self
            .graph
            .neighbors_directed(target_node, Direction::Incoming)
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
                    node_type.subgraph_prefix.clone(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    use reearth_flow_types::workflow::{Node, NodeEntity};
    use uuid::Uuid;

    use crate::event::EventHub;
    use crate::executor_operation::NodeContext;
    use crate::node::{Processor, ProcessorFactory, Source, SourceFactory, DEFAULT_PORT};

    #[derive(Debug, Clone)]
    struct StubSource;
    impl SourceFactory for StubSource {
        fn name(&self) -> &str {
            "StubSource"
        }
        fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
            None
        }
        fn get_output_ports(&self) -> Vec<Port> {
            vec![DEFAULT_PORT.clone()]
        }
        fn build(
            &self,
            _ctx: NodeContext,
            _event_hub: EventHub,
            _action: String,
            _with: Option<HashMap<String, serde_json::Value>>,
            _state: Option<Vec<u8>>,
        ) -> Result<Box<dyn Source>, crate::errors::BoxedError> {
            Err("stub".into())
        }
    }

    #[derive(Debug, Clone)]
    struct StubProc;
    impl ProcessorFactory for StubProc {
        fn name(&self) -> &str {
            "StubProc"
        }
        fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
            None
        }
        fn get_input_ports(&self) -> Vec<Port> {
            vec![DEFAULT_PORT.clone()]
        }
        fn get_output_ports(&self) -> Vec<Port> {
            vec![DEFAULT_PORT.clone()]
        }
        fn build(
            &self,
            _ctx: NodeContext,
            _event_hub: EventHub,
            _action: String,
            _with: Option<HashMap<String, serde_json::Value>>,
        ) -> Result<Box<dyn Processor>, crate::errors::BoxedError> {
            unreachable!()
        }
    }

    fn dummy_node(name: &str) -> Node {
        Node::Action {
            entity: NodeEntity {
                id: Uuid::new_v4(),
                name: name.to_string(),
                with: None,
            },
            action: name.to_string(),
        }
    }

    fn empty_dag() -> DagSchemas {
        let g = reearth_flow_types::workflow::Graph {
            id: Uuid::new_v4(),
            name: "t".to_string(),
            nodes: vec![],
            edges: vec![],
        };
        DagSchemas::from_graph(&g, &HashMap::new(), &None)
    }

    enum K {
        Source,
        Proc,
        None_,
    }

    fn add(dag: &mut DagSchemas, name: &str, k: K) -> NodeIndex {
        let kind = match k {
            K::Source => Some(NodeKind::Source(Box::new(StubSource))),
            K::Proc => Some(NodeKind::Processor(Box::new(StubProc))),
            K::None_ => None,
        };
        dag.add_node(SchemaNodeType::new(
            NodeId::new(Uuid::new_v4().to_string()),
            name.to_string(),
            dummy_node(name),
            kind,
            None,
            None,
        ))
    }

    fn link(dag: &mut DagSchemas, from: NodeIndex, to: NodeIndex) {
        dag.connect_with_index(
            EdgeId::new(Uuid::new_v4().to_string()),
            from,
            &DEFAULT_PORT,
            to,
            &DEFAULT_PORT,
            None,
        );
    }

    fn reference(dag: &DagSchemas, node: NodeIndex, out: &mut HashSet<NodeHandle>) {
        for edge in dag.graph().edges_directed(node, Direction::Incoming) {
            let src = edge.source();
            let n = &dag.graph()[src];
            let Some(ref kind) = n.kind else { continue };
            if matches!(kind, NodeKind::Source(_)) {
                out.insert(n.handle.clone());
            }
            reference(dag, src, out);
        }
    }

    fn reference_all(dag: &DagSchemas) -> Vec<HashSet<NodeHandle>> {
        (0..dag.graph().node_count())
            .map(|i| {
                let mut s = HashSet::new();
                reference(dag, NodeIndex::new(i), &mut s);
                s
            })
            .collect()
    }

    #[test]
    fn chain_collects_single_source() {
        let mut dag = empty_dag();
        let s = add(&mut dag, "s", K::Source);
        let a = add(&mut dag, "a", K::Proc);
        let b = add(&mut dag, "b", K::Proc);
        link(&mut dag, s, a);
        link(&mut dag, a, b);
        let got = dag.affecting_sources_per_node().unwrap();
        let s_handle = dag.graph()[s].handle.clone();
        assert!(got[s.index()].is_empty());
        assert_eq!(got[a.index()], HashSet::from([s_handle.clone()]));
        assert_eq!(got[b.index()], HashSet::from([s_handle]));
    }

    #[test]
    fn diamond_unions_sources() {
        let mut dag = empty_dag();
        let s1 = add(&mut dag, "s1", K::Source);
        let s2 = add(&mut dag, "s2", K::Source);
        let a = add(&mut dag, "a", K::Proc);
        let b = add(&mut dag, "b", K::Proc);
        let c = add(&mut dag, "c", K::Proc);
        link(&mut dag, s1, a);
        link(&mut dag, a, c);
        link(&mut dag, s2, b);
        link(&mut dag, b, c);
        let got = dag.affecting_sources_per_node().unwrap();
        let h1 = dag.graph()[s1].handle.clone();
        let h2 = dag.graph()[s2].handle.clone();
        assert_eq!(got[c.index()], HashSet::from([h1, h2]));
    }

    #[test]
    fn none_kind_predecessor_is_a_barrier() {
        let mut dag = empty_dag();
        let s = add(&mut dag, "s", K::Source);
        let bar = add(&mut dag, "bar", K::None_);
        let c = add(&mut dag, "c", K::Proc);
        link(&mut dag, s, bar);
        link(&mut dag, bar, c);
        let got = dag.affecting_sources_per_node().unwrap();
        assert!(got[c.index()].is_empty());
    }

    #[test]
    fn differential_matches_reference_on_many_shapes() {
        for shape in 0..4u8 {
            let mut dag = empty_dag();
            let s1 = add(&mut dag, "s1", K::Source);
            let p1 = add(&mut dag, "p1", K::Proc);
            link(&mut dag, s1, p1);
            if shape >= 1 {
                let s2 = add(&mut dag, "s2", K::Source);
                link(&mut dag, s2, p1);
            }
            if shape >= 2 {
                let p2 = add(&mut dag, "p2", K::Proc);
                link(&mut dag, p1, p2);
            }
            if shape >= 3 {
                let bar = add(&mut dag, "bar", K::None_);
                let z = add(&mut dag, "z", K::Proc);
                link(&mut dag, s1, bar);
                link(&mut dag, bar, z);
            }
            let got = dag.affecting_sources_per_node().unwrap();
            let want = reference_all(&dag);
            assert_eq!(got, want, "iterative != reference for shape {shape}");
        }
    }

    #[test]
    fn cycle_returns_schema_inference_cycle() {
        let mut dag = empty_dag();
        let a = add(&mut dag, "a", K::Proc);
        let b = add(&mut dag, "b", K::Proc);
        link(&mut dag, a, b);
        link(&mut dag, b, a);
        assert!(matches!(
            dag.affecting_sources_per_node(),
            Err(crate::errors::ExecutionError::SchemaInferenceCycle)
        ));
    }
}
