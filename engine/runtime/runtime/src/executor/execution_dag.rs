use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};

use reearth_flow_common::collection::insert_vec_element;
use reearth_flow_state::State;

use crate::{
    builder_dag::{BuilderDag, NodeKind},
    dag_schemas::{EdgeHavePorts, SchemaEdgeKind},
    errors::ExecutionError,
    event::EventHub,
    executor_operation::ExecutorOperation,
    feature_store::{create_feature_writer, FeatureWriter},
    forwarder::SenderWithPortMapping,
    node::{EdgeId, GraphId, NodeHandle, Port},
};
use crossbeam::channel::{bounded, Receiver, Sender};
use petgraph::graph::NodeIndex;
use petgraph::{visit::EdgeRef, Direction};

#[derive(Debug)]
pub struct NodeType {
    pub handle: NodeHandle,
    pub name: String,
    pub kind: Option<NodeKind>,
    // Persisted here because `kind` is take()n during execution graph construction and becomes unavailable later.
    pub is_source: bool,
    pub output_ports: Vec<Port>,
    pub subgraph_prefix: Option<String>,
    pub is_subgraph_output: bool,
    pub action: String,
}

impl NodeType {
    // Mirrors builder_dag::NodeType::composed_id exactly (duplicated, not shared) — keep the two formats in sync.
    pub fn composed_id(&self) -> String {
        match &self.subgraph_prefix {
            Some(prefix) => format!("{prefix}.{}", self.handle.id),
            None => self.handle.id.to_string(),
        }
    }
}

#[derive(Clone)]
pub struct EdgeType {
    pub edge_id: EdgeId,
    pub output_port: Port,
    pub edge_kind: SchemaEdgeKind,
    pub sender: Sender<ExecutorOperation>,
    pub input_port: Port,
    pub receiver: Receiver<ExecutorOperation>,
}

pub struct ExecutionDag {
    pub(crate) id: GraphId,
    pub(crate) executor_id: uuid::Uuid,
    graph: petgraph::graph::DiGraph<NodeType, EdgeType>,
    event_hub: EventHub,
    ingress_state: Arc<State>,
    port_writers: HashMap<NodeIndex, HashMap<Port, Box<dyn FeatureWriter>>>,
}

impl ExecutionDag {
    pub fn new(
        builder_dag: BuilderDag,
        channel_buffer_sz: usize,
        feature_flush_threshold: usize,
        ingress_state: Arc<State>,
        feature_state: Arc<State>,
        executor_id: uuid::Uuid,
    ) -> Result<Self, ExecutionError> {
        let graph_id = builder_dag.id;
        let mut channels = HashMap::<
            (petgraph::graph::NodeIndex, petgraph::graph::NodeIndex),
            (Sender<ExecutorOperation>, Receiver<ExecutorOperation>),
        >::new();

        let mut edges = vec![];
        for builder_dag_edge in builder_dag.graph().raw_edges().iter() {
            let source_node_index = builder_dag_edge.source();
            let target_node_index = builder_dag_edge.target();
            let edge = &builder_dag_edge.weight;
            let edge_id = edge.id.clone();
            let output_port = edge.to_port();
            let edge_kind = edge.edge_kind.clone();

            let (sender, receiver) = match channels.entry((source_node_index, target_node_index)) {
                Entry::Vacant(entry) => {
                    let (sender, receiver) = bounded(channel_buffer_sz);
                    entry.insert((sender.clone(), receiver.clone()));
                    (sender, receiver)
                }
                Entry::Occupied(entry) => entry.get().clone(),
            };

            let edge = EdgeType {
                edge_id,
                output_port,
                edge_kind,
                sender,
                input_port: edge.from_port().clone(),
                receiver,
            };
            edges.push(Some(edge));
        }

        let (graph, event_hub) = builder_dag.into_graph_and_event_hub();
        let graph = graph.map(
            |_, node| NodeType {
                handle: node.handle.clone(),
                name: node.name.clone(),
                is_source: matches!(node.kind, NodeKind::Source(_)),
                kind: match &node.kind {
                    NodeKind::Source(source) => Some(NodeKind::Source(source.clone())),
                    NodeKind::Processor(processor) => Some(NodeKind::Processor(processor.clone())),
                    NodeKind::Sink(sink) => Some(NodeKind::Sink(sink.clone())),
                },
                output_ports: node.output_ports.clone(),
                subgraph_prefix: node.subgraph_prefix.clone(),
                is_subgraph_output: node.is_subgraph_output,
                action: node.action.clone(),
            },
            |edge_index, _| {
                edges[edge_index.index()]
                    .take()
                    .expect("We created all edges")
            },
        );
        let mut port_writers = HashMap::new();
        for node_index in graph.node_indices() {
            let node = &graph[node_index];
            let mut node_port_writers = HashMap::new();
            for port in &node.output_ports {
                let file_id = match (&node.subgraph_prefix, node.is_subgraph_output) {
                    (Some(prefix), true) => format!("{}.{}", prefix, port),
                    (Some(prefix), false) => format!("{}.{}.{}", prefix, node.handle.id, port),
                    (None, _) => format!("{}.{}", node.handle.id, port),
                };
                let writer = create_feature_writer(
                    EdgeId::new(file_id),
                    Arc::clone(&feature_state),
                    feature_flush_threshold,
                );
                node_port_writers.insert(port.clone(), writer);
            }
            if !node_port_writers.is_empty() {
                port_writers.insert(node_index, node_port_writers);
            }
        }

        Ok(ExecutionDag {
            id: graph_id,
            executor_id,
            graph,
            event_hub,
            ingress_state,
            port_writers,
        })
    }

    pub fn graph(&self) -> &petgraph::graph::DiGraph<NodeType, EdgeType> {
        &self.graph
    }

    pub fn executor_id(&self) -> uuid::Uuid {
        self.executor_id
    }

    pub fn node_weight_mut(&mut self, node_index: petgraph::graph::NodeIndex) -> &mut NodeType {
        &mut self.graph[node_index]
    }

    pub fn event_hub(&self) -> &EventHub {
        &self.event_hub
    }

    pub fn feature_state(&self) -> Arc<State> {
        Arc::clone(&self.ingress_state)
    }

    pub fn ingress_state(&self) -> &Arc<State> {
        &self.ingress_state
    }

    pub fn collect_senders(
        &self,
        node_index: petgraph::graph::NodeIndex,
    ) -> Vec<SenderWithPortMapping> {
        let mut senders = HashMap::<petgraph::graph::NodeIndex, SenderWithPortMapping>::new();
        for edge in self.graph.edges(node_index) {
            match senders.entry(edge.target()) {
                Entry::Vacant(entry) => {
                    let port_mapping = [(
                        edge.weight().input_port.clone(),
                        vec![edge.weight().output_port.clone()],
                    )]
                    .into_iter()
                    .collect();
                    entry.insert(SenderWithPortMapping {
                        sender: edge.weight().sender.clone(),
                        port_mapping,
                    });
                }
                Entry::Occupied(mut entry) => {
                    insert_vec_element(
                        &mut entry.get_mut().port_mapping,
                        edge.weight().input_port.clone(),
                        edge.weight().output_port.clone(),
                    );
                }
            }
        }
        senders.into_values().collect()
    }

    pub fn collect_port_writers(
        &self,
        node_index: petgraph::graph::NodeIndex,
    ) -> HashMap<Port, Box<dyn FeatureWriter>> {
        self.port_writers
            .get(&node_index)
            .cloned()
            .unwrap_or_default()
    }

    pub fn collect_receivers(
        &self,
        node_index: petgraph::graph::NodeIndex,
    ) -> (Vec<NodeHandle>, Vec<Receiver<ExecutorOperation>>) {
        let mut handles_and_receivers =
            HashMap::<petgraph::graph::NodeIndex, (NodeHandle, Receiver<ExecutorOperation>)>::new();
        for edge in self.graph.edges_directed(node_index, Direction::Incoming) {
            let source_node_index = edge.source();
            if let Entry::Vacant(entry) = handles_and_receivers.entry(source_node_index) {
                entry.insert((
                    self.graph[source_node_index].handle.clone(),
                    edge.weight().receiver.clone(),
                ));
            }
        }
        handles_and_receivers.into_values().unzip()
    }
}
