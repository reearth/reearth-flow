use std::path::PathBuf;
use std::sync::Arc;

use reearth_flow_common::uri::Uri;
use reearth_flow_diagnostics::{
    Diagnostic, DiagnosticDraft, DiagnosticKind, Disposition, DispositionPolicy, ErrorCode,
};
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::Feature;
use tracing::{error_span, info_span};

use crate::{
    event::EventHub,
    kvs::KvStore,
    node::{Port, FEATURES_PORT},
};

#[derive(Clone, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum ExecutorOperation {
    Op {
        ctx: ExecutorContext,
    },
    FileBackedOp {
        path: PathBuf,
        port: Port,
        context: Context,
    },
    Terminate {
        ctx: NodeContext,
    },
}

#[derive(Debug, Clone)]
pub struct Context {
    pub env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
    pub storage_resolver: Arc<StorageResolver>,
    pub kv_store: Arc<dyn KvStore>,
    pub event_hub: EventHub,
    /// Per-job sandbox root for sink writes. Production callers (worker, CLI)
    /// MUST set this to the resolved workerArtifactPath URI; production
    /// entrypoints (`Runner::run_with_sandbox_root`) reject the `file:///`
    /// sentinel. Tests using `NodeContext::default()` get `file:///`, which
    /// `sandbox::ensure_under` treats as "no sandbox" for any candidate scheme.
    pub sandbox_root: Uri,
    /// Per-node diagnostics handle for the `report`/`warn`/`warn_once` API.
    /// `None` for fresh/legacy contexts; derived contexts propagate it from
    /// their source.
    pub diagnostics: Option<crate::diagnostics::SharedNodeDiagnostics>,
}

impl From<ExecutorContext> for Context {
    fn from(ctx: ExecutorContext) -> Self {
        Self {
            env_vars: ctx.env_vars,
            storage_resolver: ctx.storage_resolver,
            kv_store: ctx.kv_store,
            event_hub: ctx.event_hub,
            sandbox_root: ctx.sandbox_root,
            diagnostics: ctx.diagnostics,
        }
    }
}

impl From<NodeContext> for Context {
    fn from(ctx: NodeContext) -> Self {
        Self {
            env_vars: ctx.env_vars,
            storage_resolver: ctx.storage_resolver,
            kv_store: ctx.kv_store,
            event_hub: ctx.event_hub,
            sandbox_root: ctx.sandbox_root,
            diagnostics: ctx.diagnostics,
        }
    }
}

impl Context {
    pub fn new(
        env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
        storage_resolver: Arc<StorageResolver>,
        kv_store: Arc<dyn KvStore>,
        event_hub: EventHub,
        sandbox_root: Uri,
    ) -> Self {
        Self {
            env_vars,
            storage_resolver,
            kv_store,
            event_hub,
            sandbox_root,
            diagnostics: None,
        }
    }

    pub fn as_executor_context(&self, feature: Feature, port: Port) -> ExecutorContext {
        ExecutorContext {
            feature,
            port,
            env_vars: self.env_vars.clone(),
            storage_resolver: self.storage_resolver.clone(),
            kv_store: self.kv_store.clone(),
            event_hub: self.event_hub.clone(),
            sandbox_root: self.sandbox_root.clone(),
            diagnostics: self.diagnostics.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecutorContext {
    pub feature: Feature,
    pub port: Port,
    pub env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
    pub storage_resolver: Arc<StorageResolver>,
    pub kv_store: Arc<dyn KvStore>,
    pub event_hub: EventHub,
    /// Per-job sandbox root for sink writes. Production callers (worker, CLI)
    /// MUST set this to the resolved workerArtifactPath URI; production
    /// entrypoints (`Runner::run_with_sandbox_root`) reject the `file:///`
    /// sentinel. Tests using `NodeContext::default()` get `file:///`, which
    /// `sandbox::ensure_under` treats as "no sandbox" for any candidate scheme.
    pub sandbox_root: Uri,
    /// Per-node diagnostics handle for the `report`/`warn`/`warn_once` API.
    /// `None` for fresh/legacy contexts; derived contexts propagate it from
    /// their source.
    pub diagnostics: Option<crate::diagnostics::SharedNodeDiagnostics>,
}

impl ExecutorContext {
    pub fn new(
        feature: Feature,
        port: Port,
        env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
        storage_resolver: Arc<StorageResolver>,
        kv_store: Arc<dyn KvStore>,
        event_hub: EventHub,
        sandbox_root: Uri,
    ) -> Self {
        Self {
            feature,
            port,
            env_vars,
            storage_resolver,
            kv_store,
            event_hub,
            sandbox_root,
            diagnostics: None,
        }
    }

    pub fn as_context(&self) -> Context {
        Context {
            env_vars: self.env_vars.clone(),
            storage_resolver: self.storage_resolver.clone(),
            kv_store: self.kv_store.clone(),
            event_hub: self.event_hub.clone(),
            sandbox_root: self.sandbox_root.clone(),
            diagnostics: self.diagnostics.clone(),
        }
    }

    pub fn new_with_feature_and_port(&self, feature: Feature, port: Port) -> Self {
        Self {
            feature,
            port,
            env_vars: Arc::clone(&self.env_vars),
            storage_resolver: Arc::clone(&self.storage_resolver),
            kv_store: Arc::clone(&self.kv_store),
            event_hub: self.event_hub.clone(),
            sandbox_root: self.sandbox_root.clone(),
            diagnostics: self.diagnostics.clone(),
        }
    }

    pub fn new_with_node_context_feature_and_port(
        ctx: &NodeContext,
        feature: Feature,
        port: Port,
    ) -> Self {
        Self {
            feature,
            port,
            env_vars: Arc::clone(&ctx.env_vars),
            storage_resolver: Arc::clone(&ctx.storage_resolver),
            kv_store: Arc::clone(&ctx.kv_store),
            event_hub: ctx.event_hub.clone(),
            sandbox_root: ctx.sandbox_root.clone(),
            diagnostics: ctx.diagnostics.clone(),
        }
    }

    pub fn new_with_context_feature_and_port(ctx: &Context, feature: Feature, port: Port) -> Self {
        Self {
            feature,
            port,
            env_vars: Arc::clone(&ctx.env_vars),
            storage_resolver: Arc::clone(&ctx.storage_resolver),
            kv_store: Arc::clone(&ctx.kv_store),
            event_hub: ctx.event_hub.clone(),
            sandbox_root: ctx.sandbox_root.clone(),
            diagnostics: ctx.diagnostics.clone(),
        }
    }

    pub fn new_with_features_port(
        feature: Feature,
        env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
        storage_resolver: Arc<StorageResolver>,
        kv_store: Arc<dyn KvStore>,
        event_hub: EventHub,
        sandbox_root: Uri,
    ) -> Self {
        Self {
            feature,
            port: FEATURES_PORT.clone(),
            env_vars,
            storage_resolver,
            kv_store,
            event_hub,
            sandbox_root,
            diagnostics: None,
        }
    }

    pub fn info_span(&self) -> tracing::Span {
        info_span!("action")
    }

    pub fn error_span(&self) -> tracing::Span {
        error_span!("action")
    }
}

impl ExecutorContext {
    fn diagnostic_identity(&self) -> (Option<String>, Option<String>) {
        match &self.diagnostics {
            Some(handle) => (
                Some(handle.inner.node_id().to_string()),
                Some(handle.inner.action_type().to_string()),
            ),
            None => (None, None),
        }
    }

    fn emit_immediate_warn(&self, diagnostic: &Diagnostic) {
        self.event_hub.diagnostic(diagnostic.clone());
    }

    /// Report a feature-disposition decision (drop / reject / fail).
    /// Auto-injects node_id, action_type and the current feature id.
    /// Fatal is returned as Err(diagnostic) — `ctx.report(draft)?` is the
    /// idiomatic call shape — but the no-silent-fatal guarantee is executor-side:
    /// the fatal is recorded in the per-node slot BEFORE Err is returned, and the
    /// executor fails the node at drain end even if the action swallowed the Err.
    // `Diagnostic` is the deliberate error type here (actions match on
    // `effective_disposition`/`?`-propagate it as a `BoxedError`); boxing it
    // would break the interface this task specifies.
    #[allow(clippy::result_large_err)]
    pub fn report(&self, draft: DiagnosticDraft) -> Result<Disposition, Diagnostic> {
        let (node_id, action_type) = self.diagnostic_identity();
        let mut diagnostic =
            Diagnostic::from_draft(draft, node_id, action_type, Some(self.feature.id));
        // The compiled policy's resolve() ladder decides the effective
        // disposition when this context has a diagnostics handle (i.e. a
        // real node context); a handle-less context (tests/legacy paths)
        // has no composed node id or policy to resolve against, so it
        // falls back to the registry default — matching `Default::default()`
        // for `DispositionPolicy`, which always resolves every code to its
        // registry default anyway.
        let effective = match &self.diagnostics {
            Some(handle) => handle.resolve(diagnostic.code),
            None => diagnostic.default_disposition,
        };
        diagnostic.effective_disposition = Some(effective);
        match effective {
            Disposition::Fatal => {
                if let Some(handle) = &self.diagnostics {
                    handle.inner.record_fatal(diagnostic.clone());
                }
                Err(diagnostic)
            }
            Disposition::WarnDrop | Disposition::Reject => {
                let kind = if effective == Disposition::WarnDrop {
                    DiagnosticKind::WarnDrop
                } else {
                    DiagnosticKind::Reject
                };
                match &self.diagnostics {
                    Some(handle) => {
                        handle
                            .inner
                            .record(kind, diagnostic.code, Some(self.feature.id));
                        // D7 (Task 5): capture a side-file row alongside the
                        // aggregation bucket above. `record_reject_row` is a
                        // no-op unless this handle belongs to a sink node
                        // under a `side_file()` policy, so this is free on
                        // every other path. `self.feature` is live here (this
                        // is the per-feature report() path), so
                        // `has_geometry` is always known (`Some`, computed
                        // for real rather than guessed). Finish()-time
                        // `report_drop` (`NodeContext::report_drop` below)
                        // threads the same tri-state field through, but its
                        // callers often have no live `Feature` and may pass
                        // `None` instead.
                        if effective == Disposition::Reject {
                            handle.record_reject_row(
                                Some(self.feature.id),
                                Some(self.feature.has_geometry()),
                                diagnostic.code,
                            );
                        }
                    }
                    // never silent, even on a context without a handle (tests/legacy paths)
                    None => self.emit_immediate_warn(&diagnostic),
                }
                Ok(effective)
            }
        }
    }

    /// Warn-and-continue: the feature keeps flowing untouched. Aggregated per
    /// node keyed on code; one finish() summary per code. Never fails.
    pub fn warn(&self, draft: DiagnosticDraft) {
        let (node_id, action_type) = self.diagnostic_identity();
        let diagnostic = Diagnostic::from_draft(draft, node_id, action_type, Some(self.feature.id));
        match &self.diagnostics {
            Some(handle) => handle.inner.record(
                DiagnosticKind::WarnContinue,
                diagnostic.code,
                Some(self.feature.id),
            ),
            None => self.emit_immediate_warn(&diagnostic),
        }
    }

    /// Run-level notice: one immediate line per run per code, bypasses the aggregator.
    /// On a context without a diagnostics handle (`self.diagnostics == None`),
    /// there is no dedup ledger to consult, so `first` is unconditionally
    /// `true` and this degrades to warn-every-time instead of once-per-run;
    /// that's a test-only/legacy situation — production contexts are always
    /// stamped with a diagnostics handle.
    pub fn warn_once(&self, draft: DiagnosticDraft) {
        let first = match &self.diagnostics {
            Some(handle) => handle.inner.try_mark_warn_once(draft.code),
            None => true,
        };
        if !first {
            return;
        }
        let (node_id, action_type) = self.diagnostic_identity();
        let diagnostic = Diagnostic::from_draft(draft, node_id, action_type, None);
        self.emit_immediate_warn(&diagnostic);
    }
}

#[derive(Debug, Clone)]
pub struct NodeContext {
    pub env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
    pub storage_resolver: Arc<StorageResolver>,
    pub kv_store: Arc<dyn KvStore>,
    pub event_hub: EventHub,
    /// Per-job sandbox root for sink writes. Production callers (worker, CLI)
    /// MUST set this to the resolved workerArtifactPath URI; production
    /// entrypoints (`Runner::run_with_sandbox_root`) reject the `file:///`
    /// sentinel. Tests using `NodeContext::default()` get `file:///`, which
    /// `sandbox::ensure_under` treats as "no sandbox" for any candidate scheme.
    pub sandbox_root: Uri,
    /// Per-node diagnostics handle for the `report`/`warn`/`warn_once` API.
    /// `None` for fresh/legacy contexts; derived contexts propagate it from
    /// their source.
    pub diagnostics: Option<crate::diagnostics::SharedNodeDiagnostics>,
}

impl From<Context> for NodeContext {
    fn from(ctx: Context) -> Self {
        Self {
            env_vars: ctx.env_vars,
            storage_resolver: ctx.storage_resolver,
            kv_store: ctx.kv_store,
            event_hub: ctx.event_hub,
            sandbox_root: ctx.sandbox_root,
            diagnostics: ctx.diagnostics,
        }
    }
}

impl From<ExecutorContext> for NodeContext {
    fn from(ctx: ExecutorContext) -> Self {
        Self {
            env_vars: ctx.env_vars,
            storage_resolver: ctx.storage_resolver,
            kv_store: ctx.kv_store,
            event_hub: ctx.event_hub,
            sandbox_root: ctx.sandbox_root,
            diagnostics: ctx.diagnostics,
        }
    }
}

impl Default for NodeContext {
    fn default() -> Self {
        Self {
            env_vars: Arc::new(serde_json::Map::new()),
            storage_resolver: Arc::new(StorageResolver::new()),
            kv_store: Arc::new(crate::kvs::create_kv_store()),
            event_hub: EventHub::new(30),
            // Permissive sentinel: `file:///` is treated by
            // `sandbox::ensure_under` as "no sandbox" — any candidate URI,
            // regardless of scheme, passes. Only used by tests / the legacy
            // `Runner::run` path; production entrypoints reject this value.
            sandbox_root: std::str::FromStr::from_str("file:///")
                .expect("'file:///' is always a valid URI"),
            diagnostics: None,
        }
    }
}

impl NodeContext {
    pub fn new(
        env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
        storage_resolver: Arc<StorageResolver>,
        kv_store: Arc<dyn KvStore>,
        event_hub: EventHub,
        sandbox_root: Uri,
    ) -> Self {
        Self {
            env_vars,
            storage_resolver,
            kv_store,
            event_hub,
            sandbox_root,
            diagnostics: None,
        }
    }

    pub fn info_span(&self) -> tracing::Span {
        info_span!("action")
    }

    pub fn error_span(&self) -> tracing::Span {
        error_span!("action")
    }

    pub fn as_context(&self) -> Context {
        Context {
            env_vars: self.env_vars.clone(),
            storage_resolver: self.storage_resolver.clone(),
            kv_store: self.kv_store.clone(),
            event_hub: self.event_hub.clone(),
            sandbox_root: self.sandbox_root.clone(),
            diagnostics: self.diagnostics.clone(),
        }
    }
}

impl NodeContext {
    /// finish()-time drop reporting (sinks run finish with only a NodeContext).
    /// `has_geometry` threads through to `NodeDiagnosticsHandle::report_drop`
    /// verbatim (tri-state: `None` = unknown, not `false` — see
    /// `RejectRow`'s doc comment); the no-handle fallback below doesn't need
    /// it, since it never captures a side-file row.
    pub fn report_drop(
        &self,
        code: ErrorCode,
        feature_id: Option<uuid::Uuid>,
        has_geometry: Option<bool>,
    ) {
        match &self.diagnostics {
            Some(handle) => handle.report_drop(code, feature_id, has_geometry),
            None => {
                // Never silent even on a context without a handle
                // (tests/legacy paths), same guarantee as `report()`.
                let diagnostic =
                    Diagnostic::from_draft(DiagnosticDraft::new(code), None, None, feature_id);
                self.event_hub.diagnostic(diagnostic);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecutorOptions {
    pub channel_buffer_sz: usize,
    pub event_hub_capacity: usize,
    pub thread_pool_size: usize,
    pub feature_flush_threshold: usize,
    /// Per-job sandbox root for sink writes, wired from the worker's resolved
    /// `workerArtifactPath`. CLI callers set this too (Task 5). Tests and legacy
    /// callers that do not set it via the builder will get the permissive
    /// `file:///` sentinel through `ExecutorOptions::default()`.
    pub sandbox_root: Uri,
    /// The workflow's compiled `errorPolicy` (`DispositionPolicy::compile`,
    /// done once at load by the runner before DAG construction — see
    /// `reearth_flow_runner::orchestrator`). `DagExecutor::start` clones this
    /// into every node's `NodeDiagnosticsHandle`, which resolves against it
    /// on every `report()`/`report_drop()` call. Defaults to
    /// `DispositionPolicy::default()`, the empty policy that resolves every
    /// code to its registry default — i.e. byte-identical to Phase 1 for any
    /// workflow with no `errorPolicy` block.
    pub disposition_policy: std::sync::Arc<DispositionPolicy>,
}

impl Default for ExecutorOptions {
    fn default() -> Self {
        Self {
            channel_buffer_sz: 256,
            event_hub_capacity: 8192,
            thread_pool_size: 30,
            feature_flush_threshold: 512,
            // Permissive sentinel — same as NodeContext::default().
            // Production callers must override this with the real artifact URI.
            sandbox_root: std::str::FromStr::from_str("file:///")
                .expect("'file:///' is always a valid URI"),
            disposition_policy: std::sync::Arc::new(DispositionPolicy::default()),
        }
    }
}

#[cfg(test)]
mod diagnostics_tests {
    use std::sync::Arc;

    use indexmap::IndexMap;
    use reearth_flow_diagnostics::{
        DiagnosticDraft, Disposition, DispositionPolicy, ErrorCode, OverrideInput, PolicyInput,
    };
    use reearth_flow_types::{AttributeValue, Feature};

    use crate::diagnostics::NodeDiagnosticsHandle;
    use crate::executor_operation::{ExecutorContext, NodeContext};
    use crate::node::{NodeHandle, NodeId, FEATURES_PORT};

    const COMPOSED_ID: &str = "node-1";

    fn ctx_with_handle() -> (ExecutorContext, crate::diagnostics::SharedNodeDiagnostics) {
        ctx_with_policy(Arc::default())
    }

    fn ctx_with_policy(
        disposition_policy: Arc<DispositionPolicy>,
    ) -> (ExecutorContext, crate::diagnostics::SharedNodeDiagnostics) {
        ctx_with_policy_and_sink(disposition_policy, false)
    }

    fn ctx_with_policy_and_sink(
        disposition_policy: Arc<DispositionPolicy>,
        is_sink: bool,
    ) -> (ExecutorContext, crate::diagnostics::SharedNodeDiagnostics) {
        let handle = Arc::new(NodeDiagnosticsHandle::new(
            COMPOSED_ID.to_string(),
            NodeHandle::new(NodeId::new("node-1".to_string())),
            "writer-1".to_string(),
            "Cesium 3D Tiles Writer".to_string(),
            Arc::default(),
            disposition_policy,
            is_sink,
        ));
        let node_ctx = NodeContext::default();
        let mut ctx = ExecutorContext::new_with_node_context_feature_and_port(
            &node_ctx,
            Feature::from(IndexMap::<String, AttributeValue>::new()),
            FEATURES_PORT.clone(),
        );
        ctx.diagnostics = Some(handle.clone());
        (ctx, handle)
    }

    fn override_node_code(code: &str, disposition: Disposition) -> OverrideInput {
        OverrideInput {
            node: Some(COMPOSED_ID.to_string()),
            code: Some(code.to_string()),
            category: None,
            disposition,
        }
    }

    #[test]
    fn report_warn_drop_increments_bucket_and_returns_disposition() {
        let (ctx, handle) = ctx_with_handle();
        let disp = ctx
            .report(DiagnosticDraft::new(ErrorCode::Cesium3dtilesEmptyGeometry))
            .unwrap();
        assert_eq!(disp, Disposition::WarnDrop);
        let summaries = handle.inner.drain_summaries();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].aggregated.as_ref().unwrap().count, 1);
        // node identity was auto-injected
        assert_eq!(summaries[0].node_id.as_deref(), Some("node-1"));
        assert_eq!(
            summaries[0].action_type.as_deref(),
            Some("Cesium 3D Tiles Writer")
        );
    }

    #[test]
    fn report_fatal_records_slot_before_returning_err_even_if_swallowed() {
        let (ctx, handle) = ctx_with_handle();
        // deliberately swallow the Err — the no-silent-fatal guarantee is executor-side
        let _ = ctx.report(DiagnosticDraft::new(ErrorCode::InternalInvariantViolation));
        let fatal = handle.inner.take_fatal().expect("fatal slot must be set");
        assert_eq!(fatal.effective_disposition, Some(Disposition::Fatal));
        assert_eq!(fatal.feature_id, Some(ctx.feature.id));
    }

    #[test]
    fn report_resolves_a_promoting_override_to_reject_instead_of_the_registry_default() {
        let policy = DispositionPolicy::compile(PolicyInput {
            overrides: vec![override_node_code(
                "cesium3dtiles.empty_geometry",
                Disposition::Reject,
            )],
            ..Default::default()
        })
        .expect("policy should compile");
        let (ctx, handle) = ctx_with_policy(Arc::new(policy));
        let disp = ctx
            .report(DiagnosticDraft::new(ErrorCode::Cesium3dtilesEmptyGeometry))
            .unwrap();
        assert_eq!(disp, Disposition::Reject);
        let summaries = handle.inner.drain_summaries();
        assert_eq!(summaries.len(), 1);
        assert_eq!(
            summaries[0].effective_disposition,
            Some(Disposition::Reject)
        );
    }

    /// D7 (Task 5): a resolved `Reject` under a sink handle with
    /// `side_file()` enabled captures a row (feature id + has_geometry +
    /// code), not just the aggregation bucket.
    #[test]
    fn report_resolves_a_promoting_override_to_reject_and_captures_a_side_file_row() {
        let policy = DispositionPolicy::compile(PolicyInput {
            side_file: true,
            overrides: vec![override_node_code(
                "cesium3dtiles.empty_geometry",
                Disposition::Reject,
            )],
            ..Default::default()
        })
        .expect("policy should compile");
        let (ctx, handle) = ctx_with_policy_and_sink(Arc::new(policy), true);
        let disp = ctx
            .report(DiagnosticDraft::new(ErrorCode::Cesium3dtilesEmptyGeometry))
            .unwrap();
        assert_eq!(disp, Disposition::Reject);
        let (rows, overflow) = handle
            .drain_reject_rows()
            .expect("a reject row should have been captured");
        assert_eq!(overflow, 0);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].feature_id, Some(ctx.feature.id));
        // ctx_with_policy's feature has no geometry set.
        assert_eq!(rows[0].has_geometry, Some(false));
        assert_eq!(rows[0].code, ErrorCode::Cesium3dtilesEmptyGeometry);
    }

    /// Same promoting override, but without `side_file()` — the aggregation
    /// bucket still records `Reject` (unchanged, pre-Task-5 behavior); no
    /// row is captured since there's nowhere configured to flush it.
    #[test]
    fn report_resolves_a_promoting_override_to_reject_without_side_file_captures_no_row() {
        let policy = DispositionPolicy::compile(PolicyInput {
            overrides: vec![override_node_code(
                "cesium3dtiles.empty_geometry",
                Disposition::Reject,
            )],
            ..Default::default()
        })
        .expect("policy should compile");
        let (ctx, handle) = ctx_with_policy_and_sink(Arc::new(policy), true);
        let disp = ctx
            .report(DiagnosticDraft::new(ErrorCode::Cesium3dtilesEmptyGeometry))
            .unwrap();
        assert_eq!(disp, Disposition::Reject);
        assert!(handle.drain_reject_rows().is_none());
    }

    /// A `WarnDrop` resolution never captures a side-file row, even under a
    /// sink + `side_file()` handle — only `Reject` does.
    #[test]
    fn report_warn_drop_never_captures_a_side_file_row_even_with_side_file_policy() {
        let (ctx, handle) = ctx_with_policy_and_sink(side_file_only_policy(), true);
        let disp = ctx
            .report(DiagnosticDraft::new(ErrorCode::Cesium3dtilesEmptyGeometry))
            .unwrap();
        assert_eq!(disp, Disposition::WarnDrop);
        assert!(handle.drain_reject_rows().is_none());
    }

    fn side_file_only_policy() -> Arc<DispositionPolicy> {
        Arc::new(
            DispositionPolicy::compile(PolicyInput {
                side_file: true,
                ..Default::default()
            })
            .expect("policy should compile"),
        )
    }

    #[test]
    fn report_resolves_a_promoting_override_to_fatal_and_records_the_slot() {
        let policy = DispositionPolicy::compile(PolicyInput {
            overrides: vec![override_node_code(
                "cesium3dtiles.empty_geometry",
                Disposition::Fatal,
            )],
            ..Default::default()
        })
        .expect("policy should compile");
        let (ctx, handle) = ctx_with_policy(Arc::new(policy));
        let err = ctx
            .report(DiagnosticDraft::new(ErrorCode::Cesium3dtilesEmptyGeometry))
            .expect_err("resolved Fatal must return Err");
        assert_eq!(err.effective_disposition, Some(Disposition::Fatal));
        let fatal = handle.inner.take_fatal().expect("fatal slot must be set");
        assert_eq!(fatal.code, ErrorCode::Cesium3dtilesEmptyGeometry);
    }

    #[test]
    fn warn_feeds_warn_continue_bucket_and_never_fails() {
        let (ctx, handle) = ctx_with_handle();
        ctx.warn(DiagnosticDraft::new(ErrorCode::GltfZeroFaceSolid));
        ctx.warn(DiagnosticDraft::new(ErrorCode::GltfZeroFaceSolid));
        let summaries = handle.inner.drain_summaries();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].aggregated.as_ref().unwrap().count, 2);
        assert_eq!(summaries[0].effective_disposition, None);
    }

    #[test]
    fn warn_once_emits_immediately_and_only_once() {
        let (ctx, handle) = ctx_with_handle();
        let mut receiver = ctx.event_hub.sender.subscribe();
        ctx.warn_once(DiagnosticDraft::new(ErrorCode::GltfZeroFaceSolid));
        ctx.warn_once(DiagnosticDraft::new(ErrorCode::GltfZeroFaceSolid));
        // exactly one immediate Event::Diagnostic, no twin Event::Log, nothing aggregated
        let first = receiver.try_recv().expect("one immediate diagnostic event");
        assert!(matches!(first, crate::event::Event::Diagnostic(_)));
        assert!(receiver.try_recv().is_err());
        assert!(handle.inner.drain_summaries().is_empty());
    }

    #[test]
    fn report_drop_on_node_context_feeds_warn_drop_bucket() {
        let (ctx, handle) = ctx_with_handle();
        let node_ctx: NodeContext = ctx.into();
        node_ctx.report_drop(ErrorCode::CitygmlEmptyGeometry, None, None);
        let summaries = handle.inner.drain_summaries();
        assert_eq!(summaries.len(), 1);
        assert_eq!(
            summaries[0].effective_disposition,
            Some(Disposition::WarnDrop)
        );
    }

    #[test]
    fn report_drop_resolves_a_promoting_override_to_reject_via_node_context() {
        let policy = DispositionPolicy::compile(PolicyInput {
            overrides: vec![override_node_code(
                "citygml.empty_geometry",
                Disposition::Reject,
            )],
            ..Default::default()
        })
        .expect("policy should compile");
        let (ctx, handle) = ctx_with_policy(Arc::new(policy));
        let node_ctx: NodeContext = ctx.into();
        node_ctx.report_drop(ErrorCode::CitygmlEmptyGeometry, None, None);
        let summaries = handle.inner.drain_summaries();
        assert_eq!(summaries.len(), 1);
        assert_eq!(
            summaries[0].effective_disposition,
            Some(Disposition::Reject)
        );
        assert!(handle.inner.take_fatal().is_none());
    }

    /// Final-review fix round, Item 1: the same promoting override, but
    /// through a sink handle under `side_file()` — proves
    /// `NodeContext::report_drop` threads `has_geometry` all the way to a
    /// captured side-file row matching the bucket count, not just the
    /// aggregation bucket. This is the exact call shape the production
    /// `citygml.rs`/`image_rasterizer.rs` call sites use (they call
    /// `NodeDiagnosticsHandle::report_drop` directly, but
    /// `NodeContext::report_drop` above is a thin, behavior-preserving
    /// pass-through to it).
    #[test]
    fn report_drop_resolves_a_promoting_override_to_reject_and_captures_a_side_file_row_via_node_context(
    ) {
        let policy = DispositionPolicy::compile(PolicyInput {
            side_file: true,
            overrides: vec![override_node_code(
                "citygml.empty_geometry",
                Disposition::Reject,
            )],
            ..Default::default()
        })
        .expect("policy should compile");
        let (ctx, handle) = ctx_with_policy_and_sink(Arc::new(policy), true);
        let feature_id = ctx.feature.id;
        let node_ctx: NodeContext = ctx.into();
        node_ctx.report_drop(
            ErrorCode::CitygmlEmptyGeometry,
            Some(feature_id),
            Some(true),
        );

        let summaries = handle.inner.drain_summaries();
        assert_eq!(summaries.len(), 1);
        let bucket_count = summaries[0].aggregated.as_ref().unwrap().count;
        let (rows, overflow) = handle
            .drain_reject_rows()
            .expect("a reject row should have been captured");
        assert_eq!(overflow, 0);
        assert_eq!(
            rows.len() as u64,
            bucket_count,
            "side-file row count must match the aggregation bucket count"
        );
        assert_eq!(rows[0].feature_id, Some(feature_id));
        assert_eq!(rows[0].has_geometry, Some(true));
    }

    #[test]
    fn report_drop_resolves_a_promoting_override_to_fatal_via_node_context() {
        let policy = DispositionPolicy::compile(PolicyInput {
            overrides: vec![override_node_code(
                "citygml.empty_geometry",
                Disposition::Fatal,
            )],
            ..Default::default()
        })
        .expect("policy should compile");
        let (ctx, handle) = ctx_with_policy(Arc::new(policy));
        let node_ctx: NodeContext = ctx.into();
        node_ctx.report_drop(ErrorCode::CitygmlEmptyGeometry, None, None);
        // the drain-end backstop's slot is populated, not the aggregation bucket
        assert!(handle.inner.drain_summaries().is_empty());
        let fatal = handle.inner.take_fatal().expect("fatal slot must be set");
        assert_eq!(fatal.effective_disposition, Some(Disposition::Fatal));
        assert_eq!(fatal.code, ErrorCode::CitygmlEmptyGeometry);
    }

    #[test]
    fn report_on_no_handle_context_emits_diagnostic_only() {
        let node_ctx = NodeContext::default();
        let ctx = ExecutorContext::new_with_node_context_feature_and_port(
            &node_ctx,
            Feature::from(IndexMap::<String, AttributeValue>::new()),
            FEATURES_PORT.clone(),
        );
        // ctx.diagnostics is None (never wired to a handle)
        let mut receiver = ctx.event_hub.sender.subscribe();
        let disp = ctx
            .report(DiagnosticDraft::new(ErrorCode::Cesium3dtilesEmptyGeometry))
            .unwrap();
        assert_eq!(disp, Disposition::WarnDrop);
        // never-silent: exactly one Event::Diagnostic, no twin Event::Log
        let first = receiver.try_recv().expect("one immediate diagnostic event");
        assert!(matches!(first, crate::event::Event::Diagnostic(_)));
        assert!(receiver.try_recv().is_err());
    }

    #[test]
    fn report_drop_on_no_handle_node_context_emits_diagnostic_only() {
        let node_ctx = NodeContext::default();
        let mut receiver = node_ctx.event_hub.sender.subscribe();
        node_ctx.report_drop(ErrorCode::CitygmlEmptyGeometry, None, None);
        let first = receiver.try_recv().expect("one immediate diagnostic event");
        assert!(matches!(first, crate::event::Event::Diagnostic(_)));
        assert!(receiver.try_recv().is_err());
    }
}
