use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Debug;
use std::sync::Arc;
use std::thread::Builder;
use std::thread::JoinHandle;
use std::time::Duration;

use crossbeam::channel::Sender;
use futures::Future;
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use reearth_flow_eval_expr::engine::Engine;
use reearth_flow_state::State;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::workflow::Graph;
use tokio::runtime::Handle;
use tokio::sync::broadcast::Receiver;
use tokio::sync::Notify;

use super::node::Node;
use super::processor_node::ProcessorNode;
use super::sink_node::SinkNode;
use crate::builder_dag::{BuilderDag, NodeKind};
use crate::dag_schemas::DagSchemas;
use crate::errors::ExecutionError;
use crate::event::{Event, EventHandler, EventHub};
use crate::executor_operation::{ExecutorOperation, ExecutorOptions, NodeContext};
use crate::incremental::IncrementalRunConfig;
use crate::kvs::KvStore;
use crate::node::{EdgeId, NodeId, Port};

use super::execution_dag::ExecutionDag;
use super::source_node::{create_source_node, SourceNode};

pub struct DagExecutor {
    builder_dag: BuilderDag,
    options: ExecutorOptions,
}

pub struct DagExecutorJoinHandle {
    event_hub: EventHub,
    join_handles: Vec<JoinHandle<Result<(), ExecutionError>>>,
    notify: Arc<Notify>,
}

impl DagExecutor {
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        expr_engine: Arc<Engine>,
        storage_resolver: Arc<StorageResolver>,
        kv_store: Arc<dyn KvStore>,
        entry_graph_id: uuid::Uuid,
        graphs: Vec<Graph>,
        options: ExecutorOptions,
        factories: HashMap<String, crate::node::NodeKind>,
        global_params: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<Self, ExecutionError> {
        let dag_schemas = DagSchemas::from_graphs(entry_graph_id, graphs, factories, global_params);
        let event_hub = EventHub::new(options.event_hub_capacity);
        let ctx = NodeContext::new(expr_engine, storage_resolver, kv_store, event_hub);
        let builder_dag = BuilderDag::new(ctx, dag_schemas).await?;
        Ok(Self {
            builder_dag,
            options,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn start<F: Send + 'static + Future + Unpin + Debug + Clone>(
        self,
        shutdown: F,
        runtime: Arc<Handle>,
        expr_engine: Arc<Engine>,
        storage_resolver: Arc<StorageResolver>,
        kv_store: Arc<dyn crate::kvs::KvStore>,
        ingress_state: Arc<State>,
        feature_state: Arc<State>,
        incremental_run_config: Option<IncrementalRunConfig>,
        event_handlers: Vec<Arc<dyn EventHandler>>,
    ) -> Result<DagExecutorJoinHandle, ExecutionError> {
        // Construct execution dag.
        let mut execution_dag = ExecutionDag::new(
            self.builder_dag,
            self.options.channel_buffer_sz,
            self.options.feature_flush_threshold,
            Arc::clone(&ingress_state),
            Arc::clone(&feature_state),
        )?;
        let execute_node_ids: HashSet<NodeId> = if let Some(cfg) = &incremental_run_config {
            collect_executable_node_ids(&execution_dag, cfg)?
        } else {
            execution_dag
                .graph()
                .node_indices()
                .map(|i| execution_dag.graph()[i].handle.id.clone())
                .collect()
        };
        let node_indexes = execution_dag.graph().node_indices().collect::<Vec<_>>();

        let event_hub = execution_dag.event_hub().clone();

        let ctx = NodeContext::new(
            Arc::clone(&expr_engine),
            Arc::clone(&storage_resolver),
            Arc::clone(&kv_store),
            execution_dag.event_hub().clone(),
        );

        let should_run_sources = execution_dag.graph().node_indices().any(|i| {
            execution_dag.graph()[i].is_source
                && execute_node_ids.contains(&execution_dag.graph()[i].handle.id)
        });

        let mut join_handles = vec![];
        let mut receiver = execution_dag.event_hub().sender.subscribe();
        let notify = Arc::new(Notify::new());
        let notify_publish = Arc::clone(&notify);
        let notify_subscribe = Arc::clone(&notify);
        runtime.spawn(async move {
            subscribe_event(&mut receiver, notify_subscribe.clone(), &event_handlers).await;
        });

        // Start the threads.
        if should_run_sources {
            let source_node = create_source_node(
                ctx,
                &mut execution_dag,
                &self.options,
                shutdown.clone(),
                runtime.clone(),
                incremental_run_config
                    .as_ref()
                    .map(|_| execute_node_ids.clone()),
            )
            .await;

            join_handles.push(start_source(source_node)?);
        } else {
            tracing::info!("No executable source nodes. SourceNode will not start.");
        }

        for node_index in node_indexes {
            let Some(node) = execution_dag.graph()[node_index].kind.as_ref() else {
                continue;
            };
            let node_id = execution_dag.graph()[node_index].handle.id.clone();
            if !execute_node_ids.contains(&node_id) {
                tracing::info!("Skipping node {} for incremental run", node_id);
                continue;
            }
            match node {
                NodeKind::Source { .. } => continue,
                NodeKind::Processor(_) => {
                    let ctx = NodeContext::new(
                        Arc::clone(&expr_engine),
                        Arc::clone(&storage_resolver),
                        Arc::clone(&kv_store),
                        execution_dag.event_hub().clone(),
                    );
                    let processor_node = ProcessorNode::new(
                        ctx,
                        &mut execution_dag,
                        node_index,
                        shutdown.clone(),
                        runtime.clone(),
                        incremental_run_config.is_some(),
                    )
                    .await;
                    join_handles.push(start_processor(processor_node)?);
                }
                NodeKind::Sink(_) => {
                    let ctx = NodeContext::new(
                        Arc::clone(&expr_engine),
                        Arc::clone(&storage_resolver),
                        Arc::clone(&kv_store),
                        execution_dag.event_hub().clone(),
                    );
                    let sink_node = SinkNode::new(
                        ctx,
                        &mut execution_dag,
                        node_index,
                        shutdown.clone(),
                        runtime.clone(),
                        incremental_run_config.is_some(),
                    );
                    join_handles.push(start_sink(sink_node)?);
                }
            }
        }

        if let Some(cfg) = incremental_run_config.clone() {
            let replay_groups =
                build_replay_groups(&execution_dag, &execute_node_ids, &cfg.available_edge_ids);
            tracing::info!("Replay groups:");
            for g in &replay_groups {
                tracing::info!("  group edges={}", g.edges.len());
                for e in &g.edges {
                    tracing::info!(
                        "    edge_id={}, port={}",
                        e.edge_id,
                        e.downstream_input_port
                    );
                }
            }

            tracing::info!(
                "Incremental replay: {} group(s) to inject",
                replay_groups.len()
            );

            let expr_engine2 = Arc::clone(&expr_engine);
            let storage_resolver2 = Arc::clone(&storage_resolver);
            let kv_store2 = Arc::clone(&kv_store);
            let event_hub2 = execution_dag.event_hub().clone();

            let injector_handle = std::thread::Builder::new()
                .name("replay-injector".to_string())
                .spawn(move || {
                    let node_ctx =
                        NodeContext::new(expr_engine2, storage_resolver2, kv_store2, event_hub2);
                    replay_inject(cfg, replay_groups, node_ctx);
                    Ok::<(), ExecutionError>(())
                })
                .map_err(ExecutionError::CannotSpawnWorkerThread)?;

            join_handles.push(injector_handle);
        }

        Ok(DagExecutorJoinHandle {
            event_hub,
            join_handles,
            notify: notify_publish.clone(),
        })
    }
}

async fn subscribe_event(
    receiver: &mut Receiver<Event>,
    notify: Arc<Notify>,
    event_handlers: &[Arc<dyn EventHandler>],
) {
    crate::event::subscribe_event(receiver, notify, event_handlers).await;
}

impl DagExecutorJoinHandle {
    pub fn join(&mut self, runtime: Handle) -> Result<(), ExecutionError> {
        loop {
            let Some(finished) = self
                .join_handles
                .iter()
                .enumerate()
                .find_map(|(i, handle)| handle.is_finished().then_some(i))
            else {
                std::thread::sleep(Duration::from_millis(250));

                continue;
            };
            let handle = self.join_handles.swap_remove(finished);
            handle.join().unwrap()?;

            if self.join_handles.is_empty() {
                // All threads have completed, add a delay before returning
                tracing::info!("Workflow complete, waiting for final events to be published...");

                // Enhanced delay approach - use improved flush with dynamic waiting
                runtime.block_on(self.event_hub.enhanced_flush(5000));

                tracing::info!("Proceeding with workflow termination");

                return Ok(());
            }
        }
    }

    pub fn notify(&self) {
        self.notify.notify_waiters();
    }
}

fn start_source<F: Send + 'static + Future + Unpin + Debug>(
    source: SourceNode<F>,
) -> Result<JoinHandle<Result<(), ExecutionError>>, ExecutionError> {
    let handle = Builder::new()
        .name("sources".into())
        .spawn(move || match source.run() {
            Ok(()) => Ok(()),
            // Channel disconnection means the source listener has quit.
            // Maybe it quit gracefully so we don't need to propagate the error.
            Err(e) => {
                if let ExecutionError::Source(e) = &e {
                    if let Some(ExecutionError::CannotSendToChannel(_)) = e.downcast_ref() {
                        return Ok(());
                    }
                }
                Err(e)
            }
        })
        .map_err(ExecutionError::CannotSpawnWorkerThread)?;

    Ok(handle)
}

fn start_processor<F: Send + 'static + Future + Unpin + Debug>(
    processor: ProcessorNode<F>,
) -> Result<JoinHandle<Result<(), ExecutionError>>, ExecutionError> {
    Builder::new()
        .name(processor.handle().to_string())
        .spawn(move || {
            processor.run()?;
            Ok(())
        })
        .map_err(ExecutionError::CannotSpawnWorkerThread)
}

fn start_sink<F: Send + 'static + Future + Unpin + Debug>(
    sink: SinkNode<F>,
) -> Result<JoinHandle<Result<(), ExecutionError>>, ExecutionError> {
    Builder::new()
        .name(sink.handle().to_string())
        .spawn(|| {
            sink.run()?;
            Ok(())
        })
        .map_err(ExecutionError::CannotSpawnWorkerThread)
}

/// Collects nodes that should be executed in incremental run.
/// Only includes nodes that either:
/// 1. Are downstream of the start node, OR
/// 2. Have at least one incoming edge that is NOT in available_edges (need to be executed)
fn collect_executable_node_ids(
    dag: &ExecutionDag,
    cfg: &IncrementalRunConfig,
) -> Result<HashSet<NodeId>, ExecutionError> {
    let g = dag.graph();

    let start_index = g
        .node_indices()
        .find(|&i| g[i].handle.id.to_string() == cfg.start_node_id.to_string())
        .ok_or_else(|| {
            ExecutionError::CannotSpawnWorkerThread(std::io::Error::other(format!(
                "start_node_id not found in execution graph: {}",
                cfg.start_node_id
            )))
        })?;

    // First, collect all downstream nodes from start_node
    let mut downstream_nodes: HashSet<NodeId> = HashSet::new();
    let mut q = VecDeque::new();

    downstream_nodes.insert(g[start_index].handle.id.clone());
    q.push_back(start_index);

    while let Some(n) = q.pop_front() {
        for nb in g.neighbors_directed(n, Direction::Outgoing) {
            let id = g[nb].handle.id.clone();
            if downstream_nodes.insert(id) {
                q.push_back(nb);
            }
        }
    }

    // Second, check all nodes to see if they have incoming edges not in available_edges
    let mut executable_nodes = downstream_nodes.clone();

    for node_idx in g.node_indices() {
        let node_id = g[node_idx].handle.id.clone();

        // Skip if already marked as executable
        if executable_nodes.contains(&node_id) {
            continue;
        }

        // Check incoming edges
        let has_unavailable_edge = g.edges_directed(node_idx, Direction::Incoming).any(|edge| {
            let edge_id = edge.weight().edge_id.to_string().parse::<uuid::Uuid>().ok();
            edge_id.map_or(true, |id| !cfg.available_edge_ids.contains(&id))
        });

        // If this node has any incoming edge that's not available, it must be executed
        if has_unavailable_edge {
            tracing::info!(
                "Node {} marked as executable: has incoming edges not in available_edges",
                node_id
            );
            executable_nodes.insert(node_id);
        }
    }

    Ok(executable_nodes)
}

#[derive(Clone)]
struct ReplayEdge {
    edge_id: EdgeId,
    downstream_input_port: Port,
}

#[derive(Clone)]
struct ReplayGroup {
    sender: Sender<ExecutorOperation>,
    edges: Vec<ReplayEdge>,
}

/// Builds replay groups only for edges that are in available_edges.
/// This ensures we only replay data that was actually copied from the previous run.
fn build_replay_groups(
    dag: &ExecutionDag,
    execute: &HashSet<NodeId>,
    available_edge_ids: &HashSet<uuid::Uuid>,
) -> Vec<ReplayGroup> {
    let g = dag.graph();

    let mut grouped: HashMap<(NodeId, Port), (Sender<ExecutorOperation>, Vec<ReplayEdge>)> =
        HashMap::new();

    for e in g.edge_references() {
        let src = g[e.source()].handle.id.clone();
        let dst = g[e.target()].handle.id.clone();

        // Only create replay groups for edges where:
        // 1. Destination is executable
        // 2. Source is not executable (i.e., upstream)
        // 3. Edge is in available_edges (was actually copied)
        if execute.contains(&dst) && !execute.contains(&src) {
            let edge_id_parsed = e.weight().edge_id.to_string().parse::<uuid::Uuid>().ok();
            let is_available = edge_id_parsed.map_or(false, |id| available_edge_ids.contains(&id));

            if !is_available {
                tracing::info!(
                    "Skipping replay edge {} -> {}: edge {} not in available_edges",
                    src,
                    dst,
                    e.weight().edge_id
                );
                continue;
            }

            let downstream_input_port = e.weight().input_port.clone();

            let replay_edge = ReplayEdge {
                edge_id: e.weight().edge_id.clone(),
                downstream_input_port: downstream_input_port.clone(),
            };

            grouped
                .entry((dst.clone(), downstream_input_port))
                .and_modify(|(_, v)| v.push(replay_edge.clone()))
                .or_insert((e.weight().sender.clone(), vec![replay_edge]));
        }
    }

    grouped
        .into_iter()
        .map(|(_, (sender, edges))| ReplayGroup { sender, edges })
        .collect()
}

fn read_replay_features(
    state: &reearth_flow_state::State,
    edge_id: &str,
) -> std::io::Result<Vec<reearth_flow_types::Feature>> {
    let values = state.read_jsonl_auto_sync::<serde_json::Value>(edge_id)?;
    let mut out = Vec::with_capacity(values.len());
    for v in values {
        let f: reearth_flow_types::Feature =
            serde_json::from_value(v).map_err(std::io::Error::other)?;
        out.push(f);
    }
    Ok(out)
}

fn replay_inject(cfg: IncrementalRunConfig, groups: Vec<ReplayGroup>, node_ctx: NodeContext) {
    for g in groups {
        tracing::info!("Replay inject start: {} edge(s)", g.edges.len());

        let mut sent = 0usize;

        for e in &g.edges {
            let edge_id_str = e.edge_id.to_string();
            match read_replay_features(&cfg.previous_feature_state, &edge_id_str) {
                Ok(features) => {
                    for feature in features {
                        let ctx = crate::executor_operation::ExecutorContext::new(
                            feature,
                            e.downstream_input_port.clone(),
                            node_ctx.expr_engine.clone(),
                            node_ctx.storage_resolver.clone(),
                            node_ctx.kv_store.clone(),
                            node_ctx.event_hub.clone(),
                        );

                        if let Err(err) = g.sender.send(ExecutorOperation::Op { ctx }) {
                            tracing::error!("Replay inject send failed: {:?}", err);
                            break;
                        }
                        sent += 1;
                    }
                }
                Err(err) => {
                    tracing::warn!("Replay inject read failed for {}: {:?}", edge_id_str, err);
                }
            }
        }

        let _ = g.sender.send(ExecutorOperation::Terminate {
            ctx: node_ctx.clone(),
        });

        tracing::info!("Replay inject done: sent {} op(s) and terminate", sent);
    }
}
