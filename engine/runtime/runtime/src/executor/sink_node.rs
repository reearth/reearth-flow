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
    errors::{to_node_error, ExecutionError, NodeErrorKind},
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
    #[allow(clippy::too_many_arguments)]
    pub fn new(
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
        let NodeKind::Sink(sink) = kind else {
            panic!("Must pass in a sink node");
        };
        // NOTE: `action` is NOT asserted equal to `sink.name()` here. See the
        // matching note in `processor_node.rs::ProcessorNode::new`:
        // `builder_dag.rs`'s `ActionNameMismatch` check validates the
        // *factory's* `SinkFactory::name()` against `node.node.action()` at
        // build time (`action`'s provenance) — the *built instance*'s
        // `Sink::name()` is a different trait and can legitimately diverge
        // (e.g. profile-namespaced factory keys vs. a generic instance
        // display name), as proven for the processor case by the
        // quality-check workflow fixtures.
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
            "node.id" = composed_id.as_str(),
            "node.name" = node_name.as_str(),
        );
        let diagnostics = Arc::new(crate::diagnostics::NodeDiagnosticsHandle::new(
            composed_id,
            node_handle.clone(),
            node_name.clone(),
            action,
            warn_once,
            disposition_policy,
            // D7 (Task 5): every SinkNode may capture reject rows — the
            // handle itself gates capture further on `side_file()`.
            true,
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

    /// This node's `(composed_id, action)`, read off the same diagnostics
    /// handle every `report()`/`report_drop()` call resolves against.
    /// `start_sink` (`dag_executor.rs`) carries this alongside the spawned
    /// thread's `JoinHandle` so the collect-all fold can attribute a
    /// synthesized failure diagnostic to this node.
    pub fn node_meta(&self) -> super::dag_executor::NodeMeta {
        super::dag_executor::NodeMeta {
            composed_id: self.diagnostics.inner.node_id().to_string(),
            action: self.diagnostics.inner.action_type().to_string(),
        }
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
                        let (final_result, node_failed, superseded_fatal) =
                            reconcile_sink_terminate_result(
                                first_error,
                                terminate_result,
                                fatal,
                                has_failed,
                            );

                        // 2a-core T1 ledger item: a swallowed-fatal `report()`
                        // that loses precedence to a real returned error must
                        // not vanish silently — warn once, naming the
                        // superseded diagnostic's code, so operators can see
                        // both failures happened even though only one wins
                        // the final `Result`.
                        if let Some(superseded) = superseded_fatal {
                            self.event_hub.warn_log_with_node_info(
                                Some(span.clone()),
                                self.node_handle.clone(),
                                self.node_name.clone(),
                                format!(
                                    "{} sink: swallowed fatal diagnostic ({}) superseded by a \
                                     real error and dropped from the final result",
                                    self.sink.name(),
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
            .map_err(|e| to_node_error(e, NodeErrorKind::Sink))
    }

    fn on_terminate(&mut self, ctx: NodeContext) -> Result<(), ExecutionError> {
        // The incoming ctx was built by the upstream sender (or, for the
        // last-writer-wins multi-input case, whichever sender terminated
        // last) — overwrite with this node's own handle before finish() runs
        // so finish()-time drops are attributed to this node, not the sender.
        let mut ctx = ctx;
        ctx.diagnostics = Some(self.diagnostics.clone());
        let mut result = self
            .sink
            .finish(ctx)
            .map_err(|e| to_node_error(e, NodeErrorKind::Sink));
        // Emit this node's aggregated warn/drop/reject summaries regardless of
        // whether finish() itself succeeded — reports recorded during
        // process()/finish() must not be silently dropped just because
        // finish() failed.
        // Stashed in `summaries_sink` for the spawning thread to read after
        // `run()` returns and fold into the run's `RunSummary` (Task 5).
        let summaries = crate::diagnostics::emit_summaries(&self.event_hub, &self.diagnostics);
        *self.summaries_sink.lock() = summaries;

        // D7 (Task 5): flush any buffered reject rows to this node's side-
        // file shard, alongside emit_summaries above — same "always run,
        // never silently skip" shape. `drain_reject_rows` is `None` unless
        // reject capture was enabled for this node (`policy.side_file()` at
        // construction time), so this is a no-op for every no-policy or
        // side_file-disabled run. A flush failure doesn't override a real
        // finish() error (that failure already has precedence downstream in
        // `reconcile_sink_terminate_result`), but if finish() itself
        // succeeded, a flush failure still fails the node — rejected-row
        // evidence must not vanish silently.
        if let Some((rows, overflow)) = self.diagnostics.drain_reject_rows() {
            if let Err(e) = self.flush_reject_shard(rows, overflow) {
                if result.is_ok() {
                    result = Err(e);
                }
            }
        }

        self.event_hub.send(Event::SinkFinished {
            node: self.node_handle.clone(),
            name: self.node_name.clone(),
        });
        result
    }
}

impl<F> SinkNode<F> {
    /// Write this node's D7 reject-shard side file
    /// (`rejected/{composed_id}.jsonl`, spec §7) under `self.sandbox_root`.
    /// `put`/overwrite-only, one shard per node — the composed id keys the
    /// path, so distinct sink nodes never clobber each other's shard even
    /// when they share a run.
    fn flush_reject_shard(
        &self,
        rows: Vec<crate::diagnostics::RejectRow>,
        overflow: u64,
    ) -> Result<(), ExecutionError> {
        let relative_path = reject_shard_relative_path(self.diagnostics.inner.node_id());
        let bytes = crate::diagnostics::render_reject_jsonl(&rows, overflow);
        write_sandboxed_reject_shard(
            &self.sandbox_root,
            &relative_path,
            &self.storage_resolver,
            bytes::Bytes::from(bytes),
        )
    }
}

/// The D7 reject-shard's path, relative to `sandbox_root` (spec §7:
/// `rejected/{composed_id}.jsonl`). Pure/no I/O — the composed-id-keying
/// invariant (distinct nodes never share a shard) is directly unit-testable
/// without a sandbox.
fn reject_shard_relative_path(composed_id: &str) -> String {
    format!("rejected/{composed_id}.jsonl")
}

/// Minimal, runtime-crate-local re-implementation of the sandbox-join-and-
/// put invariant `action-sink::SinkOutput` (its full API, including request
/// validation only meaningful for user-authored paths) already implements
/// for every other sink write — duplicated rather than reused because
/// `action-sink` depends on this crate (`reearth-flow-runtime`, for the
/// `Sink`/`Processor`/`Source` traits every action implements), so a
/// `runtime -> action-sink` dependency the other way would be circular. The
/// *invariant* is the requirement (put/overwrite-only, resolves strictly
/// under `sandbox_root`, no `..` escape survives the join) — not the
/// specific `SinkOutput` type — so this reimplements only that: no leading-
/// `/`/`~`/whitespace/empty-path hygiene checks, since `relative_path` here
/// is always program-built (`reject_shard_relative_path`), never directly
/// user-authored.
fn write_sandboxed_reject_shard(
    sandbox_root: &Uri,
    relative_path: &str,
    resolver: &StorageResolver,
    bytes: bytes::Bytes,
) -> Result<(), ExecutionError> {
    let resolved = sandbox_root.join(relative_path).map_err(|e| {
        ExecutionError::Sink(
            format!("reject side-file: failed to join {relative_path:?} with sandbox_root: {e}")
                .into(),
        )
    })?;
    ensure_under_sandbox(sandbox_root, &resolved)?;
    let storage = resolver.resolve(&resolved).map_err(|e| {
        ExecutionError::Sink(
            format!("reject side-file: failed to resolve storage for {resolved}: {e}").into(),
        )
    })?;
    storage
        .put_sync(resolved.path().as_path(), bytes)
        .map_err(|e| {
            ExecutionError::Sink(
                format!("reject side-file: failed to write {resolved}: {e}").into(),
            )
        })?;
    Ok(())
}

/// Segment-aligned "resolved is under sandbox_root" check, mirroring
/// `action-sink::sandbox::ensure_under` (see `write_sandboxed_reject_shard`
/// for why this is duplicated rather than reused). `file:///` is treated as
/// "no sandbox" — same permissive sentinel `NodeContext::default()`/
/// `ExecutorOptions::default()` use for tests — otherwise any `..` segment
/// surviving the join, or a resolved URI that isn't a same-scheme/authority
/// prefix of `sandbox_root`, is rejected.
fn ensure_under_sandbox(sandbox_root: &Uri, resolved: &Uri) -> Result<(), ExecutionError> {
    if sandbox_root.as_str() == "file:///" || sandbox_root.as_str() == resolved.as_str() {
        return Ok(());
    }
    let root_prefix = sandbox_root.as_str().trim_end_matches('/');
    let candidate_str = resolved.as_str();
    let after_prefix = candidate_str.strip_prefix(root_prefix).ok_or_else(|| {
        ExecutionError::Sink(
            format!("reject side-file {resolved} is outside the sandbox root {sandbox_root}")
                .into(),
        )
    })?;
    let escapes = (!after_prefix.is_empty() && !after_prefix.starts_with('/'))
        || after_prefix.split('/').any(|segment| segment == "..");
    if escapes {
        return Err(ExecutionError::Sink(
            format!("reject side-file {resolved} is outside the sandbox root {sandbox_root}")
                .into(),
        ));
    }
    Ok(())
}

/// Unified failure precedence for the sink drain end: a real returned error
/// wins; the fatal slot is only a backstop for swallowed `report()` fatals.
/// Precedence: `first_error` (from process()) > `terminate_result` Err
/// (from finish()/terminate-send) > fatal slot.
///
/// Returns `(final_result, node_failed, superseded_fatal)`. `superseded_fatal`
/// is `Some(diag)` exactly when a swallowed fatal was present but lost
/// precedence to a real returned error (the first two match arms) — i.e. it
/// was recorded via `ctx.fatal()`/`report()` but never surfaces in
/// `final_result`. The 2a-core T1 ledger item: this must not vanish
/// silently, so the call site (`receiver_loop`) warns once using this
/// indicator. It is `None` both when there is no fatal to begin with and
/// when the fatal slot itself wins (it's already reflected in
/// `final_result` there, so nothing was superseded).
fn reconcile_sink_terminate_result(
    first_error: Option<ExecutionError>,
    terminate_result: Result<(), ExecutionError>,
    fatal: Option<Diagnostic>,
    has_failed: bool,
) -> (Result<(), ExecutionError>, bool, Option<Diagnostic>) {
    match (first_error, terminate_result, fatal) {
        (Some(e), _, fatal) => (Err(e), true, fatal),
        (None, Err(e), fatal) => (Err(e), true, fatal),
        (None, Ok(()), Some(diag)) => (Err(ExecutionError::Sink(Box::new(diag))), true, None),
        (None, Ok(()), None) => (Ok(()), has_failed, None),
    }
}

#[cfg(test)]
mod reject_shard_tests {
    use std::str::FromStr;

    use tempfile::tempdir;

    use super::*;

    /// Composed-id keying (Task 5 brief): two distinct composed ids produce
    /// two distinct shard paths, each keyed under `rejected/`.
    #[test]
    fn reject_shard_relative_path_keys_by_composed_id() {
        assert_eq!(
            reject_shard_relative_path("writer-a"),
            "rejected/writer-a.jsonl"
        );
        assert_eq!(
            reject_shard_relative_path("prefix.writer-b"),
            "rejected/prefix.writer-b.jsonl"
        );
        assert_ne!(
            reject_shard_relative_path("writer-a"),
            reject_shard_relative_path("prefix.writer-b")
        );
    }

    fn file_uri(path: &std::path::Path) -> Uri {
        Uri::from_str(&format!("file://{}", path.display())).unwrap()
    }

    #[test]
    fn write_sandboxed_reject_shard_writes_under_the_sandbox_root() {
        let tmp = tempdir().unwrap();
        let root = file_uri(tmp.path());
        let resolver = StorageResolver::new();
        write_sandboxed_reject_shard(
            &root,
            "rejected/node-a.jsonl",
            &resolver,
            bytes::Bytes::from_static(b"{\"featureId\":null}\n"),
        )
        .expect("write should succeed under the sandbox root");
        let content = std::fs::read_to_string(tmp.path().join("rejected/node-a.jsonl")).unwrap();
        assert_eq!(content, "{\"featureId\":null}\n");
    }

    /// Two distinct shard paths under the same sandbox never clobber each
    /// other — the reject-shard no-clobber invariant at the write-helper
    /// level (the runner-level test covers it end to end through a real
    /// two-sink run).
    #[test]
    fn write_sandboxed_reject_shard_does_not_clobber_a_sibling_shard() {
        let tmp = tempdir().unwrap();
        let root = file_uri(tmp.path());
        let resolver = StorageResolver::new();
        write_sandboxed_reject_shard(
            &root,
            &reject_shard_relative_path("writer-a"),
            &resolver,
            bytes::Bytes::from_static(b"a\n"),
        )
        .unwrap();
        write_sandboxed_reject_shard(
            &root,
            &reject_shard_relative_path("writer-b"),
            &resolver,
            bytes::Bytes::from_static(b"b\n"),
        )
        .unwrap();
        assert_eq!(
            std::fs::read_to_string(tmp.path().join("rejected/writer-a.jsonl")).unwrap(),
            "a\n"
        );
        assert_eq!(
            std::fs::read_to_string(tmp.path().join("rejected/writer-b.jsonl")).unwrap(),
            "b\n"
        );
    }

    #[test]
    fn write_sandboxed_reject_shard_rejects_traversal_outside_the_sandbox() {
        let tmp = tempdir().unwrap();
        let root = file_uri(tmp.path());
        let resolver = StorageResolver::new();
        let err = write_sandboxed_reject_shard(
            &root,
            "../../escape.jsonl",
            &resolver,
            bytes::Bytes::from_static(b"x\n"),
        )
        .expect_err("a path escaping the sandbox root must be rejected");
        assert!(matches!(err, ExecutionError::Sink(_)));
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
            let (result, node_failed, superseded) =
                reconcile_sink_terminate_result(None, Ok(()), None, has_failed);
            assert!(result.is_ok());
            assert_eq!(node_failed, has_failed);
            assert!(superseded.is_none());
        }
    }

    #[test]
    fn fatal_backstop_fires_only_when_nothing_else_failed() {
        for has_failed in [false, true] {
            let (result, node_failed, superseded) = reconcile_sink_terminate_result(
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
            // The fatal slot itself won here (it's reflected in `result`),
            // so nothing was superseded — no WARN should fire for this case.
            assert!(superseded.is_none());
        }
    }

    #[test]
    fn terminate_err_wins_over_fatal_when_there_is_no_first_error() {
        for fatal_present in [false, true] {
            for has_failed in [false, true] {
                let fatal = fatal_present.then(|| dummy_diagnostic("fatal"));
                let (result, node_failed, superseded) = reconcile_sink_terminate_result(
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
                // A real terminate error superseded the fatal slot whenever
                // one was actually present.
                assert_eq!(superseded.is_some(), fatal_present);
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
                    let (result, node_failed, superseded) = reconcile_sink_terminate_result(
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
                    // `first_error` superseded the fatal slot whenever one
                    // was actually present, regardless of terminate_result.
                    assert_eq!(superseded.is_some(), fatal_present);
                }
            }
        }
    }

    /// The 2a-core T1 ledger item, unit-tested directly on the reconcile
    /// helper: when a swallowed fatal loses to a real error, the returned
    /// indicator must carry the SAME diagnostic that was superseded (not
    /// just `Some(_)`) so the call site's WARN can name its code.
    #[test]
    fn superseded_fatal_indicator_carries_the_original_diagnostic() {
        let fatal = dummy_diagnostic("fatal");
        let (_, _, superseded) = reconcile_sink_terminate_result(
            Some(first_err("first boom")),
            Ok(()),
            Some(fatal),
            false,
        );
        let superseded = superseded.expect("fatal was present and lost to first_error");
        assert_eq!(superseded.code, ErrorCode::InternalInvariantViolation);
        assert_eq!(superseded.message, "fatal");
    }
}
