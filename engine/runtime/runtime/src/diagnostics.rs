use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use reearth_flow_diagnostics::{
    Diagnostic, DiagnosticDraft, DiagnosticKind, Disposition, DispositionPolicy, ErrorCode,
    NodeDiagnostics,
};
use uuid::Uuid;

use crate::event::EventHub;
use crate::node::NodeHandle;

/// D7 (spec §7 / Task 5): hard cap on buffered reject rows per node before
/// the side-file flush degrades to counting-only. Rows beyond the cap are
/// never buffered — only counted — so a pathological run can't grow this
/// unboundedly in memory; the residual count surfaces as one overflow
/// marker row at flush time (see `render_reject_jsonl`).
pub const REJECT_ROW_CAP: usize = 10_000;

/// One row of the D7 sink reject side-file (`rejected/{composed_id}.jsonl`,
/// flushed by `SinkNode::on_terminate`). PII-minimal by design: only the
/// feature id, whether it had geometry, and the diagnostic code — no
/// feature attributes (opt-in attribute capture is deferred, see Task 5's
/// brief).
#[derive(Debug, Clone)]
pub struct RejectRow {
    pub feature_id: Option<Uuid>,
    pub has_geometry: bool,
    pub code: ErrorCode,
}

/// Sink-only buffer for D7 reject rows, owned by `NodeDiagnosticsHandle`
/// (`Some` only when constructed for a sink node under a `side_file()`
/// policy — see `NodeDiagnosticsHandle::new`). Hard-capped at
/// `REJECT_ROW_CAP` rows.
#[derive(Debug, Default)]
struct RejectCapture {
    rows: Mutex<Vec<RejectRow>>,
    overflow: AtomicU64,
}

impl RejectCapture {
    fn record(&self, feature_id: Option<Uuid>, has_geometry: bool, code: ErrorCode) {
        let mut rows = self.rows.lock().unwrap();
        if rows.len() < REJECT_ROW_CAP {
            rows.push(RejectRow {
                feature_id,
                has_geometry,
                code,
            });
        } else {
            self.overflow.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Drains buffered rows plus the residual overflow count. `None` when
    /// there is nothing to flush (no `Reject` was ever captured).
    fn drain(&self) -> Option<(Vec<RejectRow>, u64)> {
        let rows = std::mem::take(&mut *self.rows.lock().unwrap());
        let overflow = self.overflow.swap(0, Ordering::Relaxed);
        if rows.is_empty() && overflow == 0 {
            None
        } else {
            Some((rows, overflow))
        }
    }
}

/// Render buffered D7 reject rows as newline-delimited JSON — one object
/// per row (`{"featureId":...,"hasGeometry":...,"code":"..."}`), plus one
/// trailing overflow-marker row when `overflow > 0` (spec: "an overflow
/// marker row at flush notes the residual count"). Pure/no I/O so the cap
/// and shape are directly unit-testable.
pub fn render_reject_jsonl(rows: &[RejectRow], overflow: u64) -> Vec<u8> {
    let mut buf = Vec::new();
    for row in rows {
        let obj = serde_json::json!({
            "featureId": row.feature_id,
            "hasGeometry": row.has_geometry,
            "code": row.code,
        });
        buf.extend_from_slice(obj.to_string().as_bytes());
        buf.push(b'\n');
    }
    if overflow > 0 {
        let marker = serde_json::json!({
            "overflow": true,
            "count": overflow,
        });
        buf.extend_from_slice(marker.to_string().as_bytes());
        buf.push(b'\n');
    }
    buf
}

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
    /// D7 (Task 5): `Some` only when this handle was constructed for a sink
    /// node (`is_sink: true`) under a `side_file()` policy — see `new`.
    /// Everywhere else `record_reject_row`/`drain_reject_rows` are no-ops.
    reject_capture: Option<RejectCapture>,
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
        is_sink: bool,
    ) -> Self {
        let reject_capture =
            (is_sink && disposition_policy.side_file()).then(RejectCapture::default);
        Self {
            node_handle,
            node_name,
            inner: Arc::new(NodeDiagnostics::new(composed_id, action_type, warn_once)),
            disposition_policy,
            reject_capture,
        }
    }

    /// D7: buffer one rejected-feature row for the sink side-file flush
    /// (`SinkNode::on_terminate`). No-op unless this handle was constructed
    /// for a sink node under a `side_file()` policy (see `new`) — the
    /// buffer simply doesn't exist otherwise, so this never allocates for a
    /// no-policy or non-sink node.
    pub fn record_reject_row(&self, feature_id: Option<Uuid>, has_geometry: bool, code: ErrorCode) {
        if let Some(capture) = &self.reject_capture {
            capture.record(feature_id, has_geometry, code);
        }
    }

    /// Drain buffered D7 reject rows for flush (see `record_reject_row`).
    /// `None` when reject capture isn't enabled for this node, or nothing
    /// was ever captured.
    pub fn drain_reject_rows(&self) -> Option<(Vec<RejectRow>, u64)> {
        self.reject_capture.as_ref().and_then(RejectCapture::drain)
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
                // D7 (Task 5): deliberately NOT calling `record_reject_row`
                // here. This branch has no `Feature` to derive `has_geometry`
                // from — only a bare `feature_id` — and hardcoding `false`
                // would write wrong data to the side file. Today no caller
                // resolves to `Reject` via this finish()-time path (the two
                // real `report_drop` call sites, in
                // `action-sink/src/file/citygml.rs`, both currently land on
                // `WarnDrop`-default codes with no test overriding them to
                // `Reject`), so the seam is left minimal rather than
                // threading a `has_geometry` bool through a signature no one
                // needs yet. If a finish()-time `Reject` caller appears,
                // extend this branch (and `report_drop`'s signature) then.
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
        handle_with_policy_and_sink(disposition_policy, false)
    }

    fn handle_with_policy_and_sink(
        disposition_policy: Arc<DispositionPolicy>,
        is_sink: bool,
    ) -> NodeDiagnosticsHandle {
        NodeDiagnosticsHandle::new(
            COMPOSED_ID.to_string(),
            NodeHandle::new(NodeId::new("node-1".to_string())),
            "writer-1".to_string(),
            "Cesium 3D Tiles Writer".to_string(),
            Arc::default(),
            disposition_policy,
            is_sink,
        )
    }

    fn side_file_policy() -> Arc<DispositionPolicy> {
        Arc::new(
            DispositionPolicy::compile(PolicyInput {
                side_file: true,
                ..Default::default()
            })
            .expect("policy should compile"),
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

    // -----------------------------------------------------------------
    // D7 (Task 5): reject-row capture (`record_reject_row`/
    // `drain_reject_rows`) and the JSONL flush rendering.
    // -----------------------------------------------------------------

    #[test]
    fn record_reject_row_is_a_no_op_without_side_file_policy() {
        // is_sink: true, but the default policy has side_file() == false.
        let handle = handle_with_policy_and_sink(Arc::default(), true);
        handle.record_reject_row(
            Some(uuid::Uuid::nil()),
            true,
            ErrorCode::CitygmlEmptyGeometry,
        );
        assert!(handle.drain_reject_rows().is_none());
    }

    #[test]
    fn record_reject_row_is_a_no_op_for_a_non_sink_handle_even_with_side_file_policy() {
        let handle = handle_with_policy_and_sink(side_file_policy(), false);
        handle.record_reject_row(
            Some(uuid::Uuid::nil()),
            true,
            ErrorCode::CitygmlEmptyGeometry,
        );
        assert!(handle.drain_reject_rows().is_none());
    }

    #[test]
    fn record_reject_row_buffers_when_sink_and_side_file_both_apply() {
        let handle = handle_with_policy_and_sink(side_file_policy(), true);
        let id = uuid::Uuid::new_v4();
        handle.record_reject_row(Some(id), true, ErrorCode::CitygmlEmptyGeometry);
        handle.record_reject_row(None, false, ErrorCode::GltfZeroFaceSolid);
        let (rows, overflow) = handle.drain_reject_rows().expect("rows should be buffered");
        assert_eq!(overflow, 0);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].feature_id, Some(id));
        assert!(rows[0].has_geometry);
        assert_eq!(rows[0].code, ErrorCode::CitygmlEmptyGeometry);
        assert_eq!(rows[1].feature_id, None);
        assert!(!rows[1].has_geometry);
        assert_eq!(rows[1].code, ErrorCode::GltfZeroFaceSolid);
    }

    #[test]
    fn drain_reject_rows_empties_the_buffer_and_a_second_drain_is_none() {
        let handle = handle_with_policy_and_sink(side_file_policy(), true);
        handle.record_reject_row(None, false, ErrorCode::CitygmlEmptyGeometry);
        assert!(handle.drain_reject_rows().is_some());
        assert!(handle.drain_reject_rows().is_none());
    }

    /// Cap behavior (Task 5 brief): 10_001 records -> 10_000 buffered rows
    /// plus an overflow count of 1, never growing the buffer past the cap.
    #[test]
    fn record_reject_row_caps_at_reject_row_cap_and_counts_the_residual() {
        let handle = handle_with_policy_and_sink(side_file_policy(), true);
        for _ in 0..(REJECT_ROW_CAP + 1) {
            handle.record_reject_row(None, false, ErrorCode::CitygmlEmptyGeometry);
        }
        let (rows, overflow) = handle.drain_reject_rows().expect("rows should be buffered");
        assert_eq!(rows.len(), REJECT_ROW_CAP);
        assert_eq!(overflow, 1);
    }

    #[test]
    fn render_reject_jsonl_emits_one_json_object_per_row_with_no_overflow_marker() {
        let id = uuid::Uuid::new_v4();
        let rows = vec![
            RejectRow {
                feature_id: Some(id),
                has_geometry: true,
                code: ErrorCode::CitygmlEmptyGeometry,
            },
            RejectRow {
                feature_id: None,
                has_geometry: false,
                code: ErrorCode::GltfZeroFaceSolid,
            },
        ];
        let bytes = render_reject_jsonl(&rows, 0);
        let text = String::from_utf8(bytes).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 2);
        let first: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(first["featureId"], serde_json::json!(id));
        assert_eq!(first["hasGeometry"], serde_json::json!(true));
        assert_eq!(first["code"], serde_json::json!("citygml.empty_geometry"));
        let second: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
        assert_eq!(second["featureId"], serde_json::Value::Null);
        assert_eq!(second["hasGeometry"], serde_json::json!(false));
    }

    #[test]
    fn render_reject_jsonl_appends_an_overflow_marker_row_when_capped() {
        let rows = vec![RejectRow {
            feature_id: None,
            has_geometry: false,
            code: ErrorCode::CitygmlEmptyGeometry,
        }];
        let bytes = render_reject_jsonl(&rows, 42);
        let text = String::from_utf8(bytes).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 2, "expected 1 data row + 1 overflow marker");
        let marker: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
        assert_eq!(marker["overflow"], serde_json::json!(true));
        assert_eq!(marker["count"], serde_json::json!(42));
    }

    #[test]
    fn render_reject_jsonl_of_empty_rows_and_zero_overflow_is_empty() {
        let bytes = render_reject_jsonl(&[], 0);
        assert!(bytes.is_empty());
    }
}
