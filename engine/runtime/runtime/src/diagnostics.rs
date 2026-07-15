use std::sync::Arc;

use reearth_flow_diagnostics::{
    Diagnostic, DiagnosticDraft, DiagnosticKind, Disposition, DispositionPolicy, ErrorCode,
    NodeDiagnostics,
};

use crate::event::EventHub;
use crate::node::NodeHandle;

/// Runtime-side wrapper pairing the crate-agnostic aggregator with the
/// runtime identities needed to emit `Event::Diagnostic`s.
///
/// `inner`'s `node_id` is the *composed* id (`builder_dag::NodeType::
/// composed_id`, `"{subgraph_prefix}.{raw_id}"` or just the raw id with no
/// subgraph) — the identity used for diagnostic/log attribution text and
/// policy resolution (spec 4.2/4.3). `node_handle` and `node_name` below
/// deliberately stay the *raw*, un-composed identity and are kept even
/// though nothing in this module reads them back: they mirror the same two
/// fields every `ProcessorNode`/`SinkNode` already carries, where
/// `node_handle` is what `Event::Log`'s `node_handle` field is keyed off
/// (see `event.rs` — that wire shape is the description-DAG id, not the
/// subgraph-composed one) and `node_name` is what feeds
/// `node_info_tls`/`UserFacingLogHandler` attribution via those same
/// `Event::Log` fields. Keeping both here too means a caller holding only a
/// `SharedNodeDiagnostics` never has to go find the owning node struct to
/// recover the raw identity.
#[derive(Debug)]
pub struct NodeDiagnosticsHandle {
    pub node_handle: NodeHandle,
    pub node_name: String,
    pub inner: Arc<NodeDiagnostics>,
    disposition_policy: Arc<DispositionPolicy>,
}

pub type SharedNodeDiagnostics = Arc<NodeDiagnosticsHandle>;

impl NodeDiagnosticsHandle {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        composed_id: String,
        node_handle: NodeHandle,
        node_name: String,
        action_type: String,
        warn_once: reearth_flow_diagnostics::WarnOnceSet,
        disposition_policy: Arc<DispositionPolicy>,
    ) -> Self {
        Self {
            node_handle,
            node_name,
            inner: Arc::new(NodeDiagnostics::new(composed_id, action_type, warn_once)),
            disposition_policy,
        }
    }

    /// Resolves the effective disposition for `code` at this node
    /// (`inner.node_id()`, the composed id) via the compiled policy —
    /// `ExecutorContext::report()`'s resolve() ladder (spec 4.2).
    pub fn resolve(&self, code: ErrorCode) -> Disposition {
        self.disposition_policy.resolve(self.inner.node_id(), code)
    }

    /// finish()-time drop reporting for code without an ExecutorContext.
    /// Despite the name, this is no longer unconditionally a WarnDrop: it
    /// runs the same `resolve()` ladder `report()` does, since policy can
    /// promote/demote the code just as freely for a finish()-time drop as
    /// for a per-feature one. `WarnDrop`/`Reject` land in the normal
    /// aggregation bucket; a resolved `Fatal` can't be returned as an `Err`
    /// here (finish()-time drop sites are fire-and-forget, unlike
    /// `report()`), so it goes to the fatal slot instead — the same
    /// drain-end backstop `report()`'s Fatal branch relies on
    /// (`take_fatal()` in `processor_node.rs`/`sink_node.rs`'s
    /// `on_terminate` reconciliation) fails the node from there.
    pub fn report_drop(&self, code: ErrorCode, feature_id: Option<uuid::Uuid>) {
        let effective = self.resolve(code);
        match effective {
            Disposition::Fatal => {
                let mut diagnostic = Diagnostic::from_draft(
                    DiagnosticDraft::new(code),
                    Some(self.inner.node_id().to_string()),
                    Some(self.inner.action_type().to_string()),
                    feature_id,
                );
                diagnostic.effective_disposition = Some(Disposition::Fatal);
                self.inner.record_fatal(diagnostic);
            }
            Disposition::WarnDrop | Disposition::Reject => {
                let kind = if effective == Disposition::WarnDrop {
                    DiagnosticKind::WarnDrop
                } else {
                    DiagnosticKind::Reject
                };
                self.inner.record(kind, code, feature_id);
            }
        }
    }
}

/// Drain the node's buckets once and, for each (code, kind) summary, emit a
/// structured `Event::Diagnostic`. `LogEventHandler` renders it through the
/// action log (see `runner::log_event_handler`), so no twin `Event::Log` is
/// sent here. Called once per node after `finish()`. Returns the drained
/// summaries for callers that need them (e.g. RunSummary, Task 5).
pub fn emit_summaries(
    event_hub: &EventHub,
    handle: &NodeDiagnosticsHandle,
) -> Vec<reearth_flow_diagnostics::Diagnostic> {
    let summaries = handle.inner.drain_summaries();
    for summary in &summaries {
        event_hub.diagnostic(summary.clone());
    }
    summaries
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::Event;
    use crate::node::{NodeHandle, NodeId};
    use reearth_flow_diagnostics::{
        DiagnosticKind, Disposition, DispositionPolicy, ErrorCode, OverrideInput, PolicyInput,
    };

    const COMPOSED_ID: &str = "node-1";

    fn handle() -> NodeDiagnosticsHandle {
        handle_with_policy(Arc::default())
    }

    fn handle_with_policy(disposition_policy: Arc<DispositionPolicy>) -> NodeDiagnosticsHandle {
        NodeDiagnosticsHandle::new(
            COMPOSED_ID.to_string(),
            NodeHandle::new(NodeId::new("node-1".to_string())),
            "writer-1".to_string(),
            "Cesium 3D Tiles Writer".to_string(),
            Arc::default(),
            disposition_policy,
        )
    }

    fn override_all(
        node: Option<&str>,
        code: Option<&str>,
        disposition: Disposition,
    ) -> OverrideInput {
        OverrideInput {
            node: node.map(String::from),
            code: code.map(String::from),
            category: None,
            disposition,
        }
    }

    #[test]
    fn emit_summaries_sends_exactly_one_diagnostic_and_zero_log_per_summary_and_drains_once() {
        let handle = handle();
        handle.inner.record(
            DiagnosticKind::WarnDrop,
            ErrorCode::Cesium3dtilesEmptyGeometry,
            None,
        );
        handle.inner.record(
            DiagnosticKind::WarnDrop,
            ErrorCode::Cesium3dtilesNonCitygmlGeometry,
            None,
        );
        let event_hub = EventHub::new(30);
        let mut receiver = event_hub.sender.subscribe();

        let summaries = emit_summaries(&event_hub, &handle);
        assert_eq!(summaries.len(), 2);

        let mut log_count = 0;
        let mut diagnostic_count = 0;
        for _ in 0..2 {
            match receiver.try_recv().expect("expected event") {
                Event::Log { .. } => log_count += 1,
                Event::Diagnostic(d) => {
                    diagnostic_count += 1;
                    assert_eq!(d.aggregated.as_ref().unwrap().count, 1);
                }
                other => panic!("unexpected event: {other:?}"),
            }
        }
        assert_eq!(log_count, 0);
        assert_eq!(diagnostic_count, 2);
        assert!(receiver.try_recv().is_err());

        // second call: buckets already drained, no events, empty Vec
        let mut receiver2 = event_hub.sender.subscribe();
        let summaries2 = emit_summaries(&event_hub, &handle);
        assert!(summaries2.is_empty());
        assert!(receiver2.try_recv().is_err());
    }

    #[test]
    fn resolve_falls_back_to_registry_default_under_the_default_policy() {
        let handle = handle();
        assert_eq!(
            handle.resolve(ErrorCode::Cesium3dtilesEmptyGeometry),
            Disposition::WarnDrop
        );
    }

    #[test]
    fn resolve_honors_a_node_plus_code_override() {
        let policy = DispositionPolicy::compile(PolicyInput {
            overrides: vec![override_all(
                Some(COMPOSED_ID),
                Some("cesium3dtiles.empty_geometry"),
                Disposition::Fatal,
            )],
            ..Default::default()
        })
        .expect("policy should compile");
        let handle = handle_with_policy(Arc::new(policy));
        assert_eq!(
            handle.resolve(ErrorCode::Cesium3dtilesEmptyGeometry),
            Disposition::Fatal
        );
    }

    #[test]
    fn report_drop_under_default_policy_still_lands_in_the_warn_drop_bucket() {
        let handle = handle();
        handle.report_drop(ErrorCode::CitygmlEmptyGeometry, None);
        let summaries = handle.inner.drain_summaries();
        assert_eq!(summaries.len(), 1);
        assert_eq!(
            summaries[0].effective_disposition,
            Some(Disposition::WarnDrop)
        );
        assert!(handle.inner.take_fatal().is_none());
    }

    #[test]
    fn report_drop_resolves_a_promoting_override_into_the_reject_bucket() {
        let policy = DispositionPolicy::compile(PolicyInput {
            overrides: vec![override_all(
                Some(COMPOSED_ID),
                Some("citygml.empty_geometry"),
                Disposition::Reject,
            )],
            ..Default::default()
        })
        .expect("policy should compile");
        let handle = handle_with_policy(Arc::new(policy));
        handle.report_drop(ErrorCode::CitygmlEmptyGeometry, None);
        let summaries = handle.inner.drain_summaries();
        assert_eq!(summaries.len(), 1);
        assert_eq!(
            summaries[0].effective_disposition,
            Some(Disposition::Reject)
        );
        assert!(handle.inner.take_fatal().is_none());
    }

    #[test]
    fn report_drop_resolves_a_fatal_override_into_the_fatal_slot_not_the_bucket() {
        let policy = DispositionPolicy::compile(PolicyInput {
            overrides: vec![override_all(
                Some(COMPOSED_ID),
                Some("citygml.empty_geometry"),
                Disposition::Fatal,
            )],
            ..Default::default()
        })
        .expect("policy should compile");
        let handle = handle_with_policy(Arc::new(policy));
        handle.report_drop(ErrorCode::CitygmlEmptyGeometry, Some(uuid::Uuid::nil()));

        // the drain-end backstop's slot is populated, not the aggregation bucket
        assert!(handle.inner.drain_summaries().is_empty());
        let fatal = handle.inner.take_fatal().expect("fatal slot should be set");
        assert_eq!(fatal.effective_disposition, Some(Disposition::Fatal));
        assert_eq!(fatal.code, ErrorCode::CitygmlEmptyGeometry);
        assert_eq!(fatal.node_id.as_deref(), Some(COMPOSED_ID));
        assert_eq!(fatal.feature_id, Some(uuid::Uuid::nil()));
    }
}
