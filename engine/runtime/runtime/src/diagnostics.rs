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
///
/// `has_geometry` is tri-state (`Option<bool>`, serializing to a nullable
/// `hasGeometry` in the JSONL row): `report()`'s per-feature path always
/// knows it (`Some`, computed from the live `Feature`), but a finish()-time
/// `report_drop` caller may have no `Feature` to derive it from honestly —
/// `None` means "unknown", not "false" (2a-policy final-review fix round,
/// Item 1).
#[derive(Debug, Clone)]
pub struct RejectRow {
    pub feature_id: Option<Uuid>,
    pub has_geometry: Option<bool>,
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
    fn record(&self, feature_id: Option<Uuid>, has_geometry: Option<bool>, code: ErrorCode) {
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
    /// no-policy or non-sink node. `has_geometry` is tri-state — see
    /// `RejectRow`'s doc comment.
    pub fn record_reject_row(
        &self,
        feature_id: Option<Uuid>,
        has_geometry: Option<bool>,
        code: ErrorCode,
    ) {
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
    ///
    /// `has_geometry` (2a-policy final-review fix round, Item 1): a resolved
    /// `Reject` now also reaches the side-file capture
    /// (`record_reject_row`), same as `report()`'s per-feature path — an
    /// override promoting a finish()-time code to `reject` used to count the
    /// rejection in the bucket but write no side-file row at all, not even
    /// an empty one. Unlike `report()`, callers here often have no live
    /// `Feature` to derive `has_geometry` from, so it's threaded through as
    /// a caller-supplied tri-state (`None` = unknown, not `false`) rather
    /// than computed here.
    pub fn report_drop(
        &self,
        code: ErrorCode,
        feature_id: Option<uuid::Uuid>,
        has_geometry: Option<bool>,
    ) {
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
                // D7 (final-review fix round, Item 1): capture a side-file
                // row alongside the aggregation bucket above, same shape as
                // `ExecutorContext::report()`'s `Reject` branch.
                // `record_reject_row` is a no-op unless this handle belongs
                // to a sink node under a `side_file()` policy, so this stays
                // free on every other path.
                if effective == Disposition::Reject {
                    self.record_reject_row(feature_id, has_geometry, code);
                }
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
        handle.report_drop(ErrorCode::CitygmlEmptyGeometry, None, None);
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
        handle.report_drop(ErrorCode::CitygmlEmptyGeometry, None, None);
        let summaries = handle.inner.drain_summaries();
        assert_eq!(summaries.len(), 1);
        assert_eq!(
            summaries[0].effective_disposition,
            Some(Disposition::Reject)
        );
        assert!(handle.inner.take_fatal().is_none());
        // no side-file capture: `handle_with_policy` builds a non-sink handle.
        assert!(handle.drain_reject_rows().is_none());
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
        handle.report_drop(
            ErrorCode::CitygmlEmptyGeometry,
            Some(uuid::Uuid::nil()),
            None,
        );

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
            Some(true),
            ErrorCode::CitygmlEmptyGeometry,
        );
        assert!(handle.drain_reject_rows().is_none());
    }

    #[test]
    fn record_reject_row_is_a_no_op_for_a_non_sink_handle_even_with_side_file_policy() {
        let handle = handle_with_policy_and_sink(side_file_policy(), false);
        handle.record_reject_row(
            Some(uuid::Uuid::nil()),
            Some(true),
            ErrorCode::CitygmlEmptyGeometry,
        );
        assert!(handle.drain_reject_rows().is_none());
    }

    #[test]
    fn record_reject_row_buffers_when_sink_and_side_file_both_apply() {
        let handle = handle_with_policy_and_sink(side_file_policy(), true);
        let id = uuid::Uuid::new_v4();
        handle.record_reject_row(Some(id), Some(true), ErrorCode::CitygmlEmptyGeometry);
        handle.record_reject_row(None, Some(false), ErrorCode::GltfZeroFaceSolid);
        let (rows, overflow) = handle.drain_reject_rows().expect("rows should be buffered");
        assert_eq!(overflow, 0);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].feature_id, Some(id));
        assert_eq!(rows[0].has_geometry, Some(true));
        assert_eq!(rows[0].code, ErrorCode::CitygmlEmptyGeometry);
        assert_eq!(rows[1].feature_id, None);
        assert_eq!(rows[1].has_geometry, Some(false));
        assert_eq!(rows[1].code, ErrorCode::GltfZeroFaceSolid);
    }

    /// `has_geometry: None` (unknown) buffers just like `Some` — the tri-state
    /// is opaque to the capture/drain path, only `render_reject_jsonl`
    /// interprets it (as a nullable `hasGeometry`).
    #[test]
    fn record_reject_row_buffers_a_none_has_geometry_row_as_unknown() {
        let handle = handle_with_policy_and_sink(side_file_policy(), true);
        handle.record_reject_row(None, None, ErrorCode::CitygmlEmptyGeometry);
        let (rows, overflow) = handle.drain_reject_rows().expect("rows should be buffered");
        assert_eq!(overflow, 0);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].has_geometry, None);
    }

    #[test]
    fn drain_reject_rows_empties_the_buffer_and_a_second_drain_is_none() {
        let handle = handle_with_policy_and_sink(side_file_policy(), true);
        handle.record_reject_row(None, Some(false), ErrorCode::CitygmlEmptyGeometry);
        assert!(handle.drain_reject_rows().is_some());
        assert!(handle.drain_reject_rows().is_none());
    }

    /// Cap behavior (Task 5 brief): 10_001 records -> 10_000 buffered rows
    /// plus an overflow count of 1, never growing the buffer past the cap.
    #[test]
    fn record_reject_row_caps_at_reject_row_cap_and_counts_the_residual() {
        let handle = handle_with_policy_and_sink(side_file_policy(), true);
        for _ in 0..(REJECT_ROW_CAP + 1) {
            handle.record_reject_row(None, Some(false), ErrorCode::CitygmlEmptyGeometry);
        }
        let (rows, overflow) = handle.drain_reject_rows().expect("rows should be buffered");
        assert_eq!(rows.len(), REJECT_ROW_CAP);
        assert_eq!(overflow, 1);
    }

    // -----------------------------------------------------------------
    // Final-review fix round, Item 1: `report_drop`'s Reject branch now
    // reaches the side-file capture too, not just the aggregation bucket.
    // -----------------------------------------------------------------

    /// The core regression proof: a Reject-resolving `report_drop` under a
    /// sink + `side_file()` handle must produce exactly as many side-file
    /// rows as the aggregation bucket counted (previously it produced zero
    /// rows for every such drop — no shard was ever written, not even an
    /// empty one).
    #[test]
    fn report_drop_resolving_reject_captures_a_row_matching_the_bucket_count() {
        let policy = DispositionPolicy::compile(PolicyInput {
            side_file: true,
            overrides: vec![override_all(
                Some(COMPOSED_ID),
                Some("citygml.empty_geometry"),
                Disposition::Reject,
            )],
            ..Default::default()
        })
        .expect("policy should compile");
        let handle = handle_with_policy_and_sink(Arc::new(policy), true);
        let id = uuid::Uuid::new_v4();
        handle.report_drop(ErrorCode::CitygmlEmptyGeometry, Some(id), Some(true));

        let summaries = handle.inner.drain_summaries();
        assert_eq!(summaries.len(), 1);
        assert_eq!(
            summaries[0].effective_disposition,
            Some(Disposition::Reject)
        );
        let bucket_count = summaries[0].aggregated.as_ref().unwrap().count;
        assert_eq!(bucket_count, 1);

        let (rows, overflow) = handle
            .drain_reject_rows()
            .expect("a reject row should have been captured");
        assert_eq!(overflow, 0);
        assert_eq!(
            rows.len() as u64,
            bucket_count,
            "side-file row count must match the aggregation bucket count"
        );
        assert_eq!(rows[0].feature_id, Some(id));
        assert_eq!(rows[0].has_geometry, Some(true));
        assert_eq!(rows[0].code, ErrorCode::CitygmlEmptyGeometry);
    }

    /// Same override, but the caller has no `Feature` to derive
    /// `has_geometry` from (the real finish()-time situation for some
    /// call sites) — `None` is captured verbatim, not guessed as `false`.
    #[test]
    fn report_drop_resolving_reject_with_unknown_geometry_captures_a_null_row() {
        let policy = DispositionPolicy::compile(PolicyInput {
            side_file: true,
            overrides: vec![override_all(
                Some(COMPOSED_ID),
                Some("citygml.empty_geometry"),
                Disposition::Reject,
            )],
            ..Default::default()
        })
        .expect("policy should compile");
        let handle = handle_with_policy_and_sink(Arc::new(policy), true);
        handle.report_drop(ErrorCode::CitygmlEmptyGeometry, None, None);

        let (rows, overflow) = handle
            .drain_reject_rows()
            .expect("a reject row should have been captured");
        assert_eq!(overflow, 0);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].feature_id, None);
        assert_eq!(rows[0].has_geometry, None);
    }

    /// Without `side_file()`, a Reject-resolving `report_drop` still
    /// increments the aggregation bucket but captures no row — unchanged,
    /// pre-fix-round behavior for the no-side-file-policy case.
    #[test]
    fn report_drop_resolving_reject_without_side_file_captures_no_row() {
        let policy = DispositionPolicy::compile(PolicyInput {
            overrides: vec![override_all(
                Some(COMPOSED_ID),
                Some("citygml.empty_geometry"),
                Disposition::Reject,
            )],
            ..Default::default()
        })
        .expect("policy should compile");
        // is_sink: true, but the policy itself has no side_file() -- capture
        // stays disabled per `NodeDiagnosticsHandle::new`.
        let handle = handle_with_policy_and_sink(Arc::new(policy), true);
        handle.report_drop(ErrorCode::CitygmlEmptyGeometry, None, Some(true));
        assert_eq!(handle.inner.drain_summaries().len(), 1);
        assert!(handle.drain_reject_rows().is_none());
    }

    /// A `WarnDrop`-resolving `report_drop` never captures a side-file row,
    /// even under a sink + `side_file()` handle — only `Reject` does.
    #[test]
    fn report_drop_resolving_warn_drop_never_captures_a_row_even_with_side_file_policy() {
        let handle = handle_with_policy_and_sink(side_file_policy(), true);
        handle.report_drop(ErrorCode::CitygmlEmptyGeometry, None, Some(true));
        assert_eq!(
            handle.inner.drain_summaries()[0].effective_disposition,
            Some(Disposition::WarnDrop)
        );
        assert!(handle.drain_reject_rows().is_none());
    }

    #[test]
    fn render_reject_jsonl_emits_one_json_object_per_row_with_no_overflow_marker() {
        let id = uuid::Uuid::new_v4();
        let rows = vec![
            RejectRow {
                feature_id: Some(id),
                has_geometry: Some(true),
                code: ErrorCode::CitygmlEmptyGeometry,
            },
            RejectRow {
                feature_id: None,
                has_geometry: Some(false),
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

    /// `has_geometry: None` (unknown, e.g. a finish()-time `report_drop`
    /// caller with no live `Feature`) renders as a JSON `null`, not `false` —
    /// the row serialization is honest about "unknown" vs. "known false".
    #[test]
    fn render_reject_jsonl_emits_null_has_geometry_for_unknown_rows() {
        let rows = vec![RejectRow {
            feature_id: None,
            has_geometry: None,
            code: ErrorCode::CitygmlEmptyGeometry,
        }];
        let bytes = render_reject_jsonl(&rows, 0);
        let text = String::from_utf8(bytes).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 1);
        let row: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(row["hasGeometry"], serde_json::Value::Null);
    }

    #[test]
    fn render_reject_jsonl_appends_an_overflow_marker_row_when_capped() {
        let rows = vec![RejectRow {
            feature_id: None,
            has_geometry: Some(false),
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
