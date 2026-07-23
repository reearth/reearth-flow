use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use reearth_flow_diagnostics::{
    Diagnostic, DiagnosticDraft, DiagnosticKind, Disposition, DispositionPolicy, ErrorCode,
    NodeDiagnostics,
};
use uuid::Uuid;

use crate::event::EventHub;
use crate::node::NodeHandle;

/// Rows beyond the cap are counted, not buffered, and surface as one overflow marker row at flush.
pub const REJECT_ROW_CAP: usize = 10_000;

/// PII-minimal (no feature attributes). `has_geometry` is tri-state: `None` means unknown, not `false`.
#[derive(Debug, Clone)]
pub struct RejectRow {
    pub feature_id: Option<Uuid>,
    pub has_geometry: Option<bool>,
    pub code: ErrorCode,
}

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

/// `node_handle`/`node_name` are unused here but must stay — `Event::Log` keys off them for log attribution.
#[derive(Debug)]
pub struct NodeDiagnosticsHandle {
    pub node_handle: NodeHandle,
    pub node_name: String,
    pub inner: Arc<NodeDiagnostics>,
    disposition_policy: Arc<DispositionPolicy>,
    /// `Some` only for a sink node under a `side_file()` policy; otherwise the record/drain methods are no-ops.
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

    /// `None` when capture isn't enabled, or nothing was ever captured.
    pub fn drain_reject_rows(&self) -> Option<(Vec<RejectRow>, u64)> {
        self.reject_capture.as_ref().and_then(RejectCapture::drain)
    }

    pub fn resolve(&self, code: ErrorCode) -> Disposition {
        self.disposition_policy.resolve(self.inner.node_id(), code)
    }

    /// Unlike `report()`, a resolved `Fatal` lands in the fatal slot, not `Err` — finish()-time sites are fire-and-forget.
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
                if effective == Disposition::Reject {
                    self.record_reject_row(feature_id, has_geometry, code);
                }
            }
        }
    }
}

/// No twin `Event::Log` is emitted — `LogEventHandler` renders diagnostics via the action log instead.
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

        assert!(handle.inner.drain_summaries().is_empty());
        let fatal = handle.inner.take_fatal().expect("fatal slot should be set");
        assert_eq!(fatal.effective_disposition, Some(Disposition::Fatal));
        assert_eq!(fatal.code, ErrorCode::CitygmlEmptyGeometry);
        assert_eq!(fatal.node_id.as_deref(), Some(COMPOSED_ID));
        assert_eq!(fatal.feature_id, Some(uuid::Uuid::nil()));
    }

    #[test]
    fn record_reject_row_is_a_no_op_without_side_file_policy() {
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
        let handle = handle_with_policy_and_sink(Arc::new(policy), true);
        handle.report_drop(ErrorCode::CitygmlEmptyGeometry, None, Some(true));
        assert_eq!(handle.inner.drain_summaries().len(), 1);
        assert!(handle.drain_reject_rows().is_none());
    }

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
