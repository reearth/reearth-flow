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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failed_nodes: Option<Vec<WireDiagnostic>>,
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

    /// Builds the event with structured diagnostics populated from `summary`
    /// (the caller is expected to have already bounded it, e.g. via
    /// `summary.capped(JOB_COMPLETE_TOP_K)`). Use `new` instead when there is
    /// no summary to report.
    pub fn with_summary(
        workflow_id: Uuid,
        job_id: Uuid,
        result: JobResult,
        summary: &RunSummary,
    ) -> Self {
        Self {
            failed_nodes: Some(
                summary
                    .failed_nodes
                    .iter()
                    .map(WireDiagnostic::from)
                    .collect(),
            ),
            aggregated_diagnostics: Some(
                summary
                    .aggregated_diagnostics
                    .iter()
                    .map(WireDiagnostic::from)
                    .collect(),
            ),
            dropped_event_count: Some(summary.dropped_event_count),
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
