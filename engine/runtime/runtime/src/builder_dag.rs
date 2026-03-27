use std::{
    collections::{hash_map::Entry, HashMap},
    fmt::{Debug, Display},
    hash::Hash,
};

use petgraph::graph::NodeIndex;

use crate::{
    dag_schemas::{DagSchemas, EdgeHavePorts, SchemaEdgeKind},
    errors::ExecutionError,
    event::EventHub,
    executor_operation::NodeContext,
    node::{
        EdgeId, GraphId, NodeHandle, NodeId, NodeKind as DagNodeKind, Port, Processor, Sink,
        Source, FEATURE_FILTER_ACTION, OUTPUT_ROUTING_ACTION, ROUTING_PARAM_KEY,
    },
};

#[derive(Debug, Clone)]
pub struct NodeType {
    pub handle: NodeHandle,
    pub name: String,
    pub kind: NodeKind,
    /// Factory-declared output ports for this node.
    pub output_ports: Vec<Port>,
    /// Accumulated subgraph prefix, propagated from SchemaNodeType.
    pub subgraph_prefix: Option<String>,
}

impl Eq for NodeType {}

impl PartialEq for NodeType {
    fn eq(&self, rhs: &Self) -> bool {
        self.handle.id == rhs.handle.id
    }
}

impl NodeType {
    pub fn new(id: NodeId, name: String, kind: NodeKind) -> Self {
        Self {
            handle: NodeHandle { id },
            name,
            kind,
            output_ports: vec![],
            subgraph_prefix: None,
        }
    }
}

impl Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.handle.id)
    }
}

impl Hash for NodeType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.handle.id.to_string().hash(state);
    }
}

#[derive(Debug, Clone)]
/// Node kind, source, processor or sink. Source has a checkpoint to start from.
pub enum NodeKind {
    Source(Box<dyn Source>),
    Processor(Box<dyn Processor>),
    Sink(Box<dyn Sink>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EdgeType {
    pub id: EdgeId,
    pub from: Port,
    pub to: Port,
    pub edge_kind: SchemaEdgeKind,
}

impl EdgeType {
    pub fn new(id: EdgeId, from: Port, to: Port, edge_kind: SchemaEdgeKind) -> Self {
        Self {
            id,
            from,
            to,
            edge_kind,
        }
    }
}

impl EdgeHavePorts for EdgeType {
    fn from_port(&self) -> Port {
        self.from.to_owned()
    }

    fn to_port(&self) -> Port {
        self.to.to_owned()
    }
}

pub struct BuilderDag {
    pub(crate) id: GraphId,
    graph: petgraph::graph::DiGraph<NodeType, EdgeType>,
    event_hub: EventHub,
}

impl BuilderDag {
    pub async fn new(ctx: NodeContext, dag_schemas: DagSchemas) -> Result<Self, ExecutionError> {
        let graph_id = dag_schemas.id;
        // Collect sources that may affect a node.
        let mut affecting_sources = dag_schemas
            .graph()
            .node_indices()
            .map(|node_index| dag_schemas.collect_ancestor_sources(node_index))
            .collect::<Vec<_>>();

        // Prepare nodes and edges for consuming.
        let (nodes, edges) = dag_schemas.into_graph().into_nodes_edges();
        let mut nodes = nodes
            .into_iter()
            .map(|node| Some(node.weight))
            .collect::<Vec<_>>();

        // Build the sinks and load checkpoint.
        let mut graph = petgraph::graph::DiGraph::<NodeType, EdgeType>::new();
        let mut source_id_to_sinks = HashMap::<NodeHandle, Vec<NodeIndex>>::new();
        let mut node_index_map: HashMap<NodeIndex, NodeIndex> = HashMap::new();
        let mut source_states = HashMap::new();
        for (node_index, node) in nodes.iter_mut().enumerate() {
            if let Some(node) = node {
                let handle = node.handle.clone();
                let Some(ref kind) = node.kind else {
                    continue;
                };
                let DagNodeKind::Sink(sink) = kind.clone() else {
                    continue;
                };
                let sources = std::mem::take(&mut affecting_sources[node_index]);
                let Some(source) = sources.into_iter().next() else {
                    continue; // orphaned sink (no connected source)
                };
                let node_index = NodeIndex::new(node_index);
                if sink.name() != node.node.action() {
                    return Err(ExecutionError::ActionNameMismatch(
                        node.handle.id.to_string(),
                        sink.name().to_string(),
                        node.node.action().to_string(),
                    ));
                }
                let mut sink = sink
                    .build(
                        ctx.clone(),
                        ctx.event_hub.clone(),
                        node.node.action().to_string(),
                        node.with.clone(),
                    )
                    .map_err(|e| ExecutionError::Factory {
                        node_id: node.handle.id.to_string(),
                        node_name: node.name.clone(),
                        error: e,
                    })?;

                let state = sink.get_source_state().map_err(ExecutionError::Sink)?;
                if let Some(state) = state {
                    match source_states.entry(source.clone()) {
                        Entry::Occupied(entry) => {
                            if entry.get() != &state {
                                return Err(ExecutionError::SourceStateConflict(source));
                            }
                        }
                        Entry::Vacant(entry) => {
                            entry.insert(state);
                        }
                    }
                }

                let new_node_index = graph.add_node(NodeType {
                    handle: handle.clone(),
                    name: node.name.clone(),
                    kind: NodeKind::Sink(sink),
                    output_ports: vec![],
                    subgraph_prefix: node.subgraph_prefix.clone(),
                });
                node_index_map.insert(node_index, new_node_index);
                source_id_to_sinks
                    .entry(source)
                    .or_default()
                    .push(new_node_index);
            }
        }

        // Build sources, processors, and collect source states.
        for (node_index, node) in nodes.iter_mut().enumerate() {
            let Some(node) = node.take() else {
                continue;
            };
            let node_index = NodeIndex::new(node_index);
            let Some(ref kind) = node.kind else {
                continue;
            };
            let node = match kind {
                DagNodeKind::Source(source) => {
                    if source.name() != node.node.action() {
                        return Err(ExecutionError::ActionNameMismatch(
                            node.handle.id.to_string(),
                            source.name().to_string(),
                            node.node.action().to_string(),
                        ));
                    }
                    let output_ports = source.get_output_ports();
                    let source = source
                        .build(
                            ctx.clone(),
                            ctx.event_hub.clone(),
                            node.node.action().to_string(),
                            node.with.clone(),
                            source_states.remove(&node.handle),
                        )
                        .map_err(|e| ExecutionError::Factory {
                            node_id: node.handle.id.to_string(),
                            node_name: node.name.clone(),
                            error: e,
                        })?;

                    // Write state to relevant sink.
                    let state = source
                        .serialize_state()
                        .await
                        .map_err(ExecutionError::Source)?;
                    for sink in source_id_to_sinks.remove(&node.handle).unwrap_or_default() {
                        let sink = &mut graph[sink];
                        let NodeKind::Sink(sink) = &mut sink.kind else {
                            unreachable!()
                        };
                        sink.set_source_state(&state)
                            .map_err(ExecutionError::Sink)?;
                    }

                    NodeType {
                        handle: node.handle,
                        name: node.name,
                        kind: NodeKind::Source(source),
                        output_ports,
                        subgraph_prefix: node.subgraph_prefix,
                    }
                }
                DagNodeKind::Processor(processor) => {
                    if processor.name() != node.node.action() {
                        return Err(ExecutionError::ActionNameMismatch(
                            node.handle.id.to_string(),
                            processor.name().to_string(),
                            node.node.action().to_string(),
                        ));
                    }
                    let mut output_ports = processor.get_output_ports();
                    // IMPORTANT: Dynamic port extraction for actions whose output
                    // ports are determined by configuration, not by
                    // get_output_ports(). These are handled as ad-hoc special cases
                    // here rather than adding a new trait method (e.g.
                    // `ProcessorFactory::build_output_ports(with)`) because:
                    //   1. Such a method would require an empty default impl to
                    //      avoid mechanical changes across ~100 factories, but an
                    //      empty default lets developers forget to implement it for
                    //      new dynamic-port actions — silently dropping ports.
                    //      Such fragile approach should not be committed to the core
                    //      without a more robust architectural decision.
                    //   2. The existing get_output_ports() serves schema generation
                    //      where no `with` is available, and adding a second method
                    //      with overlapping semantics risks premature commitment.
                    // An architectural revision of port handling is needed to
                    // resolve this ad-hoc approach properly.
                    if let Some(ref with) = node.with {
                        let action = node.node.action();
                        if action == OUTPUT_ROUTING_ACTION {
                            // OutputRouter declares no output ports; the actual
                            // port is specified via the routingPort parameter.
                            if let Some(serde_json::Value::String(rp)) = with.get(ROUTING_PARAM_KEY)
                            {
                                output_ports.push(Port::new(rp));
                            }
                        } else if action == FEATURE_FILTER_ACTION {
                            // FeatureFilter declares only the static "unfiltered"
                            // port; dynamic ports come from conditions[].outputPort.
                            if let Some(serde_json::Value::Array(conditions)) =
                                with.get("conditions")
                            {
                                for condition in conditions {
                                    if let Some(serde_json::Value::String(port)) =
                                        condition.get("outputPort")
                                    {
                                        output_ports.push(Port::new(port));
                                    }
                                }
                            }
                        }
                    }
                    let processor = processor
                        .build(
                            ctx.clone(),
                            ctx.event_hub.clone(),
                            node.node.action().to_string(),
                            node.with.clone(),
                        )
                        .map_err(|e| ExecutionError::Factory {
                            node_id: node.handle.id.to_string(),
                            node_name: node.name.clone(),
                            error: e,
                        })?;
                    NodeType {
                        handle: node.handle,
                        name: node.name,
                        kind: NodeKind::Processor(processor),
                        output_ports,
                        subgraph_prefix: node.subgraph_prefix,
                    }
                }
                DagNodeKind::Sink(_) => continue,
            };
            let new_node_index = graph.add_node(node);
            node_index_map.insert(node_index, new_node_index);
        }

        // Connect the edges, skipping any that involve nodes not added to the graph
        // (e.g. orphaned sinks or nodes connected only to orphaned sinks).
        for edge in edges {
            let (Some(&src), Some(&dst)) = (
                node_index_map.get(&edge.source()),
                node_index_map.get(&edge.target()),
            ) else {
                continue;
            };
            graph.add_edge(
                src,
                dst,
                EdgeType::new(
                    edge.weight.id.clone(),
                    edge.weight.from,
                    edge.weight.to,
                    edge.weight.edge_kind.unwrap(),
                ),
            );
        }

        Ok(BuilderDag {
            id: graph_id,
            graph,
            event_hub: ctx.event_hub.clone(),
        })
    }

    pub fn graph(&self) -> &petgraph::graph::DiGraph<NodeType, EdgeType> {
        &self.graph
    }

    pub fn into_graph_and_event_hub(
        self,
    ) -> (petgraph::graph::DiGraph<NodeType, EdgeType>, EventHub) {
        (self.graph, self.event_hub)
    }
}
