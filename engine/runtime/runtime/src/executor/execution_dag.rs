use std::{
    collections::{hash_map::Entry, HashMap},
    fmt::Debug,
    sync::Arc,
};

use reearth_flow_common::collection::insert_vec_element;
use reearth_flow_state::State;
use tokio::sync::Mutex;

use crate::{
    builder_dag::{BuilderDag, NodeKind},
    dag_schemas::{EdgeHavePorts, SchemaEdgeKind},
    errors::ExecutionError,
    event::EventHub,
    executor_operation::ExecutorOperation,
    feature_store::{create_feature_writer, FeatureWriter, FeatureWriterKey},
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
    /// Snapshot of the node role (e.g., Source/Processor/Sink).
    /// Although this is derivable from `kind`, we persist it here because `kind` is moved (`take()`) during execution graph construction (e.g., in ProcessorNode::new),
    /// making it unavailable later. Keeping this immutable snapshot avoids timing issues.
    /// TODO: refactor to remove duplication once initialization no longer requires taking `kind`.
    pub is_source: bool,
    /// Factory-declared output ports, propagated from BuilderDag.
    pub output_ports: Vec<Port>,
    /// Accumulated subgraph prefix, propagated from BuilderDag.
    pub subgraph_prefix: Option<String>,
    /// True for OutputRouter nodes inside subgraphs: port-based intermediate
    /// data is named `<prefix>.<port>` rather than `<prefix>.<node_id>.<port>`.
    pub is_subgraph_output: bool,
}

type SharedFeatureWriter = Arc<Mutex<Box<dyn FeatureWriter>>>;

#[derive(Clone)]
pub struct EdgeType {
    /// Edge ID.
    pub edge_id: EdgeId,
    /// Output port handle.
    pub output_port: Port,
    /// Edge kind.
    pub edge_kind: SchemaEdgeKind,
    /// The sender for data flowing downstream. Edges that have same source and target node share the same sender.
    pub sender: Sender<ExecutorOperation>,
    /// The record writer for persisting data for downstream queries, if persistency is needed. Different edges with the same output port share the same record writer.
    pub feature_writer: SharedFeatureWriter,
    /// Input port handle.
    pub input_port: Port,
    /// The receiver from receiving data from upstream. Edges that have same source and target node share the same receiver.
    pub receiver: Receiver<ExecutorOperation>,
}

pub struct ExecutionDag {
    pub(crate) id: GraphId,
    /// Unique identifier for this workflow execution, used for cache isolation.
    pub(crate) executor_id: uuid::Uuid,
    /// Nodes will be moved into execution threads.
    graph: petgraph::graph::DiGraph<NodeType, EdgeType>,
    event_hub: EventHub,
    ingress_state: Arc<State>,
    /// Port-based feature writers: one writer per (node, output_port) pair.
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
        // We only create record writer once for every output port. Every `HashMap` in this `Vec` tracks if a node's output ports already have the record writer created.
        let mut all_feature_writers = vec![
            HashMap::<Port, Vec<SharedFeatureWriter>>::new();
            builder_dag.graph().node_count()
        ];
        // We only create channel once for every pair of nodes.
        let mut channels = HashMap::<
            (petgraph::graph::NodeIndex, petgraph::graph::NodeIndex),
            (Sender<ExecutorOperation>, Receiver<ExecutorOperation>),
        >::new();

        // Create new edges.
        let mut edges = vec![];
        for builder_dag_edge in builder_dag.graph().raw_edges().iter() {
            let source_node_index = builder_dag_edge.source();
            let target_node_index = builder_dag_edge.target();
            let edge = &builder_dag_edge.weight;
            let edge_id = edge.id.clone();
            let output_port = edge.to_port();
            let input_port = edge.from_port();
            let edge_kind = edge.edge_kind.clone();

            // Create or get feature writer.
            let feature_writer =
                match all_feature_writers[source_node_index.index()].entry(input_port.clone()) {
                    Entry::Vacant(entry) => {
                        let feature_writer = create_feature_writer(
                            edge_id.clone(),
                            Arc::clone(&feature_state),
                            feature_flush_threshold,
                        );
                        let feature_writer = Arc::new(Mutex::new(feature_writer));
                        entry.insert(vec![feature_writer.clone()]);
                        feature_writer
                    }
                    Entry::Occupied(mut entry) => {
                        let feature_writer = create_feature_writer(
                            edge_id.clone(),
                            Arc::clone(&feature_state),
                            feature_flush_threshold,
                        );
                        let feature_writer = Arc::new(Mutex::new(feature_writer));
                        entry.get_mut().push(feature_writer.clone());
                        feature_writer
                    }
                };

            // Create or get channel.
            let (sender, receiver) = match channels.entry((source_node_index, target_node_index)) {
                Entry::Vacant(entry) => {
                    let (sender, receiver) = bounded(channel_buffer_sz);
                    entry.insert((sender.clone(), receiver.clone()));
                    (sender, receiver)
                }
                Entry::Occupied(entry) => entry.get().clone(),
            };

            // Create edge.
            let edge = EdgeType {
                edge_id,
                output_port,
                edge_kind,
                sender,
                feature_writer,
                input_port: edge.from_port().clone(),
                receiver,
            };
            edges.push(Some(edge));
        }

        // Create new graph.
        let (graph, event_hub) = builder_dag.into_graph_and_event_hub();
        let graph = graph.map(
            |_, node| NodeType {
                handle: node.handle.clone(),
                name: node.name.clone(),
                // Persist role early. `kind` will be taken later (e.g., in ProcessorNode::new),
                // so we cannot reliably derive the role at that time.
                is_source: matches!(node.kind, NodeKind::Source(_)),
                kind: match &node.kind {
                    NodeKind::Source(source) => Some(NodeKind::Source(source.clone())),
                    NodeKind::Processor(processor) => Some(NodeKind::Processor(processor.clone())),
                    NodeKind::Sink(sink) => Some(NodeKind::Sink(sink.clone())),
                },
                output_ports: node.output_ports.clone(),
                subgraph_prefix: node.subgraph_prefix.clone(),
                is_subgraph_output: node.is_subgraph_output,
            },
            |edge_index, _| {
                edges[edge_index.index()]
                    .take()
                    .expect("We created all edges")
            },
        );
        // Create port-based writers: one writer per (node, output_port) pair.
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
        // Map from target node index to `SenderWithPortMapping`.
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

    pub async fn collect_record_writers(
        &self,
        node_index: petgraph::graph::NodeIndex,
    ) -> HashMap<FeatureWriterKey, Vec<Box<dyn FeatureWriter>>> {
        let mut feature_writers = HashMap::<FeatureWriterKey, Vec<Box<dyn FeatureWriter>>>::new();

        // Check if this node is a Source (Reader)
        let is_source_node = self.graph[node_index].is_source;

        for edge in self.graph.edges(node_index) {
            let weight = edge.weight();

            // Skip creating feature_writers for Source→Processor edges
            // ProcessorNode handles Reader intermediate data writes directly
            if is_source_node {
                continue;
            }

            // Note: Despite the confusing names, weight.input_port is actually the SOURCE output port
            // and weight.output_port is actually the DOWNSTREAM input port (see lines 91-92 where they're swapped)
            let writer_key =
                FeatureWriterKey(weight.input_port.clone(), weight.output_port.clone());
            let edge_type = self
                .graph
                .edge_weight(edge.id())
                .expect("We don't modify graph structure, only modify the edge weight");
            match feature_writers.entry(writer_key) {
                Entry::Vacant(entry) => {
                    let record_writer = edge_type.feature_writer.lock().await;
                    entry.insert(vec![record_writer.clone()]);
                }
                Entry::Occupied(mut entry) => {
                    let record_writer = edge_type.feature_writer.lock().await;
                    entry.get_mut().push(record_writer.clone());
                }
            }
        }
        feature_writers
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
        // Map from source node index to source node handle and the receiver to receiver from source.
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
