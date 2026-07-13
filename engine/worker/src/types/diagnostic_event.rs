use std::env;

use bytes::Bytes;
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use reearth_flow_diagnostics::{AggregateInfo, Diagnostic, SourceSpan};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::pubsub::{
    message::{EncodableMessage, ValidatedMessage},
    topic::Topic,
};

/// Gate for `DiagnosticEvent` publishing. Defaults to `false`: as of Phase
/// 2a the Go subscriber for `flow-diagnostic-topic` does not exist yet, so
/// publishing must stay off until that consumer ships (deploy-order step 1
/// in the diagnostics wire spec — publish only after a consumer exists).
pub static ENABLE_DIAGNOSTICS: Lazy<bool> = Lazy::new(|| {
    env::var("FLOW_WORKER_ENABLE_DIAGNOSTICS")
        .ok()
        .map(|s| s.to_lowercase() == "true")
        .unwrap_or(false)
});

static DIAGNOSTIC_TOPIC: Lazy<String> = Lazy::new(|| {
    env::var("FLOW_WORKER_DIAGNOSTIC_TOPIC")
        .ok()
        .unwrap_or("flow-diagnostic-topic".to_string())
});

/// Wire mirror of `reearth_flow_diagnostics::SourceSpan`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WireSourceSpan {
    pub offset: usize,
    pub length: Option<usize>,
}

impl From<&SourceSpan> for WireSourceSpan {
    fn from(span: &SourceSpan) -> Self {
        Self {
            offset: span.offset,
            length: span.length,
        }
    }
}

/// Wire mirror of `reearth_flow_diagnostics::AggregateInfo`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WireAggregateInfo {
    pub count: u64,
    pub sample_feature_ids: Vec<Uuid>,
}

impl From<&AggregateInfo> for WireAggregateInfo {
    fn from(info: &AggregateInfo) -> Self {
        Self {
            count: info.count,
            sample_feature_ids: info.sample_feature_ids.clone(),
        }
    }
}

/// Serializes an enum whose derived `Serialize` renders it as a bare JSON
/// string (every `snake_case`-renamed enum in the diagnostics crate) into
/// that string. Kept generic instead of hand-written per-enum `match`es so
/// this mapping cannot silently fall out of sync with the diagnostics
/// crate's own enum variants.
fn enum_to_string<T: Serialize>(value: &T) -> String {
    match serde_json::to_value(value) {
        Ok(serde_json::Value::String(s)) => s,
        other => unreachable!("expected enum to serialize to a JSON string, got {other:?}"),
    }
}

/// Wire form of `Diagnostic`. Enums are carried as their `snake_case`
/// string values rather than native/tagged JSON enums so unknown or newer
/// values survive a Go round-trip verbatim instead of failing to
/// deserialize (forward-compat requirement, spec diagnostic.v1 4.7).
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WireDiagnostic {
    pub code: String,
    pub category: String,
    pub severity: String,
    pub effective_disposition: Option<String>,
    pub node_id: Option<String>,
    pub action_type: Option<String>,
    pub feature_id: Option<Uuid>,
    pub message: String,
    pub help: Option<String>,
    pub source_span: Option<WireSourceSpan>,
    pub aggregated: Option<WireAggregateInfo>,
}

impl From<&Diagnostic> for WireDiagnostic {
    fn from(d: &Diagnostic) -> Self {
        Self {
            code: d.code.as_str().to_string(),
            category: enum_to_string(&d.category),
            severity: enum_to_string(&d.severity),
            effective_disposition: d.effective_disposition.as_ref().map(enum_to_string),
            node_id: d.node_id.clone(),
            action_type: d.action_type.clone(),
            feature_id: d.feature_id,
            message: d.message.clone(),
            help: d.help.clone(),
            source_span: d.source_span.as_ref().map(WireSourceSpan::from),
            aggregated: d.aggregated.as_ref().map(WireAggregateInfo::from),
        }
    }
}

/// Pubsub wire event for a single `Diagnostic`. Gated by
/// `ENABLE_DIAGNOSTICS` (see above) — construction itself is unconditional,
/// callers decide whether to publish it.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticEvent {
    /// Wire schema tag for this event shape. Always `"diagnostic.v1"`.
    pub schema: String,
    pub workflow_id: Uuid,
    pub job_id: Uuid,
    #[serde(flatten)]
    pub diagnostic: WireDiagnostic,
    pub timestamp: DateTime<Utc>,
}

impl DiagnosticEvent {
    pub fn new(workflow_id: Uuid, job_id: Uuid, diagnostic: &Diagnostic) -> Self {
        Self {
            schema: "diagnostic.v1".to_string(),
            workflow_id,
            job_id,
            diagnostic: WireDiagnostic::from(diagnostic),
            timestamp: Utc::now(),
        }
    }
}

impl EncodableMessage for DiagnosticEvent {
    type Error = crate::errors::Error;

    fn topic(&self) -> Topic {
        Topic::new(DIAGNOSTIC_TOPIC.clone())
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
    use reearth_flow_diagnostics::{Disposition, ErrorCategory, ErrorCode, Severity};

    use super::*;

    fn fixed_uuid(byte: u8) -> Uuid {
        Uuid::from_bytes([byte; 16])
    }

    /// A fully-populated `Diagnostic`: every optional field set except
    /// `feature_id`, which is `None` because this is an aggregated
    /// (finish()-time, node-level) summary rather than a per-feature one —
    /// the two are mutually exclusive in practice (see `Diagnostic` doc
    /// comment in the diagnostics crate).
    fn full_diagnostic() -> Diagnostic {
        Diagnostic {
            code: ErrorCode::Cesium3dtilesEmptyGeometry,
            category: ErrorCategory::Geometry,
            severity: Severity::Warn,
            default_disposition: Disposition::WarnDrop,
            effective_disposition: Some(Disposition::WarnDrop),
            node_id: Some("node-1".to_string()),
            action_type: Some("Cesium 3D Tiles Writer".to_string()),
            feature_id: None,
            message: "3 features skipped due to empty geometry".to_string(),
            help: Some(
                "Filter empty-geometry features upstream or fix the source data.".to_string(),
            ),
            source_span: Some(SourceSpan {
                offset: 42,
                length: Some(7),
            }),
            aggregated: Some(AggregateInfo {
                count: 3,
                sample_feature_ids: vec![fixed_uuid(0x33), fixed_uuid(0x44)],
            }),
        }
    }

    fn fixed_event() -> DiagnosticEvent {
        DiagnosticEvent {
            schema: "diagnostic.v1".to_string(),
            workflow_id: fixed_uuid(0x11),
            job_id: fixed_uuid(0x22),
            diagnostic: WireDiagnostic::from(&full_diagnostic()),
            timestamp: DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
        }
    }

    /// Explicit nulls, not omission: this mirrors `NodeStatusEvent` and
    /// `LogStreamEvent`, which also serialize their `Option` fields as
    /// literal `null` rather than using `skip_serializing_if`. Kept
    /// consistent with that convention rather than introducing a new one
    /// for just this event type.
    #[test]
    fn diagnostic_event_serializes_camel_case_with_explicit_nulls_for_absent_fields() {
        let event = fixed_event();
        let json = serde_json::to_string_pretty(&event).unwrap();
        let expected = r#"{
  "schema": "diagnostic.v1",
  "workflowId": "11111111-1111-1111-1111-111111111111",
  "jobId": "22222222-2222-2222-2222-222222222222",
  "code": "cesium3dtiles.empty_geometry",
  "category": "geometry",
  "severity": "warn",
  "effectiveDisposition": "warn_drop",
  "nodeId": "node-1",
  "actionType": "Cesium 3D Tiles Writer",
  "featureId": null,
  "message": "3 features skipped due to empty geometry",
  "help": "Filter empty-geometry features upstream or fix the source data.",
  "sourceSpan": {
    "offset": 42,
    "length": 7
  },
  "aggregated": {
    "count": 3,
    "sampleFeatureIds": [
      "33333333-3333-3333-3333-333333333333",
      "44444444-4444-4444-4444-444444444444"
    ]
  },
  "timestamp": "2026-01-01T00:00:00Z"
}"#;
        assert_eq!(json, expected);
    }

    #[test]
    fn from_diagnostic_maps_aggregated_and_span_and_none_fields() {
        let wire = WireDiagnostic::from(&full_diagnostic());
        assert_eq!(wire.code, "cesium3dtiles.empty_geometry");
        assert_eq!(wire.category, "geometry");
        assert_eq!(wire.severity, "warn");
        assert_eq!(wire.effective_disposition.as_deref(), Some("warn_drop"));
        assert_eq!(wire.node_id.as_deref(), Some("node-1"));
        assert_eq!(wire.action_type.as_deref(), Some("Cesium 3D Tiles Writer"));
        assert_eq!(wire.feature_id, None);
        assert_eq!(
            wire.source_span,
            Some(WireSourceSpan {
                offset: 42,
                length: Some(7)
            })
        );
        let aggregated = wire.aggregated.expect("aggregated should be Some");
        assert_eq!(aggregated.count, 3);
        assert_eq!(
            aggregated.sample_feature_ids,
            vec![fixed_uuid(0x33), fixed_uuid(0x44)]
        );
    }

    #[test]
    fn from_diagnostic_maps_none_fields_to_none() {
        let diagnostic = Diagnostic {
            code: ErrorCode::InternalInvariantViolation,
            category: ErrorCategory::Internal,
            severity: Severity::Fatal,
            default_disposition: Disposition::Fatal,
            effective_disposition: None,
            node_id: None,
            action_type: None,
            feature_id: Some(fixed_uuid(0x55)),
            message: "boom".to_string(),
            help: None,
            source_span: None,
            aggregated: None,
        };
        let wire = WireDiagnostic::from(&diagnostic);
        assert_eq!(wire.effective_disposition, None);
        assert_eq!(wire.node_id, None);
        assert_eq!(wire.action_type, None);
        assert_eq!(wire.feature_id, Some(fixed_uuid(0x55)));
        assert_eq!(wire.help, None);
        assert_eq!(wire.source_span, None);
        assert!(wire.aggregated.is_none());
    }

    #[test]
    fn diagnostic_topic_defaults_when_env_unset() {
        // Only asserts the default; explicitly setting/unsetting the env var
        // here would race with other tests reading the same process-global
        // `Lazy`, since whichever test's environment is visible at first
        // read wins for the lifetime of the process.
        let event = fixed_event();
        assert_eq!(event.topic().to_string(), "flow-diagnostic-topic");
    }
}
