use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ErrorCode;

/// `Fatal` severity is a display level only; disposition (not severity) determines run-fatality.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

impl Severity {
    pub fn label(&self) -> &'static str {
        match self {
            Severity::Trace => "TRACE",
            Severity::Debug => "DEBUG",
            Severity::Info => "INFO",
            Severity::Warn => "WARN",
            Severity::Error => "ERROR",
            Severity::Fatal => "FATAL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCategory {
    Io,
    Parse,
    Validation,
    Geometry,
    Schema,
    Expression,
    Config,
    Network,
    Resource,
    Internal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Disposition {
    WarnDrop,
    Reject,
    Fatal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSpan {
    pub offset: usize,
    pub length: Option<usize>,
}

/// Consumers must read `aggregated.count` structurally — never parse the message for counts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AggregateInfo {
    pub count: u64,
    pub sample_feature_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub code: ErrorCode,
    pub category: ErrorCategory,
    pub severity: Severity,
    pub default_disposition: Disposition,
    /// Source of truth for fatality; `None` until resolved (permanently `None` for warn-and-continue).
    pub effective_disposition: Option<Disposition>,
    pub node_id: Option<String>,
    pub action_type: Option<String>,
    pub feature_id: Option<Uuid>,
    pub message: String,
    pub help: Option<String>,
    pub source_span: Option<SourceSpan>,
    /// `Some` for finish()-time summaries; `None` for per-feature/fatal diagnostics.
    pub aggregated: Option<AggregateInfo>,
}

impl Diagnostic {
    pub fn from_draft(
        draft: DiagnosticDraft,
        node_id: Option<String>,
        action_type: Option<String>,
        feature_id: Option<Uuid>,
    ) -> Self {
        let code = draft.code;
        let default_disposition = code.default_disposition();
        Self {
            code,
            category: code.category(),
            severity: draft.severity.unwrap_or(match default_disposition {
                Disposition::Fatal => Severity::Fatal,
                _ => Severity::Warn,
            }),
            default_disposition,
            effective_disposition: None,
            node_id,
            action_type,
            feature_id,
            message: draft
                .message
                .unwrap_or_else(|| code.default_message().to_string()),
            help: draft
                .help
                .or_else(|| code.default_help().map(str::to_string)),
            source_span: draft.source_span,
            aggregated: None,
        }
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.severity.label(), self.code.as_str())?;
        if let Some(node_id) = &self.node_id {
            write!(f, " @ {node_id}")?;
            if let Some(action_type) = &self.action_type {
                write!(f, " ({action_type})")?;
            }
        }
        write!(f, ": {}", self.message)
    }
}

impl std::error::Error for Diagnostic {}

/// `from_draft` is the only construction path reporting surfaces use, so emitted diagnostics can't disagree with the registry.
#[derive(Debug, Clone)]
pub struct DiagnosticDraft {
    pub code: ErrorCode,
    pub severity: Option<Severity>,
    pub message: Option<String>,
    pub help: Option<String>,
    pub source_span: Option<SourceSpan>,
}

impl DiagnosticDraft {
    pub fn new(code: ErrorCode) -> Self {
        Self {
            code,
            severity: None,
            message: None,
            help: None,
            source_span: None,
        }
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = Some(severity);
        self
    }

    pub fn with_source_span(mut self, span: SourceSpan) -> Self {
        self.source_span = Some(span);
        self
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunSummary {
    pub failed_nodes: Vec<Diagnostic>,
    pub aggregated_diagnostics: Vec<Diagnostic>,
    pub dropped_event_count: u64,
}

/// No-count entries sort last; when collapsed into the overflow marker they each contribute a count of 1.
fn cap_diagnostics(
    items: &[Diagnostic],
    k: usize,
    overflow_disposition: Disposition,
) -> Vec<Diagnostic> {
    if items.len() <= k {
        return items.to_vec();
    }
    let mut ranked: Vec<&Diagnostic> = items.iter().collect();
    ranked.sort_by(|a, b| {
        let ca = a.aggregated.as_ref().map(|info| info.count);
        let cb = b.aggregated.as_ref().map(|info| info.count);
        match (ca, cb) {
            (Some(x), Some(y)) => y.cmp(&x),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
    });
    let mut kept: Vec<Diagnostic> = ranked[..k].iter().map(|d| (*d).clone()).collect();
    let residual_total: u64 = ranked[k..]
        .iter()
        .map(|d| d.aggregated.as_ref().map(|info| info.count).unwrap_or(1))
        .sum();
    let mut overflow = Diagnostic::from_draft(
        DiagnosticDraft::new(ErrorCode::InternalDiagnosticsOverflow),
        None,
        None,
        None,
    );
    overflow.aggregated = Some(AggregateInfo {
        count: residual_total,
        sample_feature_ids: vec![],
    });
    overflow.effective_disposition = Some(overflow_disposition);
    kept.push(overflow);
    kept
}

impl RunSummary {
    pub fn capped(&self, k: usize) -> RunSummary {
        RunSummary {
            failed_nodes: cap_diagnostics(&self.failed_nodes, k, Disposition::Fatal),
            aggregated_diagnostics: cap_diagnostics(
                &self.aggregated_diagnostics,
                k,
                Disposition::WarnDrop,
            ),
            dropped_event_count: self.dropped_event_count,
        }
    }
}

#[cfg(test)]
mod run_summary_capped_tests {
    use super::*;
    use crate::ErrorCode;
    use pretty_assertions::assert_eq;

    fn diag_with_count(code: ErrorCode, count: u64) -> Diagnostic {
        let mut d = Diagnostic::from_draft(DiagnosticDraft::new(code), None, None, None);
        d.aggregated = Some(AggregateInfo {
            count,
            sample_feature_ids: vec![],
        });
        d
    }

    fn diag_no_count(code: ErrorCode) -> Diagnostic {
        Diagnostic::from_draft(DiagnosticDraft::new(code), None, None, None)
    }

    #[test]
    fn under_k_is_passthrough_clone() {
        let summary = RunSummary {
            failed_nodes: vec![diag_no_count(ErrorCode::InternalInvariantViolation)],
            aggregated_diagnostics: vec![
                diag_with_count(ErrorCode::GltfZeroFaceSolid, 5),
                diag_with_count(ErrorCode::Cesium3dtilesEmptyGeometry, 2),
            ],
            dropped_event_count: 7,
        };
        let capped = summary.capped(10);
        assert_eq!(capped.failed_nodes.len(), 1);
        assert_eq!(capped.aggregated_diagnostics.len(), 2);
        assert_eq!(capped.dropped_event_count, 7);
        assert_eq!(
            capped.aggregated_diagnostics[0].code,
            ErrorCode::GltfZeroFaceSolid
        );
        assert_eq!(
            capped.aggregated_diagnostics[1].code,
            ErrorCode::Cesium3dtilesEmptyGeometry
        );
    }

    #[test]
    fn len_equal_to_k_is_passthrough_no_overflow_marker() {
        let summary = RunSummary {
            failed_nodes: vec![],
            aggregated_diagnostics: vec![
                diag_with_count(ErrorCode::GltfZeroFaceSolid, 5),
                diag_with_count(ErrorCode::Cesium3dtilesEmptyGeometry, 2),
            ],
            dropped_event_count: 0,
        };
        let capped = summary.capped(2);
        assert_eq!(capped.aggregated_diagnostics.len(), 2);
        assert!(capped
            .aggregated_diagnostics
            .iter()
            .all(|d| d.code != ErrorCode::InternalDiagnosticsOverflow));
    }

    #[test]
    fn over_k_collapses_remainder_with_correct_residual_total() {
        let summary = RunSummary {
            failed_nodes: vec![],
            aggregated_diagnostics: vec![
                diag_with_count(ErrorCode::GltfZeroFaceSolid, 5),
                diag_with_count(ErrorCode::Cesium3dtilesEmptyGeometry, 2),
                diag_with_count(ErrorCode::Cesium3dtilesNonCitygmlGeometry, 8),
            ],
            dropped_event_count: 0,
        };
        let capped = summary.capped(2);
        assert_eq!(capped.aggregated_diagnostics.len(), 3);
        let overflow = capped.aggregated_diagnostics.last().unwrap();
        assert_eq!(overflow.code, ErrorCode::InternalDiagnosticsOverflow);
        assert_eq!(overflow.aggregated.as_ref().unwrap().count, 2);
        assert!(overflow
            .aggregated
            .as_ref()
            .unwrap()
            .sample_feature_ids
            .is_empty());
        assert_eq!(overflow.effective_disposition, Some(Disposition::WarnDrop));
    }

    #[test]
    fn kept_entries_are_ordered_by_count_descending() {
        let summary = RunSummary {
            failed_nodes: vec![],
            aggregated_diagnostics: vec![
                diag_with_count(ErrorCode::GltfZeroFaceSolid, 3),
                diag_with_count(ErrorCode::Cesium3dtilesEmptyGeometry, 9),
                diag_with_count(ErrorCode::Cesium3dtilesNonCitygmlGeometry, 6),
            ],
            dropped_event_count: 0,
        };
        let capped = summary.capped(2);
        assert_eq!(
            capped.aggregated_diagnostics[0].code,
            ErrorCode::Cesium3dtilesEmptyGeometry
        );
        assert_eq!(
            capped.aggregated_diagnostics[0]
                .aggregated
                .as_ref()
                .unwrap()
                .count,
            9
        );
        assert_eq!(
            capped.aggregated_diagnostics[1].code,
            ErrorCode::Cesium3dtilesNonCitygmlGeometry
        );
        assert_eq!(
            capped.aggregated_diagnostics[1]
                .aggregated
                .as_ref()
                .unwrap()
                .count,
            6
        );
    }

    #[test]
    fn none_count_entries_sort_last_and_stable_among_themselves() {
        let summary = RunSummary {
            failed_nodes: vec![],
            aggregated_diagnostics: vec![
                diag_no_count(ErrorCode::GltfZeroFaceSolid),
                diag_with_count(ErrorCode::Cesium3dtilesEmptyGeometry, 4),
                diag_no_count(ErrorCode::Cesium3dtilesNonCitygmlGeometry),
            ],
            dropped_event_count: 0,
        };
        let passthrough = summary.capped(3);
        assert_eq!(
            passthrough.aggregated_diagnostics[0].code,
            ErrorCode::GltfZeroFaceSolid
        );

        let capped = summary.capped(2);
        assert_eq!(
            capped.aggregated_diagnostics[0].code,
            ErrorCode::Cesium3dtilesEmptyGeometry
        );
        assert_eq!(
            capped.aggregated_diagnostics[1].code,
            ErrorCode::GltfZeroFaceSolid
        );
        let overflow = capped.aggregated_diagnostics.last().unwrap();
        assert_eq!(overflow.code, ErrorCode::InternalDiagnosticsOverflow);
        assert_eq!(overflow.aggregated.as_ref().unwrap().count, 1);
    }

    #[test]
    fn failed_nodes_overflow_marker_keeps_fatal_disposition_and_counts_entries() {
        let summary = RunSummary {
            failed_nodes: vec![
                diag_no_count(ErrorCode::InternalInvariantViolation),
                diag_no_count(ErrorCode::InternalUnclassified),
                diag_no_count(ErrorCode::InternalInvariantViolation),
            ],
            aggregated_diagnostics: vec![],
            dropped_event_count: 0,
        };
        let capped = summary.capped(1);
        assert_eq!(capped.failed_nodes.len(), 2);
        let overflow = capped.failed_nodes.last().unwrap();
        assert_eq!(overflow.code, ErrorCode::InternalDiagnosticsOverflow);
        assert_eq!(overflow.aggregated.as_ref().unwrap().count, 2);
        assert_eq!(overflow.effective_disposition, Some(Disposition::Fatal));
    }
}
