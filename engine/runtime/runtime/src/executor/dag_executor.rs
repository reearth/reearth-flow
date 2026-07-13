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
use reearth_flow_diagnostics::{Diagnostic, DiagnosticDraft, Disposition, ErrorCode, RunSummary};
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
use crate::cache::cleanup_executor_cache;

pub struct DagExecutor {
    builder_dag: BuilderDag,
    options: ExecutorOptions,
}

/// Per-node-thread outcome, carried alongside the thread's terminal
/// `Result<(), ExecutionError>` so `DagExecutorJoinHandle::join` can fold
/// diagnostics from every node — including ones whose thread ultimately
/// errored — into a `RunSummary`. Executor-internal wire type: lives in the
/// runtime crate rather than `reearth_flow_diagnostics` because it only
/// exists to get data across a `std::thread::JoinHandle` boundary.
#[derive(Debug, Default)]
pub struct NodeOutcome {
    pub summaries: Vec<Diagnostic>,
}

type NodeThreadResult = (NodeOutcome, Result<(), ExecutionError>);

pub struct DagExecutorJoinHandle {
    join_handles: Vec<JoinHandle<NodeThreadResult>>,
    notify: Arc<Notify>,
    executor_id: uuid::Uuid,
}

impl DagExecutor {
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
        storage_resolver: Arc<StorageResolver>,
        kv_store: Arc<dyn KvStore>,
        entry_graph_id: uuid::Uuid,
        graphs: Vec<Graph>,
        options: ExecutorOptions,
        factories: HashMap<String, crate::node::NodeKind>,
        global_params: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Result<Self, ExecutionError> {
        let dag_schemas =
            DagSchemas::from_graphs(entry_graph_id, graphs, factories, global_params)?;
        let event_hub = EventHub::new(options.event_hub_capacity);
        let ctx = NodeContext::new(
            env_vars,
            storage_resolver,
            kv_store,
            event_hub,
            options.sandbox_root.clone(),
        );
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
        env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
        storage_resolver: Arc<StorageResolver>,
        kv_store: Arc<dyn crate::kvs::KvStore>,
        ingress_state: Arc<State>,
        feature_state: Arc<State>,
        incremental_run_config: Option<IncrementalRunConfig>,
        event_handlers: Vec<Arc<dyn EventHandler>>,
        executor_id: uuid::Uuid,
    ) -> Result<DagExecutorJoinHandle, ExecutionError> {
        // Extract fields from options before partial moves.
        let sandbox_root = self.options.sandbox_root.clone();

        // Construct execution dag.
        let mut execution_dag = ExecutionDag::new(
            self.builder_dag,
            self.options.channel_buffer_sz,
            self.options.feature_flush_threshold,
            Arc::clone(&ingress_state),
            Arc::clone(&feature_state),
            executor_id,
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

        let ctx = NodeContext::new(
            Arc::clone(&env_vars),
            Arc::clone(&storage_resolver),
            Arc::clone(&kv_store),
            execution_dag.event_hub().clone(),
            sandbox_root.clone(),
        );

        // Run-scoped warn-once dedup set: shared by every processor/sink node
        // started below so `ctx.warn_once(...)` fires at most once per code
        // per run, regardless of which node reports it.
        let warn_once: reearth_flow_diagnostics::WarnOnceSet = Arc::default();

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
                        Arc::clone(&env_vars),
                        Arc::clone(&storage_resolver),
                        Arc::clone(&kv_store),
                        execution_dag.event_hub().clone(),
                        sandbox_root.clone(),
                    );
                    let processor_node = ProcessorNode::new(
                        ctx,
                        &mut execution_dag,
                        node_index,
                        shutdown.clone(),
                        runtime.clone(),
                        incremental_run_config.is_some(),
                        warn_once.clone(),
                    )
                    .await;
                    join_handles.push(start_processor(processor_node)?);
                }
                NodeKind::Sink(_) => {
                    let ctx = NodeContext::new(
                        Arc::clone(&env_vars),
                        Arc::clone(&storage_resolver),
                        Arc::clone(&kv_store),
                        execution_dag.event_hub().clone(),
                        sandbox_root.clone(),
                    );
                    let sink_node = SinkNode::new(
                        ctx,
                        &mut execution_dag,
                        node_index,
                        shutdown.clone(),
                        runtime.clone(),
                        incremental_run_config.is_some(),
                        warn_once.clone(),
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

            let env_vars2 = Arc::clone(&env_vars);
            let storage_resolver2 = Arc::clone(&storage_resolver);
            let kv_store2 = Arc::clone(&kv_store);
            let event_hub2 = execution_dag.event_hub().clone();

            let injector_handle = std::thread::Builder::new()
                .name("replay-injector".to_string())
                .spawn(move || {
                    let node_ctx = NodeContext::new(
                        env_vars2,
                        storage_resolver2,
                        kv_store2,
                        event_hub2,
                        sandbox_root,
                    );
                    replay_inject(cfg, replay_groups, node_ctx);
                    (NodeOutcome::default(), Ok::<(), ExecutionError>(()))
                })
                .map_err(ExecutionError::CannotSpawnWorkerThread)?;

            join_handles.push(injector_handle);
        }

        Ok(DagExecutorJoinHandle {
            join_handles,
            notify: notify_publish.clone(),
            executor_id,
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
    /// Collects every node thread's `(NodeOutcome, Result<(), ExecutionError>)`
    /// — never short-circuits on the first failure — then folds them into a
    /// `RunSummary` via `fold_outcomes`. `cleanup_executor_cache` always runs
    /// once every thread has been joined, on every exit path.
    ///
    /// Interim (Phase 2a Task 5) error semantics: collecting every thread
    /// before deciding anything fixes the historical leaked-threads bug,
    /// where the old fail-fast loop returned on the FIRST thread to error
    /// while every other node thread kept running unjoined. But to keep
    /// `run_dag_executor` and every golden logging scenario byte-identical
    /// until Task 6 threads `RunSummary` all the way through the runner,
    /// this still surfaces the first-completed thread's raw
    /// `ExecutionError` via `Err(..)` — exactly what the old fail-fast loop
    /// returned (it always returned whichever thread finished first, and
    /// threads are discovered/collected here in that same order). Task 6
    /// removes this branch, always returns `Ok(RunSummary)`, and leaves
    /// fatality to be decided by the caller from `failed_nodes`.
    pub fn join(&mut self) -> Result<RunSummary, ExecutionError> {
        let mut results: Vec<NodeThreadResult> = Vec::with_capacity(self.join_handles.len());

        while !self.join_handles.is_empty() {
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
            // A panicked node thread is a bug, not a per-node failure — keep
            // re-raising it here rather than folding it into `RunSummary`.
            results.push(handle.join().unwrap());
        }

        // `enhanced_flush(5000)` used to live here. Its early-break
        // condition (`sender.receiver_count() == 0`) never fired in
        // practice because broadcast subscribers stay attached for
        // the lifetime of the workflow, so the call effectively
        // waited the full 5 seconds on every workflow execution
        // (~11+ minutes across the 141 workflow-tests). The runner-
        // level shutdown sleep + the trailing settle in any caller
        // that needs one provides enough of a drain window without
        // this 5s tax.
        cleanup_executor_cache(self.executor_id);

        if let Some(pos) = results.iter().position(|(_, result)| result.is_err()) {
            let (_, result) = results.remove(pos);
            return Err(result.expect_err("position() above guarantees Err"));
        }

        Ok(fold_outcomes(results))
    }

    pub fn notify(&self) {
        self.notify.notify_waiters();
    }
}

/// Pure fold: turns every node thread's `(NodeOutcome, Result<(),
/// ExecutionError>)` into one `RunSummary`. `aggregated_diagnostics` is
/// every outcome's summaries, extended in collection order. Each `Err`
/// becomes one `failed_nodes` entry, recovered from the structured
/// `Processor`/`Sink`/`Source` fatal-backstop diagnostic when the boxed
/// error downcasts to one, else synthesized under
/// `ErrorCode::InternalUnclassified`. Every `failed_nodes` entry is stamped
/// `effective_disposition = Some(Disposition::Fatal)`: reaching this fold
/// at all means that node's thread returned `Err`, so the run is fatally
/// broken by it regardless of the diagnostic's own registry default.
fn fold_outcomes(results: Vec<NodeThreadResult>) -> RunSummary {
    let mut aggregated_diagnostics = Vec::new();
    let mut failed_nodes = Vec::new();

    for (outcome, result) in results {
        aggregated_diagnostics.extend(outcome.summaries);
        if let Err(e) = result {
            let mut diagnostic = diagnostic_from_execution_error(e);
            diagnostic.effective_disposition = Some(Disposition::Fatal);
            failed_nodes.push(diagnostic);
        }
    }

    RunSummary {
        failed_nodes,
        aggregated_diagnostics,
        dropped_event_count: 0, // Task 7 wires drop-event accounting through.
    }
}

/// Recovers the original structured `Diagnostic` when `e` is the fatal
/// backstop (`ExecutionError::Processor/Sink/Source(Box<Diagnostic>)`);
/// otherwise synthesizes one under the catch-all unclassified code so every
/// node failure — however it originated — becomes a `Diagnostic`.
fn diagnostic_from_execution_error(e: ExecutionError) -> Diagnostic {
    let rendered = e.to_string();
    let boxed = match e {
        ExecutionError::Processor(b) | ExecutionError::Sink(b) | ExecutionError::Source(b) => {
            Some(b)
        }
        _ => None,
    };
    match boxed.map(|b| b.downcast::<Diagnostic>()) {
        Some(Ok(diag)) => *diag,
        _ => Diagnostic::from_draft(
            DiagnosticDraft::new(ErrorCode::InternalUnclassified).with_message(rendered),
            None,
            None,
            None,
        ),
    }
}

fn start_source<F: Send + 'static + Future + Unpin + Debug>(
    source: SourceNode<F>,
) -> Result<JoinHandle<NodeThreadResult>, ExecutionError> {
    let handle = Builder::new()
        .name("sources".into())
        .spawn(move || {
            let result = match source.run() {
                Ok(()) => Ok(()),
                // Channel disconnection means the source listener has quit.
                // Maybe it quit gracefully so we don't need to propagate the error.
                Err(e) => {
                    if let ExecutionError::Source(e) = &e {
                        if let Some(ExecutionError::CannotSendToChannel(_)) = e.downcast_ref() {
                            return (NodeOutcome::default(), Ok(()));
                        }
                    }
                    Err(e)
                }
            };
            (NodeOutcome::default(), result)
        })
        .map_err(ExecutionError::CannotSpawnWorkerThread)?;

    Ok(handle)
}

fn start_processor<F: Send + 'static + Future + Unpin + Debug>(
    processor: ProcessorNode<F>,
) -> Result<JoinHandle<NodeThreadResult>, ExecutionError> {
    let name = processor.handle().to_string();
    let summaries_sink = processor.summaries_sink();
    Builder::new()
        .name(name)
        .spawn(move || {
            let result = processor.run();
            let summaries = std::mem::take(&mut *summaries_sink.lock());
            (NodeOutcome { summaries }, result)
        })
        .map_err(ExecutionError::CannotSpawnWorkerThread)
}

fn start_sink<F: Send + 'static + Future + Unpin + Debug>(
    sink: SinkNode<F>,
) -> Result<JoinHandle<NodeThreadResult>, ExecutionError> {
    let name = sink.handle().to_string();
    let summaries_sink = sink.summaries_sink();
    Builder::new()
        .name(name)
        .spawn(move || {
            let result = sink.run();
            let summaries = std::mem::take(&mut *summaries_sink.lock());
            (NodeOutcome { summaries }, result)
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
            edge_id.is_none_or(|id| !cfg.available_edge_ids.contains(&id))
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
            let is_available = edge_id_parsed.is_some_and(|id| available_edge_ids.contains(&id));

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
                            node_ctx.env_vars.clone(),
                            node_ctx.storage_resolver.clone(),
                            node_ctx.kv_store.clone(),
                            node_ctx.event_hub.clone(),
                            node_ctx.sandbox_root.clone(),
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

#[cfg(test)]
mod fold_outcomes_tests {
    use super::*;
    use reearth_flow_diagnostics::Severity;

    fn diagnostic(code: ErrorCode, message: &str) -> Diagnostic {
        Diagnostic::from_draft(
            DiagnosticDraft::new(code).with_message(message),
            None,
            None,
            None,
        )
    }

    fn outcome(summaries: Vec<Diagnostic>) -> NodeOutcome {
        NodeOutcome { summaries }
    }

    #[test]
    fn empty_input_returns_default_summary() {
        let summary = fold_outcomes(vec![]);
        assert!(summary.failed_nodes.is_empty());
        assert!(summary.aggregated_diagnostics.is_empty());
        assert_eq!(summary.dropped_event_count, 0);
    }

    #[test]
    fn all_success_folds_summaries_with_empty_failed_nodes() {
        let d1 = diagnostic(ErrorCode::GltfZeroFaceSolid, "d1");
        let d2 = diagnostic(ErrorCode::Cesium3dtilesEmptyGeometry, "d2");
        let results = vec![
            (outcome(vec![d1.clone()]), Ok(())),
            (outcome(vec![d2.clone()]), Ok(())),
        ];

        let summary = fold_outcomes(results);

        assert!(summary.failed_nodes.is_empty());
        assert_eq!(summary.aggregated_diagnostics.len(), 2);
        assert_eq!(summary.aggregated_diagnostics[0].message, "d1");
        assert_eq!(summary.aggregated_diagnostics[1].message, "d2");
    }

    #[test]
    fn recovers_diagnostic_via_downcast_from_processor_error() {
        let original = diagnostic(ErrorCode::InternalInvariantViolation, "boom");
        let results = vec![(
            outcome(vec![]),
            Err(ExecutionError::Processor(Box::new(original))),
        )];

        let summary = fold_outcomes(results);

        assert_eq!(summary.failed_nodes.len(), 1);
        let recovered = &summary.failed_nodes[0];
        assert_eq!(recovered.code, ErrorCode::InternalInvariantViolation);
        assert_eq!(recovered.message, "boom");
        assert_eq!(recovered.effective_disposition, Some(Disposition::Fatal));
    }

    #[test]
    fn recovers_diagnostic_via_downcast_from_sink_error() {
        let original = diagnostic(ErrorCode::InternalInvariantViolation, "sink boom");
        let results = vec![(
            outcome(vec![]),
            Err(ExecutionError::Sink(Box::new(original))),
        )];

        let summary = fold_outcomes(results);

        assert_eq!(summary.failed_nodes.len(), 1);
        assert_eq!(summary.failed_nodes[0].message, "sink boom");
        assert_eq!(
            summary.failed_nodes[0].effective_disposition,
            Some(Disposition::Fatal)
        );
    }

    #[test]
    fn recovers_diagnostic_via_downcast_from_source_error() {
        let original = diagnostic(ErrorCode::InternalInvariantViolation, "source boom");
        let boxed: crate::errors::BoxedError = Box::new(original);
        let results = vec![(outcome(vec![]), Err(ExecutionError::Source(boxed)))];

        let summary = fold_outcomes(results);

        assert_eq!(summary.failed_nodes.len(), 1);
        assert_eq!(summary.failed_nodes[0].message, "source boom");
    }

    #[test]
    fn synthesizes_unclassified_for_non_diagnostic_boxed_error() {
        // A `Processor` variant whose boxed error is NOT a `Diagnostic` —
        // downcast fails, so the fold must synthesize instead of panicking.
        let boxed: crate::errors::BoxedError = Box::new(std::io::Error::other("io boom"));
        let results = vec![(outcome(vec![]), Err(ExecutionError::Processor(boxed)))];

        let summary = fold_outcomes(results);

        assert_eq!(summary.failed_nodes.len(), 1);
        let synthesized = &summary.failed_nodes[0];
        assert_eq!(synthesized.code, ErrorCode::InternalUnclassified);
        assert_eq!(synthesized.effective_disposition, Some(Disposition::Fatal));
        assert!(synthesized.message.contains("io boom"));
        assert!(synthesized.message.contains("Processor error"));
    }

    #[test]
    fn synthesizes_unclassified_for_non_structured_execution_error_variant() {
        // A variant that never carries a `Diagnostic` at all (e.g. a channel
        // failure) must also be synthesized, not dropped or panicked on.
        let results = vec![(
            outcome(vec![]),
            Err(ExecutionError::CannotSendToChannel("channel boom".into())),
        )];

        let summary = fold_outcomes(results);

        assert_eq!(summary.failed_nodes.len(), 1);
        let synthesized = &summary.failed_nodes[0];
        assert_eq!(synthesized.code, ErrorCode::InternalUnclassified);
        assert_eq!(synthesized.severity, Severity::Fatal);
        assert_eq!(synthesized.effective_disposition, Some(Disposition::Fatal));
        assert!(synthesized.message.contains("channel boom"));
    }

    #[test]
    fn mixed_outcomes_preserve_collection_order_for_both_vecs() {
        let d_ok_1 = diagnostic(ErrorCode::GltfZeroFaceSolid, "ok-1");
        let fatal_diag = diagnostic(ErrorCode::InternalInvariantViolation, "fatal-1");
        let d_ok_2 = diagnostic(ErrorCode::Cesium3dtilesEmptyGeometry, "ok-2");

        let results = vec![
            (outcome(vec![d_ok_1.clone()]), Ok(())),
            (
                outcome(vec![]),
                Err(ExecutionError::Processor(Box::new(fatal_diag))),
            ),
            (outcome(vec![d_ok_2.clone()]), Ok(())),
            (
                outcome(vec![]),
                Err(ExecutionError::CannotReceiveFromChannel(
                    "second boom".into(),
                )),
            ),
        ];

        let summary = fold_outcomes(results);

        // aggregated_diagnostics: only from the two Ok entries, in order.
        assert_eq!(summary.aggregated_diagnostics.len(), 2);
        assert_eq!(summary.aggregated_diagnostics[0].message, "ok-1");
        assert_eq!(summary.aggregated_diagnostics[1].message, "ok-2");

        // failed_nodes: one recovered, one synthesized, in collection order.
        assert_eq!(summary.failed_nodes.len(), 2);
        assert_eq!(summary.failed_nodes[0].message, "fatal-1");
        assert_eq!(
            summary.failed_nodes[0].code,
            ErrorCode::InternalInvariantViolation
        );
        assert_eq!(
            summary.failed_nodes[1].code,
            ErrorCode::InternalUnclassified
        );
        assert!(summary.failed_nodes[1].message.contains("second boom"));
        for failed in &summary.failed_nodes {
            assert_eq!(failed.effective_disposition, Some(Disposition::Fatal));
        }
    }

    #[test]
    fn dropped_event_count_defaults_to_zero_pending_task_7() {
        let summary = fold_outcomes(vec![(outcome(vec![]), Ok(()))]);
        assert_eq!(summary.dropped_event_count, 0);
    }
}
