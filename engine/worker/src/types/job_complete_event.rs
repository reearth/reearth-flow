use std::env;

use bytes::Bytes;
use chrono::Utc;
use once_cell::sync::Lazy;
use reearth_flow_diagnostics::RunSummary;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    pubsub::{
        message::{EncodableMessage, ValidatedMessage},
        topic::Topic,
    },
    types::diagnostic_event::WireDiagnostic,
};

static JOB_COMPLETE_TOPIC: Lazy<String> = Lazy::new(|| {
    env::var("FLOW_WORKER_JOB_COMPLETE_TOPIC")
        .ok()
        .unwrap_or("flow-job-complete-topic".to_string())
});

/// Top-K bound (spec 4.7) applied to `RunSummary::capped` before it is
/// carried on `JobCompleteEvent`, so the wire payload cannot grow unbounded
/// with pathological runs that produce many distinct diagnostics.
pub const JOB_COMPLETE_TOP_K: usize = 50;

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum JobResult {
    Success,
    Failed,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct JobCompleteEvent {
    pub workflow_id: Uuid,
    pub job_id: Uuid,
    pub result: JobResult,
    pub timestamp: chrono::DateTime<Utc>,
    /// Absent (not `null`) when there is no `RunSummary` to report (the
    /// runner returned `Err`), so old subscribers that don't know about
    /// these fields see exactly the pre-Task-10 wire shape.
    ///
    /// Consumers MUST NOT infer job failure from this field: `result:
    /// "failed"` with an empty or absent `failedNodes` is common (legacy
    /// per-feature errors don't produce structured failed nodes yet; the
    /// regex log fallback remains the failure-detail source until they are
    /// converted). Absent = run aborted before a summary existed; empty =
    /// run completed with no structured node failures recorded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failed_nodes: Option<Vec<WireDiagnostic>>,
    /// finish()-time diagnostics aggregated across all nodes. Same
    /// absent-vs-empty contract as `failed_nodes` (absent = run aborted
    /// before a summary existed, empty = run completed with nothing to
    /// report) — do not infer job failure from this field either.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aggregated_diagnostics: Option<Vec<WireDiagnostic>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dropped_event_count: Option<u64>,
}

impl JobCompleteEvent {
    pub fn new(workflow_id: Uuid, job_id: Uuid, result: JobResult) -> Self {
        Self {
            workflow_id,
            job_id,
            result,
            timestamp: chrono::Utc::now(),
            failed_nodes: None,
            aggregated_diagnostics: None,
            dropped_event_count: None,
        }
    }

    /// Builds the event with structured diagnostics populated from `summary`,
    /// applying the `JOB_COMPLETE_TOP_K` wire bound (spec 4.7) internally via
    /// `summary.capped(JOB_COMPLETE_TOP_K)` before rendering. Use `new`
    /// instead when there is no summary to report.
    ///
    /// 2a-core review rec 4 hardening: capping used to be the caller's
    /// responsibility (`with_summary(..., &summary.capped(JOB_COMPLETE_TOP_K))`
    /// at the single production call site in `command.rs`) — an invariant
    /// that held only because nothing else called this function. Capping
    /// internally instead removes that single-call-site drift risk: every
    /// caller, present or future, gets the wire bound for free and cannot
    /// forget it. Callers must now pass the *uncapped* `summary` (see
    /// `command.rs`) — capping twice would be wrong, not just redundant: a
    /// second `cap_diagnostics` pass over an already-capped (`k`-real +
    /// 1-overflow-marker) list would fold the prior overflow marker's
    /// `count` into a new one, understating the true residual.
    pub fn with_summary(
        workflow_id: Uuid,
        job_id: Uuid,
        result: JobResult,
        summary: &RunSummary,
    ) -> Self {
        let capped = summary.capped(JOB_COMPLETE_TOP_K);
        Self {
            failed_nodes: Some(
                capped
                    .failed_nodes
                    .iter()
                    .map(WireDiagnostic::from)
                    .collect(),
            ),
            aggregated_diagnostics: Some(
                capped
                    .aggregated_diagnostics
                    .iter()
                    .map(WireDiagnostic::from)
                    .collect(),
            ),
            dropped_event_count: Some(capped.dropped_event_count),
            ..Self::new(workflow_id, job_id, result)
        }
    }
}

impl EncodableMessage for JobCompleteEvent {
    type Error = crate::errors::Error;

    fn topic(&self) -> Topic {
        Topic::new(JOB_COMPLETE_TOPIC.clone())
    }

    /// Encode the message payload.
    fn encode(&self) -> crate::errors::Result<ValidatedMessage<Bytes>> {
        serde_json::to_string(self)
            .map_err(crate::errors::Error::FailedToEncode)
            .map(|payload| {
                ValidatedMessage::new(uuid::Uuid::new_v4(), self.timestamp, Bytes::from(payload))
            })
    }
}

#[cfg(test)]
mod tests {
    use reearth_flow_diagnostics::{AggregateInfo, Diagnostic, DiagnosticDraft, ErrorCode};

    use super::*;

    fn fixed_uuid(byte: u8) -> Uuid {
        Uuid::from_bytes([byte; 16])
    }

    fn fixed_timestamp() -> chrono::DateTime<Utc> {
        chrono::DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn bare_event() -> JobCompleteEvent {
        let mut event =
            JobCompleteEvent::new(fixed_uuid(0x11), fixed_uuid(0x22), JobResult::Success);
        event.timestamp = fixed_timestamp();
        event
    }

    /// Locks the wire shape emitted when there is no `RunSummary` (runner
    /// returned `Err`): must be byte-identical to the pre-Task-10 shape —
    /// no new keys — so the existing Go subscriber stays safe.
    #[test]
    fn event_without_summary_has_pre_task_wire_shape_exactly() {
        let event = bare_event();
        let json = serde_json::to_string_pretty(&event).unwrap();
        let expected = r#"{
  "workflowId": "11111111-1111-1111-1111-111111111111",
  "jobId": "22222222-2222-2222-2222-222222222222",
  "result": "success",
  "timestamp": "2026-01-01T00:00:00Z"
}"#;
        assert_eq!(json, expected);
    }

    fn sample_summary() -> RunSummary {
        let mut failed = Diagnostic::from_draft(
            DiagnosticDraft::new(ErrorCode::InternalInvariantViolation),
            Some("node-1".to_string()),
            Some("Some Action".to_string()),
            None,
        );
        failed.effective_disposition = Some(reearth_flow_diagnostics::Disposition::Fatal);

        let mut aggregated = Diagnostic::from_draft(
            DiagnosticDraft::new(ErrorCode::GltfZeroFaceSolid),
            Some("node-2".to_string()),
            Some("Gltf Writer".to_string()),
            None,
        );
        aggregated.aggregated = Some(AggregateInfo {
            count: 5,
            sample_feature_ids: vec![fixed_uuid(0x33)],
        });

        RunSummary {
            failed_nodes: vec![failed],
            aggregated_diagnostics: vec![aggregated],
            dropped_event_count: 2,
        }
    }

    fn event_with_summary() -> JobCompleteEvent {
        let mut event = JobCompleteEvent::with_summary(
            fixed_uuid(0x11),
            fixed_uuid(0x22),
            JobResult::Failed,
            &sample_summary(),
        );
        event.timestamp = fixed_timestamp();
        event
    }

    #[test]
    fn event_with_summary_serializes_camel_case_diagnostics_fields_with_correct_counts() {
        let event = event_with_summary();
        let value: serde_json::Value = serde_json::to_value(&event).unwrap();
        let obj = value.as_object().unwrap();
        assert!(obj.contains_key("failedNodes"));
        assert!(obj.contains_key("aggregatedDiagnostics"));
        assert!(obj.contains_key("droppedEventCount"));
        assert_eq!(obj["droppedEventCount"], 2);
        assert_eq!(obj["failedNodes"].as_array().unwrap().len(), 1);
        assert_eq!(obj["aggregatedDiagnostics"].as_array().unwrap().len(), 1);
        assert_eq!(obj["aggregatedDiagnostics"][0]["aggregated"]["count"], 5);
        assert_eq!(
            obj["failedNodes"][0]["code"],
            "internal.invariant_violation"
        );
        assert_eq!(
            obj["aggregatedDiagnostics"][0]["code"],
            "gltf.zero_face_solid"
        );
    }

    #[test]
    fn event_with_summary_round_trips() {
        let event = event_with_summary();
        let json = serde_json::to_string(&event).unwrap();
        let back: JobCompleteEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(back.workflow_id, event.workflow_id);
        assert_eq!(back.job_id, event.job_id);
        assert_eq!(back.timestamp, event.timestamp);
        assert_eq!(back.dropped_event_count, event.dropped_event_count);
        assert_eq!(
            back.failed_nodes.as_ref().unwrap().len(),
            event.failed_nodes.as_ref().unwrap().len()
        );
        assert_eq!(
            back.failed_nodes.as_ref().unwrap()[0].code,
            event.failed_nodes.as_ref().unwrap()[0].code
        );
        assert_eq!(
            back.aggregated_diagnostics.as_ref().unwrap()[0].code,
            event.aggregated_diagnostics.as_ref().unwrap()[0].code
        );
        assert_eq!(
            back.aggregated_diagnostics.as_ref().unwrap()[0]
                .aggregated
                .as_ref()
                .unwrap()
                .count,
            5
        );
    }

    #[test]
    fn event_without_summary_round_trips_with_none_fields() {
        let event = bare_event();
        let json = serde_json::to_string(&event).unwrap();
        let back: JobCompleteEvent = serde_json::from_str(&json).unwrap();
        assert!(back.failed_nodes.is_none());
        assert!(back.aggregated_diagnostics.is_none());
        assert!(back.dropped_event_count.is_none());
    }

    /// An empty `RunSummary` (no failed nodes, no aggregated diagnostics, no
    /// dropped events) is still a `Some` summary — deliberately distinct
    /// from `new()`'s `None` fields (see
    /// `event_without_summary_has_pre_task_wire_shape_exactly`). The wire
    /// decision: `with_summary` always renders these as PRESENT keys
    /// (`Some(vec![])` / `Some(0)`), i.e. `[]` / `0`, never omitted, so a
    /// consumer can distinguish "ran, produced nothing to report" from
    /// "the runner returned `Err` before a summary existed at all".
    #[test]
    fn with_summary_on_empty_run_summary_serializes_present_empty_fields() {
        let empty = RunSummary {
            failed_nodes: vec![],
            aggregated_diagnostics: vec![],
            dropped_event_count: 0,
        };
        let mut event = JobCompleteEvent::with_summary(
            fixed_uuid(0x11),
            fixed_uuid(0x22),
            JobResult::Success,
            &empty,
        );
        event.timestamp = fixed_timestamp();

        let json = serde_json::to_string_pretty(&event).unwrap();
        let expected = r#"{
  "workflowId": "11111111-1111-1111-1111-111111111111",
  "jobId": "22222222-2222-2222-2222-222222222222",
  "result": "success",
  "timestamp": "2026-01-01T00:00:00Z",
  "failedNodes": [],
  "aggregatedDiagnostics": [],
  "droppedEventCount": 0
}"#;
        assert_eq!(json, expected);

        let value: serde_json::Value = serde_json::to_value(&event).unwrap();
        let obj = value.as_object().unwrap();
        assert!(obj.contains_key("failedNodes"));
        assert!(obj.contains_key("aggregatedDiagnostics"));
        assert!(obj.contains_key("droppedEventCount"));
    }

    /// 2a-core review rec 4 hardening: `with_summary` now caps internally,
    /// so a caller that passes an *uncapped* oversized `RunSummary` (unlike
    /// the single production call site, which used to have to remember
    /// `summary.capped(JOB_COMPLETE_TOP_K)` itself) still gets a bounded
    /// wire payload. `JOB_COMPLETE_TOP_K + 1` distinct failed nodes go in;
    /// exactly `JOB_COMPLETE_TOP_K` real entries plus one
    /// `internal.diagnostics_overflow` marker come out.
    #[test]
    fn with_summary_caps_an_oversized_run_summary_internally() {
        let failed_nodes: Vec<Diagnostic> = (0..JOB_COMPLETE_TOP_K + 1)
            .map(|i| {
                Diagnostic::from_draft(
                    DiagnosticDraft::new(ErrorCode::InternalInvariantViolation),
                    Some(format!("node-{i}")),
                    Some("Some Action".to_string()),
                    None,
                )
            })
            .collect();
        let oversized = RunSummary {
            failed_nodes,
            aggregated_diagnostics: vec![],
            dropped_event_count: 0,
        };

        let event = JobCompleteEvent::with_summary(
            fixed_uuid(0x11),
            fixed_uuid(0x22),
            JobResult::Failed,
            &oversized,
        );

        let wire_failed_nodes = event
            .failed_nodes
            .expect("with_summary always populates failed_nodes");
        assert_eq!(wire_failed_nodes.len(), JOB_COMPLETE_TOP_K + 1);
        let overflow_count = wire_failed_nodes
            .iter()
            .filter(|d| d.code == "internal.diagnostics_overflow")
            .count();
        assert_eq!(
            overflow_count, 1,
            "exactly one overflow marker must be present, got: {wire_failed_nodes:?}"
        );
    }

    #[test]
    fn job_complete_topic_defaults_when_env_unset() {
        // Only asserts the default; explicitly setting/unsetting the env var
        // here would race with other tests reading the same process-global
        // `Lazy`, since whichever test's environment is visible at first
        // read wins for the lifetime of the process.
        let event = bare_event();
        assert_eq!(event.topic().to_string(), "flow-job-complete-topic");
    }
}
