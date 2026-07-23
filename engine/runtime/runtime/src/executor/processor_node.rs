use std::env;
use std::fmt::Debug;
use std::io::BufRead;
use std::sync::atomic::{AtomicU32, AtomicU64};
use std::sync::Arc;
use std::time::{self, Duration};
use std::{borrow::Cow, mem::swap};

use crossbeam::channel::Receiver;
use futures::Future;
use once_cell::sync::Lazy;
use petgraph::graph::NodeIndex;
use reearth_flow_common::uri::Uri;
use reearth_flow_diagnostics::{Diagnostic, DiagnosticDraft, Disposition, ErrorCode};
use reearth_flow_state::State;
use reearth_flow_storage::resolve::StorageResolver;
use tokio::runtime::Handle;
use tracing::{info_span, Span};

use crate::event::{Event, EventHub, NodeMetrics};
use crate::executor_operation::{ExecutorContext, ExecutorOperation, NodeContext};
use crate::forwarder::ProcessorChannelForwarder;
use crate::kvs::KvStore;
use crate::node::NodeStatus;
use crate::{
    builder_dag::NodeKind,
    errors::{to_node_error, ExecutionError, NodeErrorKind},
    forwarder::ChannelManager,
    node::{NodeHandle, Processor},
};

use super::receiver_loop::init_select;
use super::source_intermediate::SourceIntermediateRecorder;
use super::{execution_dag::ExecutionDag, receiver_loop::ReceiverLoop};

static SLOW_ACTION_THRESHOLD: Lazy<Duration> = Lazy::new(|| {
    env::var("FLOW_RUNTIME_SLOW_ACTION_THRESHOLD")
        .ok()
        .and_then(|v| v.parse().ok())
        .map(Duration::from_millis)
        .unwrap_or(Duration::from_millis(1000))
});

#[derive(Debug)]
pub struct ProcessorNode<F> {
    node_handle: NodeHandle,
    node_name: String,
    node_handles: Vec<NodeHandle>,
    receivers: Vec<Receiver<ExecutorOperation>>,
    processor: Arc<parking_lot::RwLock<Box<dyn Processor>>>,
    channel_manager: Arc<parking_lot::RwLock<ProcessorChannelForwarder>>,
    #[allow(dead_code)]
    shutdown: F,
    #[allow(dead_code)]
    runtime: Arc<Handle>,
    span: tracing::Span,
    thread_pool: rayon::ThreadPool,
    num_threads: usize,
    thread_counter: Arc<AtomicU32>,
    features_processed: Arc<AtomicU64>,
    finish_feature_count: Arc<AtomicU64>,
    process_duration_us: Arc<AtomicU64>,
    process_duration_sq_us: Arc<AtomicU64>,
    env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
    storage_resolver: Arc<StorageResolver>,
    kv_store: Arc<dyn KvStore>,
    event_hub: EventHub,
    sandbox_root: Uri,
    source_intermediate_recorder: SourceIntermediateRecorder,
    feature_state: Arc<State>,
    incremental_mode: bool,
    diagnostics: crate::diagnostics::SharedNodeDiagnostics,
    summaries_sink: Arc<parking_lot::Mutex<Vec<Diagnostic>>>,
}

impl<F: Future + Unpin + Debug> ProcessorNode<F> {
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        ctx: NodeContext,
        dag: &mut ExecutionDag,
        node_index: NodeIndex,
        shutdown: F,
        runtime: Arc<Handle>,
        incremental_mode: bool,
        warn_once: reearth_flow_diagnostics::WarnOnceSet,
        disposition_policy: Arc<reearth_flow_diagnostics::DispositionPolicy>,
    ) -> Self {
        let node = dag.node_weight_mut(node_index);
        let node_handle = node.handle.clone();
        let node_name = node.name.clone();
        let composed_id = node.composed_id();
        let action = node.action.clone();
        let Some(kind) = node.kind.take() else {
            panic!("Must pass in a node")
        };
        let NodeKind::Processor(processor) = kind else {
            panic!("Must pass in a processor node");
        };
        // NOTE: `action` may legitimately diverge from `processor.name()` (e.g. factory vs. built-instance naming) — don't assert equality.
        let (node_handles, receivers) = dag.collect_receivers(node_index);

        let senders = dag.collect_senders(node_index);
        let port_writers = dag.collect_port_writers(node_index);

        let channel_manager = ProcessorChannelForwarder::ChannelManager(ChannelManager::new(
            node_handle.clone(),
            port_writers,
            senders,
            runtime.clone(),
            dag.event_hub().clone(),
            dag.executor_id(),
        ));
        let version = env!("CARGO_PKG_VERSION");
        let span = info_span!(
            "action",
            "engine.version" = version,
            "otel.name" = processor.name(),
            "otel.kind" = "Processor Node",
            "workflow.id" = dag.id.to_string().as_str(),
            "node.id" = composed_id.as_str(),
            "node.name" = node_name.as_str(),
        );

        let env_vars = Arc::clone(&ctx.env_vars);
        let storage_resolver = Arc::clone(&ctx.storage_resolver);
        let kv_store = Arc::clone(&ctx.kv_store);
        let sandbox_root = ctx.sandbox_root.clone();
        let num_threads = processor.num_threads();

        let source_intermediate_recorder =
            SourceIntermediateRecorder::collect(dag, node_index, &node_handles);
        let feature_state = dag.feature_state();

        let diagnostics = Arc::new(crate::diagnostics::NodeDiagnosticsHandle::new(
            composed_id,
            node_handle.clone(),
            node_name.clone(),
            action,
            warn_once,
            disposition_policy,
            false,
        ));

        Self {
            node_handle,
            node_name,
            node_handles,
            receivers,
            processor: Arc::new(parking_lot::RwLock::new(processor)),
            channel_manager: Arc::new(parking_lot::RwLock::new(channel_manager)),
            shutdown,
            runtime,
            span,
            thread_pool: rayon::ThreadPoolBuilder::new()
                .num_threads(num_threads)
                .build()
                .unwrap(),
            num_threads,
            thread_counter: Arc::new(AtomicU32::new(0)),
            features_processed: Arc::new(AtomicU64::new(0)),
            finish_feature_count: Arc::new(AtomicU64::new(0)),
            process_duration_us: Arc::new(AtomicU64::new(0)),
            process_duration_sq_us: Arc::new(AtomicU64::new(0)),
            env_vars,
            storage_resolver,
            kv_store,
            event_hub: dag.event_hub().clone(),
            sandbox_root,
            source_intermediate_recorder,
            feature_state,
            incremental_mode,
            diagnostics,
            summaries_sink: Arc::new(parking_lot::Mutex::new(Vec::new())),
        }
    }

    pub fn handle(&self) -> &NodeHandle {
        &self.node_handle
    }

    // Must be called before run() consumes self, or summaries become unreachable.
    pub fn summaries_sink(&self) -> Arc<parking_lot::Mutex<Vec<Diagnostic>>> {
        self.summaries_sink.clone()
    }

    pub fn node_meta(&self) -> super::dag_executor::NodeMeta {
        super::dag_executor::NodeMeta {
            composed_id: self.diagnostics.inner.node_id().to_string(),
            action: self.diagnostics.inner.action_type().to_string(),
        }
    }

    fn wait_until_pool_has_capacity(&self) {
        while self
            .thread_counter
            .load(std::sync::atomic::Ordering::SeqCst)
            >= self.num_threads as u32
        {
            std::thread::yield_now();
        }
    }
}

impl<F: Future + Unpin + Debug> ReceiverLoop for ProcessorNode<F> {
    fn receivers(&mut self) -> Vec<Receiver<ExecutorOperation>> {
        let mut result = vec![];
        swap(&mut self.receivers, &mut result);
        result
    }

    fn receiver_loop(mut self) -> Result<(), ExecutionError>
    where
        Self: Sized,
    {
        let receivers = self.receivers();
        let mut is_terminated = vec![false; receivers.len()];
        let mut sel = init_select(&receivers);

        let span = self.span.clone();
        let now = time::Instant::now();
        let processor = Arc::clone(&self.processor);

        self.event_hub.send(Event::NodeStatusChanged {
            node_handle: self.node_handle.clone(),
            status: NodeStatus::Starting,
            feature_id: None,
            metrics: None,
        });

        let init_result = processor
            .write()
            .initialize(NodeContext::new(
                self.env_vars.clone(),
                self.storage_resolver.clone(),
                self.kv_store.clone(),
                self.event_hub.clone(),
                self.sandbox_root.clone(),
            ))
            .map_err(ExecutionError::Processor);

        // Without this, an initialize() error leaves the node stuck at Starting forever.
        if let Err(ref e) = init_result {
            self.event_hub.error_log_with_node_info(
                Some(span.clone()),
                self.node_handle.clone(),
                self.node_name.clone(),
                format!("{} process error: {}", self.processor.read().name(), e),
            );

            self.event_hub.send(Event::NodeStatusChanged {
                node_handle: self.node_handle.clone(),
                status: NodeStatus::Failed,
                feature_id: None,
                metrics: None,
            });

            return init_result;
        }

        self.event_hub.send(Event::NodeStatusChanged {
            node_handle: self.node_handle.clone(),
            status: NodeStatus::Processing,
            feature_id: None,
            metrics: None,
        });

        self.event_hub.info_log_with_node_info(
            Some(span.clone()),
            self.node_handle.clone(),
            self.node_name.clone(),
            format!("{} process start...", self.processor.read().name()),
        );

        let has_failed = Arc::new(std::sync::atomic::AtomicBool::new(false));

        loop {
            if is_terminated.iter().all(|value| *value) {
                if self
                    .thread_counter
                    .load(std::sync::atomic::Ordering::SeqCst)
                    == 0
                {
                    let features_count = self
                        .features_processed
                        .load(std::sync::atomic::Ordering::Relaxed);
                    let is_failed = has_failed.load(std::sync::atomic::Ordering::SeqCst);

                    let total_us = self
                        .process_duration_us
                        .load(std::sync::atomic::Ordering::Relaxed);
                    let total_duration = Duration::from_micros(total_us);

                    let stats_suffix = if features_count > 1 {
                        let avg_us = total_us as f64 / features_count as f64;
                        let sq_us = self
                            .process_duration_sq_us
                            .load(std::sync::atomic::Ordering::Relaxed);
                        let variance = (sq_us as f64 / features_count as f64) - (avg_us * avg_us);
                        let stddev = if variance > 0.0 { variance.sqrt() } else { 0.0 };
                        format!(
                            ", avg = {:.3}ms, stddev = {:.3}ms",
                            avg_us / 1000.0,
                            stddev / 1000.0,
                        )
                    } else {
                        String::new()
                    };

                    let message = if features_count > 0 && !is_failed {
                        format!(
                            "{} process finish. elapsed = {:?}, features = {}{}",
                            self.processor.read().name(),
                            total_duration,
                            features_count,
                            stats_suffix,
                        )
                    } else {
                        format!(
                            "{} process terminate. elapsed = {:?}",
                            self.processor.read().name(),
                            now.elapsed()
                        )
                    };

                    self.event_hub.info_log_with_node_info(
                        Some(span.clone()),
                        self.node_handle.clone(),
                        self.node_name.clone(),
                        message,
                    );

                    let mut finish_ctx = NodeContext::new(
                        self.env_vars.clone(),
                        self.storage_resolver.clone(),
                        self.kv_store.clone(),
                        self.event_hub.clone(),
                        self.sandbox_root.clone(),
                    );
                    finish_ctx.diagnostics = Some(self.diagnostics.clone());
                    let terminate_result = self.on_terminate(finish_ctx);
                    let finish_feature_count = self
                        .finish_feature_count
                        .load(std::sync::atomic::Ordering::Relaxed);

                    // Failure precedence: a real on_terminate error always wins over the fatal slot backstop.
                    let fatal = self.diagnostics.inner.take_fatal();
                    let (final_result, node_failed, superseded_fatal) = reconcile_terminate_result(
                        terminate_result,
                        fatal,
                        has_failed.load(std::sync::atomic::Ordering::SeqCst),
                    );

                    if let Some(superseded) = superseded_fatal {
                        self.event_hub.warn_log_with_node_info(
                            Some(span.clone()),
                            self.node_handle.clone(),
                            self.node_name.clone(),
                            format!(
                                "{} process: swallowed fatal diagnostic ({}) superseded by a \
                                 real error and dropped from the final result",
                                self.processor.read().name(),
                                superseded.code
                            ),
                        );
                    }

                    self.event_hub.send(Event::NodeStatusChanged {
                        node_handle: self.node_handle.clone(),
                        status: if node_failed {
                            NodeStatus::Failed
                        } else {
                            NodeStatus::Completed
                        },
                        feature_id: None,
                        metrics: Some(NodeMetrics {
                            features_processed: features_count,
                            features_written: 0,
                            finish_feature_count,
                        }),
                    });

                    if !node_failed {
                        self.event_hub.send(Event::ProcessorFinished {
                            node: self.node_handle.clone(),
                            name: self.node_name.clone(),
                        });
                    }

                    return final_result;
                }
                // sleep, not yield_now() — yield_now() would spin at ~100% CPU with nothing else runnable.
                std::thread::sleep(Duration::from_micros(100));
                continue;
            }
            let index = sel.ready();
            let op = receivers[index]
                .recv()
                .map_err(|e| ExecutionError::CannotReceiveFromChannel(format!("{e:?}")))?;
            match op {
                ExecutorOperation::Op { ctx } => {
                    if !self.incremental_mode {
                        self.source_intermediate_recorder.record_if_from_source(
                            &self.feature_state,
                            index,
                            &ctx,
                            &self.node_name,
                            self.node_handle.id.as_ref(),
                        );
                    }
                    self.wait_until_pool_has_capacity();
                    let has_failed_clone = has_failed.clone();
                    self.on_op_with_failure_tracking(ctx, has_failed_clone)?;
                }
                ExecutorOperation::FileBackedOp {
                    path,
                    port,
                    context,
                } => {
                    let reader = crate::forwarder::open_jsonl_reader(&path).map_err(|e| {
                        ExecutionError::CannotReceiveFromChannel(format!(
                            "Failed to open file-backed op file {}: {e}",
                            path.display()
                        ))
                    })?;
                    for line in reader.lines() {
                        let line = line.map_err(|e| {
                            ExecutionError::CannotReceiveFromChannel(format!(
                                "Failed to read line from file-backed op: {e}"
                            ))
                        })?;
                        if line.is_empty() {
                            continue;
                        }
                        let feature: reearth_flow_types::Feature = serde_json::from_str(&line)
                            .map_err(|e| {
                                ExecutionError::CannotReceiveFromChannel(format!(
                                    "Failed to deserialize feature from file-backed op: {e}"
                                ))
                            })?;
                        let ctx = ExecutorContext::new_with_context_feature_and_port(
                            &context,
                            feature,
                            port.clone(),
                        );
                        self.wait_until_pool_has_capacity();
                        self.on_op_with_failure_tracking(ctx, has_failed.clone())?;
                    }
                }
                ExecutorOperation::Terminate { ctx: _ctx } => {
                    is_terminated[index] = true;
                    sel.remove(index);
                }
            }
        }
    }

    fn receiver_name(&'_ self, index: usize) -> Cow<'_, str> {
        Cow::Owned(self.node_handles[index].to_string())
    }

    fn on_op_with_failure_tracking(
        &mut self,
        mut ctx: ExecutorContext,
        has_failed: Arc<std::sync::atomic::AtomicBool>,
    ) -> Result<(), ExecutionError> {
        // This node's diagnostics handle overwrites whatever the upstream sender's context carried.
        ctx.diagnostics = Some(self.diagnostics.clone());
        let channel_manager = Arc::clone(&self.channel_manager);
        let processor = Arc::clone(&self.processor);

        let span = self.span.clone();
        let node_handle = self.node_handle.clone();
        let node_name = self.node_name.clone();
        let counter = Arc::clone(&self.thread_counter);
        let features_processed = Arc::clone(&self.features_processed);
        let process_duration_us = Arc::clone(&self.process_duration_us);
        let process_duration_sq_us = Arc::clone(&self.process_duration_sq_us);
        let event_hub = self.event_hub.clone();
        counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.thread_pool.spawn(move || {
            process(
                ctx,
                node_handle,
                node_name,
                span,
                event_hub,
                channel_manager,
                processor,
                has_failed,
                features_processed,
                process_duration_us,
                process_duration_sq_us,
            );
            counter.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        });
        Ok(())
    }

    fn on_op(&mut self, ctx: ExecutorContext) -> Result<(), ExecutionError> {
        let has_failed = Arc::new(std::sync::atomic::AtomicBool::new(false));
        self.on_op_with_failure_tracking(ctx, has_failed)
    }

    fn on_terminate(&mut self, ctx: NodeContext) -> Result<(), ExecutionError> {
        let channel_manager = Arc::clone(&self.channel_manager);
        let channel_manager_guard = channel_manager.read();
        let processor = Arc::clone(&self.processor);
        let channel_manager: &ProcessorChannelForwarder = &channel_manager_guard;
        let now = time::Instant::now();

        let _accumulating_guard = if processor.read().is_accumulating() {
            Some(super::accumulating_coordinator::acquire_permit())
        } else {
            None
        };

        // Spill excess finish()-emitted features to disk instead of blocking send() — avoids shutdown deadlock on a full channel.
        channel_manager.enable_spill_mode();
        channel_manager.reset_send_count();
        let result = processor
            .write()
            .finish(ctx.clone(), channel_manager)
            .map_err(|e| to_node_error(e, NodeErrorKind::Processor));
        // Summaries are emitted regardless of finish() outcome — must not be dropped just because finish() failed.
        let summaries = crate::diagnostics::emit_summaries(&self.event_hub, &self.diagnostics);
        *self.summaries_sink.lock() = summaries;
        channel_manager.flush_spill_files(&ctx.as_context());
        let finish_feature_count = channel_manager.get_send_count();
        self.finish_feature_count
            .store(finish_feature_count, std::sync::atomic::Ordering::Relaxed);

        drop(_accumulating_guard);

        let span = self.span.clone();
        self.event_hub.info_log_with_node_info(
            Some(span),
            self.node_handle.clone(),
            self.node_name.clone(),
            format!(
                "{} finish process complete. elapsed = {:?}, features = {}",
                self.processor.read().name(),
                now.elapsed(),
                finish_feature_count
            ),
        );

        let terminate_result = channel_manager.send_terminate(ctx);

        if result.is_err() {
            result
        } else {
            terminate_result
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn process(
    ctx: ExecutorContext,
    node_handle: NodeHandle,
    node_name: String,
    span: Span,
    event_hub: EventHub,
    channel_manager: Arc<parking_lot::RwLock<ProcessorChannelForwarder>>,
    processor: Arc<parking_lot::RwLock<Box<dyn Processor>>>,
    has_failed: Arc<std::sync::atomic::AtomicBool>,
    features_processed: Arc<AtomicU64>,
    process_duration_us: Arc<AtomicU64>,
    process_duration_sq_us: Arc<AtomicU64>,
) {
    let feature_id = ctx.feature.id;
    let diagnostics_handle = ctx.diagnostics.clone();
    let channel_manager_guard = channel_manager.read();
    let mut processor_guard = processor.write();
    let channel_manager: &ProcessorChannelForwarder = &channel_manager_guard;
    let processor: &mut Box<dyn Processor> = &mut processor_guard;
    let now = time::Instant::now();
    let result = processor.process(ctx, channel_manager);
    let elapsed = now.elapsed();
    let us = elapsed.as_micros() as u64;
    process_duration_us.fetch_add(us, std::sync::atomic::Ordering::Relaxed);
    process_duration_sq_us.fetch_add(us.saturating_mul(us), std::sync::atomic::Ordering::Relaxed);
    let name = processor.name();

    if elapsed >= *SLOW_ACTION_THRESHOLD {
        event_hub.info_log_with_node_info(
            Some(span.clone()),
            node_handle.clone(),
            node_name.clone(),
            format!(
                "Slow action, processor node name = {:?}, node_id = {}, feature id = {:?}, elapsed = {:?}",
                name,
                node_handle.id,
                feature_id,
                elapsed,
            ),
        );
    }

    if let Err(e) = result {
        has_failed.store(true, std::sync::atomic::Ordering::SeqCst);

        event_hub.error_log_with_node_info(
            Some(span.clone()),
            node_handle.clone(),
            node_name.clone(),
            format!(
                "Error operation, processor node name = {} ({}), node_id = {}, feature id = {:?}, error = {:?}",
                processor.name(),
                node_name,
                node_handle.id,
                feature_id,
                e,
            ),
        );

        event_hub.send(Event::ProcessorFailed {
            node: node_handle.clone(),
            name: node_name.clone(),
        });

        // First-wins fatal slot: without recording this, a per-feature error would never reach RunSummary.failed_nodes.
        if let Some(handle) = &diagnostics_handle {
            handle.inner.record_fatal(synthesize_process_error_fatal(
                e.to_string(),
                handle.inner.node_id().to_string(),
                handle.inner.action_type().to_string(),
                feature_id,
            ));
        }
    } else {
        features_processed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}

fn synthesize_process_error_fatal(
    message: String,
    node_id: String,
    action_type: String,
    feature_id: uuid::Uuid,
) -> Diagnostic {
    let mut diagnostic = Diagnostic::from_draft(
        DiagnosticDraft::new(ErrorCode::InternalUnclassified).with_message(message),
        Some(node_id),
        Some(action_type),
        Some(feature_id),
    );
    diagnostic.effective_disposition = Some(Disposition::Fatal);
    diagnostic
}

// Failure precedence: a real returned error wins; the fatal slot is only a backstop for swallowed report() fatals.
fn reconcile_terminate_result(
    terminate_result: Result<(), ExecutionError>,
    fatal: Option<Diagnostic>,
    has_failed: bool,
) -> (Result<(), ExecutionError>, bool, Option<Diagnostic>) {
    match (terminate_result, fatal) {
        (Err(e), fatal) => (Err(e), true, fatal),
        (Ok(()), Some(diag)) => (Err(ExecutionError::Processor(Box::new(diag))), true, None),
        (Ok(()), None) => (Ok(()), has_failed, None),
    }
}

#[cfg(test)]
mod reconcile_tests {
    use super::*;
    use reearth_flow_diagnostics::{Diagnostic, DiagnosticDraft, ErrorCode};

    fn dummy_diagnostic(message: &str) -> Diagnostic {
        Diagnostic::from_draft(
            DiagnosticDraft::new(ErrorCode::InternalInvariantViolation).with_message(message),
            None,
            None,
            None,
        )
    }

    fn terminate_err(message: &str) -> ExecutionError {
        ExecutionError::CannotSendToChannel(message.to_string())
    }

    #[test]
    fn ok_no_fatal_result_is_ok_and_node_failed_tracks_has_failed() {
        for has_failed in [false, true] {
            let (result, node_failed, superseded) =
                reconcile_terminate_result(Ok(()), None, has_failed);
            assert!(result.is_ok());
            assert_eq!(node_failed, has_failed);
            assert!(superseded.is_none());
        }
    }

    #[test]
    fn ok_with_fatal_backstop_fires_regardless_of_has_failed() {
        for has_failed in [false, true] {
            let (result, node_failed, superseded) =
                reconcile_terminate_result(Ok(()), Some(dummy_diagnostic("fatal")), has_failed);
            assert!(node_failed);
            match result {
                Err(ExecutionError::Processor(e)) => assert!(e.to_string().contains("fatal")),
                other => panic!("expected the fatal backstop to fire, got {other:?}"),
            }
            assert!(superseded.is_none());
        }
    }

    #[test]
    fn err_no_fatal_the_real_error_wins_regardless_of_has_failed() {
        for has_failed in [false, true] {
            let (result, node_failed, superseded) =
                reconcile_terminate_result(Err(terminate_err("boom")), None, has_failed);
            assert!(node_failed);
            match result {
                Err(ExecutionError::CannotSendToChannel(msg)) => assert_eq!(msg, "boom"),
                other => panic!("expected the real terminate error, got {other:?}"),
            }
            assert!(superseded.is_none());
        }
    }

    #[test]
    fn err_with_fatal_the_real_error_still_wins_over_the_fatal_backstop() {
        for has_failed in [false, true] {
            let (result, node_failed, superseded) = reconcile_terminate_result(
                Err(terminate_err("boom")),
                Some(dummy_diagnostic("fatal")),
                has_failed,
            );
            assert!(node_failed);
            match result {
                Err(ExecutionError::CannotSendToChannel(msg)) => assert_eq!(msg, "boom"),
                other => panic!(
                    "expected the real terminate error to win over the fatal backstop, got {other:?}"
                ),
            }
            let superseded = superseded.expect("fatal was present and lost to the terminate error");
            assert_eq!(superseded.message, "fatal");
        }
    }
}

#[cfg(test)]
mod synthesize_process_error_fatal_tests {
    use super::*;
    use reearth_flow_diagnostics::{Disposition, ErrorCode};

    #[test]
    fn carries_error_text_and_node_identity_as_fatal() {
        let feature_id = uuid::Uuid::new_v4();
        let diagnostic = synthesize_process_error_fatal(
            "boom".to_string(),
            "sub.node-1".to_string(),
            "AttributeAggregator".to_string(),
            feature_id,
        );

        assert_eq!(diagnostic.code, ErrorCode::InternalUnclassified);
        assert_eq!(diagnostic.message, "boom");
        assert_eq!(diagnostic.node_id.as_deref(), Some("sub.node-1"));
        assert_eq!(
            diagnostic.action_type.as_deref(),
            Some("AttributeAggregator")
        );
        assert_eq!(diagnostic.feature_id, Some(feature_id));
        assert_eq!(diagnostic.effective_disposition, Some(Disposition::Fatal));
    }
}
