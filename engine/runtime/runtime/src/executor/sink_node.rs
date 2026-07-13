use std::{
    borrow::Cow,
    fmt::Debug,
    io::BufRead,
    mem::swap,
    sync::{atomic::AtomicU64, Arc},
    time,
};

use crossbeam::channel::Receiver;
use futures::Future;
use petgraph::graph::NodeIndex;
use reearth_flow_state::State;
use reearth_flow_storage::resolve::StorageResolver;
use tokio::runtime::Handle;
use tracing::info_span;

use reearth_flow_common::uri::Uri;
use reearth_flow_diagnostics::Diagnostic;

use crate::{
    builder_dag::NodeKind,
    errors::ExecutionError,
    event::{Event, EventHub},
    executor_operation::{ExecutorContext, ExecutorOperation, NodeContext},
    kvs::KvStore,
    node::{NodeHandle, NodeStatus, Sink},
};

use super::receiver_loop::ReceiverLoop;
use super::source_intermediate::SourceIntermediateRecorder;
use super::{execution_dag::ExecutionDag, receiver_loop::init_select};

/// A sink in the execution DAG.
#[derive(Debug)]
pub struct SinkNode<F> {
    /// Node handle in description DAG.
    node_handle: NodeHandle,
    /// Node name from workflow definition.
    node_name: String,
    /// Input node handles.
    node_handles: Vec<NodeHandle>,
    /// Input data channels.
    receivers: Vec<Receiver<ExecutorOperation>>,
    /// The sink.
    sink: Box<dyn Sink>,
    event_hub: EventHub,
    /// The shutdown future.
    #[allow(dead_code)]
    shutdown: F,
    /// The runtime to run the source in.
    #[allow(dead_code)]
    runtime: Arc<Handle>,
    span: tracing::Span,
    features_written: Arc<AtomicU64>,
    env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
    storage_resolver: Arc<StorageResolver>,
    kv_store: Arc<dyn KvStore>,
    sandbox_root: Uri,
    source_intermediate_recorder: SourceIntermediateRecorder,
    /// State for writing source intermediate data
    feature_state: Arc<State>,
    incremental_mode: bool,
    /// This node's report/warn/warn_once diagnostics handle. Stamped onto
    /// every `ExecutorContext`/`NodeContext` this node receives so
    /// process()/finish()-time reports are attributed to this node.
    diagnostics: crate::diagnostics::SharedNodeDiagnostics,
    /// Sink for this node's finish()-time diagnostic summaries
    /// (`emit_summaries`'s return value), written by `on_terminate` and read
    /// by the spawning thread (`start_sink` in `dag_executor.rs`) after
    /// `run()` returns. Exists to carry a `Vec<Diagnostic>` out of the
    /// `ReceiverLoop`/`Node` traits' `Result<(), ExecutionError>`-only
    /// return type without changing either trait's signature.
    summaries_sink: Arc<parking_lot::Mutex<Vec<Diagnostic>>>,
}

impl<F: Future + Unpin + Debug> SinkNode<F> {
    pub fn new(
        ctx: NodeContext,
        dag: &mut ExecutionDag,
        node_index: NodeIndex,
        shutdown: F,
        runtime: Arc<Handle>,
        incremental_mode: bool,
        warn_once: reearth_flow_diagnostics::WarnOnceSet,
    ) -> Self {
        let node = dag.node_weight_mut(node_index);
        let Some(kind) = node.kind.take() else {
            panic!("Must pass in a node")
        };
        let node_handle = node.handle.clone();
        let node_name = node.name.clone();
        let NodeKind::Sink(sink) = kind else {
            panic!("Must pass in a sink node");
        };

        let (node_handles, receivers) = dag.collect_receivers(node_index);

        let source_intermediate_recorder =
            SourceIntermediateRecorder::collect(dag, node_index, &node_handles);
        let feature_state = dag.feature_state();

        let version = env!("CARGO_PKG_VERSION");
        let span = info_span!(
            "action",
            "engine.version" = version,
            "otel.name" = sink.name(),
            "otel.kind" = "Sink Node",
            "workflow.id" = dag.id.to_string().as_str(),
            "node.id" = node_handle.id.to_string().as_str(),
            "node.name" = node_name.as_str(),
        );
        let diagnostics = Arc::new(crate::diagnostics::NodeDiagnosticsHandle::new(
            node_handle.clone(),
            node_name.clone(),
            sink.name().to_string(),
            warn_once,
        ));
        Self {
            node_handle,
            node_name,
            node_handles,
            receivers,
            sink,
            event_hub: ctx.event_hub.clone(),
            shutdown,
            runtime,
            span,
            features_written: Arc::new(AtomicU64::new(0)),
            env_vars: ctx.env_vars.clone(),
            storage_resolver: ctx.storage_resolver.clone(),
            kv_store: ctx.kv_store.clone(),
            sandbox_root: ctx.sandbox_root.clone(),
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

    /// Clone of the handle `on_terminate` writes this node's drained
    /// finish()-time summaries into. Call before consuming `self` via
    /// `run()`/`receiver_loop()` so the summaries can still be read after.
    pub fn summaries_sink(&self) -> Arc<parking_lot::Mutex<Vec<Diagnostic>>> {
        self.summaries_sink.clone()
    }
}

impl<F: Future + Unpin + Debug> ReceiverLoop for SinkNode<F> {
    fn receivers(&mut self) -> Vec<Receiver<ExecutorOperation>> {
        let mut result = vec![];
        swap(&mut self.receivers, &mut result);
        result
    }

    fn receiver_name(&'_ self, index: usize) -> Cow<'_, str> {
        Cow::Owned(self.node_handles[index].to_string())
    }

    fn receiver_loop(mut self) -> Result<(), ExecutionError> {
        let mut has_failed = false;

        let receivers = self.receivers();
        let mut is_terminated = vec![false; receivers.len()];
        let now = time::Instant::now();
        let span = self.span.clone();
        let mut sel = init_select(&receivers);
        let mut first_error: Option<ExecutionError> = None;

        tracing::info!("Sink node {} is starting", self.node_handle.id);
        self.event_hub.send(Event::NodeStatusChanged {
            node_handle: self.node_handle.clone(),
            status: NodeStatus::Starting,
            feature_id: None,
        });

        self.event_hub.info_log_with_node_info(
            Some(span.clone()),
            self.node_handle.clone(),
            self.node_name.clone(),
            format!("{} sink start...", self.sink.name()),
        );

        let init_result = self
            .sink
            .initialize(NodeContext {
                env_vars: self.env_vars.clone(),
                kv_store: self.kv_store.clone(),
                storage_resolver: self.storage_resolver.clone(),
                event_hub: self.event_hub.clone(),
                sandbox_root: self.sandbox_root.clone(),
                diagnostics: None,
            })
            .map_err(ExecutionError::Sink);

        if let Err(ref e) = init_result {
            tracing::error!("Sink node {} initialization failed", self.node_handle.id);

            self.event_hub.error_log_with_node_info(
                Some(span.clone()),
                self.node_handle.clone(),
                self.node_name.clone(),
                format!("{} sink error: {}", self.sink.name(), e),
            );

            self.event_hub.send(Event::NodeStatusChanged {
                node_handle: self.node_handle.clone(),
                status: NodeStatus::Failed,
                feature_id: None,
            });
            return init_result;
        }

        // Log and emit Processing status
        tracing::info!("Sink node {} is processing", self.node_handle.id);
        self.event_hub.send(Event::NodeStatusChanged {
            node_handle: self.node_handle.clone(),
            status: NodeStatus::Processing,
            feature_id: None,
        });

        loop {
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

                    let result = self.on_op(ctx.clone());

                    if let Err(e) = result {
                        has_failed = true;
                        tracing::warn!(
                            "Sink node {} processing failed for feature {:?}",
                            self.node_handle.id,
                            ctx.feature.id
                        );

                        self.event_hub.error_log_with_node_info(
                            Some(span.clone()),
                            self.node_handle.clone(),
                            self.node_name.clone(),
                            format!("{} sink error: {}", self.sink.name(), e),
                        );

                        if first_error.is_none() {
                            first_error = Some(e);
                        }
                    } else {
                        self.features_written
                            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    }
                }
                ExecutorOperation::FileBackedOp {
                    path,
                    port,
                    context,
                } => {
                    let reader = match crate::forwarder::open_jsonl_reader(&path) {
                        Ok(r) => r,
                        Err(e) => {
                            has_failed = true;
                            let err = ExecutionError::CannotReceiveFromChannel(format!(
                                "Failed to open file-backed op file {}: {e}",
                                path.display()
                            ));
                            if first_error.is_none() {
                                first_error = Some(err);
                            }
                            continue;
                        }
                    };
                    for line in reader.lines() {
                        let line = match line {
                            Ok(l) => l,
                            Err(e) => {
                                has_failed = true;
                                let err = ExecutionError::CannotReceiveFromChannel(format!(
                                    "Failed to read line from file-backed op: {e}"
                                ));
                                if first_error.is_none() {
                                    first_error = Some(err);
                                }
                                break;
                            }
                        };
                        if line.is_empty() {
                            continue;
                        }
                        let feature: reearth_flow_types::Feature = match serde_json::from_str(&line)
                        {
                            Ok(f) => f,
                            Err(e) => {
                                has_failed = true;
                                let err = ExecutionError::CannotReceiveFromChannel(format!(
                                    "Failed to deserialize feature from file-backed op: {e}"
                                ));
                                if first_error.is_none() {
                                    first_error = Some(err);
                                }
                                break;
                            }
                        };
                        let ctx = ExecutorContext::new_with_context_feature_and_port(
                            &context,
                            feature,
                            port.clone(),
                        );
                        let result = self.on_op(ctx);
                        if let Err(e) = result {
                            has_failed = true;
                            if first_error.is_none() {
                                first_error = Some(e);
                            }
                        } else {
                            self.features_written
                                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        }
                    }
                }
                ExecutorOperation::Terminate { ctx } => {
                    is_terminated[index] = true;
                    sel.remove(index);
                    if is_terminated.iter().all(|value| *value) {
                        let features_count = self
                            .features_written
                            .load(std::sync::atomic::Ordering::Relaxed);
                        let message = if features_count > 0 && !has_failed {
                            format!(
                                "{} sink finish. elapsed = {:?}",
                                self.sink.name(),
                                now.elapsed()
                            )
                        } else {
                            format!(
                                "{} sink terminate. elapsed = {:?}",
                                self.sink.name(),
                                now.elapsed()
                            )
                        };

                        self.event_hub.info_log_with_node_info(
                            Some(span.clone()),
                            self.node_handle.clone(),
                            self.node_name.clone(),
                            message,
                        );

                        let terminate_result = self.on_terminate(ctx);

                        if terminate_result.is_err() && !has_failed {
                            tracing::error!("Sink node {} termination failed", self.node_handle.id);
                        }

                        // Unified failure precedence: a real error returned during
                        // processing (`first_error`) always wins; a real error from
                        // `on_terminate` (finish()/terminate-send) wins next; the
                        // fatal slot is a swallowed-fatal backstop, consulted only
                        // when nothing else failed. Exactly one
                        // NodeStatusChanged{Failed|Completed} is emitted below, from
                        // the single reconciled outcome.
                        let fatal = self.diagnostics.inner.take_fatal();
                        let (final_result, node_failed) = reconcile_sink_terminate_result(
                            first_error,
                            terminate_result,
                            fatal,
                            has_failed,
                        );

                        self.event_hub.send(Event::NodeStatusChanged {
                            node_handle: self.node_handle.clone(),
                            status: if node_failed {
                                NodeStatus::Failed
                            } else {
                                NodeStatus::Completed
                            },
                            feature_id: None,
                        });

                        return final_result;
                    }
                }
            }
        }
    }

    fn on_op(&mut self, ctx: ExecutorContext) -> Result<(), ExecutionError> {
        // Receive-site stamping: this node's own diagnostics handle wins over
        // whatever the upstream sender's context carried.
        let mut ctx = ctx;
        ctx.diagnostics = Some(self.diagnostics.clone());
        self.sink
            .process(ctx)
            .map_err(|e| crate::errors::to_node_error(e, crate::errors::NodeErrorKind::Sink))
    }

    fn on_terminate(&mut self, ctx: NodeContext) -> Result<(), ExecutionError> {
        // The incoming ctx was built by the upstream sender (or, for the
        // last-writer-wins multi-input case, whichever sender terminated
        // last) — overwrite with this node's own handle before finish() runs
        // so finish()-time drops are attributed to this node, not the sender.
        let mut ctx = ctx;
        ctx.diagnostics = Some(self.diagnostics.clone());
        let result = self
            .sink
            .finish(ctx)
            .map_err(|e| crate::errors::to_node_error(e, crate::errors::NodeErrorKind::Sink));
        // Emit this node's aggregated warn/drop/reject summaries regardless of
        // whether finish() itself succeeded — reports recorded during
        // process()/finish() must not be silently dropped just because
        // finish() failed.
        // Stashed in `summaries_sink` for the spawning thread to read after
        // `run()` returns and fold into the run's `RunSummary` (Task 5).
        let summaries = crate::diagnostics::emit_summaries(&self.event_hub, &self.diagnostics);
        *self.summaries_sink.lock() = summaries;
        self.event_hub.send(Event::SinkFinished {
            node: self.node_handle.clone(),
            name: self.node_name.clone(),
        });
        result
    }
}

/// Unified failure precedence for the sink drain end: a real returned error
/// wins; the fatal slot is only a backstop for swallowed `report()` fatals.
/// Precedence: `first_error` (from process()) > `terminate_result` Err
/// (from finish()/terminate-send) > fatal slot. Returns (final_result, node_failed).
fn reconcile_sink_terminate_result(
    first_error: Option<ExecutionError>,
    terminate_result: Result<(), ExecutionError>,
    fatal: Option<Diagnostic>,
    has_failed: bool,
) -> (Result<(), ExecutionError>, bool) {
    match (first_error, terminate_result, fatal) {
        (Some(e), _, _) => (Err(e), true),
        (None, Err(e), _) => (Err(e), true),
        (None, Ok(()), Some(diag)) => (Err(ExecutionError::Sink(Box::new(diag))), true),
        (None, Ok(()), None) => (Ok(()), has_failed),
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

    fn first_err(message: &str) -> ExecutionError {
        ExecutionError::CannotReceiveFromChannel(message.to_string())
    }

    fn term_err(message: &str) -> ExecutionError {
        ExecutionError::CannotSendToChannel(message.to_string())
    }

    #[test]
    fn no_errors_no_fatal_result_is_ok_and_node_failed_tracks_has_failed() {
        for has_failed in [false, true] {
            let (result, node_failed) =
                reconcile_sink_terminate_result(None, Ok(()), None, has_failed);
            assert!(result.is_ok());
            assert_eq!(node_failed, has_failed);
        }
    }

    #[test]
    fn fatal_backstop_fires_only_when_nothing_else_failed() {
        for has_failed in [false, true] {
            let (result, node_failed) = reconcile_sink_terminate_result(
                None,
                Ok(()),
                Some(dummy_diagnostic("fatal")),
                has_failed,
            );
            assert!(node_failed);
            match result {
                Err(ExecutionError::Sink(e)) => assert!(e.to_string().contains("fatal")),
                other => panic!("expected the fatal backstop to fire, got {other:?}"),
            }
        }
    }

    #[test]
    fn terminate_err_wins_over_fatal_when_there_is_no_first_error() {
        for fatal_present in [false, true] {
            for has_failed in [false, true] {
                let fatal = fatal_present.then(|| dummy_diagnostic("fatal"));
                let (result, node_failed) = reconcile_sink_terminate_result(
                    None,
                    Err(term_err("terminate boom")),
                    fatal,
                    has_failed,
                );
                assert!(node_failed);
                match result {
                    Err(ExecutionError::CannotSendToChannel(msg)) => {
                        assert_eq!(msg, "terminate boom")
                    }
                    other => panic!(
                        "expected the real terminate error, not the fatal backstop, got {other:?}"
                    ),
                }
            }
        }
    }

    #[test]
    fn first_error_always_wins_over_terminate_result_and_fatal() {
        for terminate_is_err in [false, true] {
            for fatal_present in [false, true] {
                for has_failed in [false, true] {
                    let terminate_result = if terminate_is_err {
                        Err(term_err("terminate boom"))
                    } else {
                        Ok(())
                    };
                    let fatal = fatal_present.then(|| dummy_diagnostic("fatal"));
                    let (result, node_failed) = reconcile_sink_terminate_result(
                        Some(first_err("first boom")),
                        terminate_result,
                        fatal,
                        has_failed,
                    );
                    assert!(node_failed);
                    match result {
                        Err(ExecutionError::CannotReceiveFromChannel(msg)) => {
                            assert_eq!(msg, "first boom")
                        }
                        other => panic!("expected first_error to win, got {other:?}"),
                    }
                }
            }
        }
    }
}
