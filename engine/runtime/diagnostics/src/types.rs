use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ErrorCode;

/// Log level. Orthogonal to control flow (see `Disposition`).
/// `Fatal` is a rendering level only — never read `severity` for run-fatality.
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

/// Machine-routable bucket for UI grouping/colorizing. Closed set, stable.
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

/// What the engine should do about this at runtime. There is no `Silent`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Disposition {
    WarnDrop,
    Reject,
    Fatal,
}

/// Best-effort source location for expression errors (miette interop lands later).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSpan {
    pub offset: usize,
    pub length: Option<usize>,
}

/// Counts + samples for aggregated summaries. Carried inside `Diagnostic` so
/// consumers rank/read counts structurally, never by parsing the message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AggregateInfo {
    pub count: u64,
    pub sample_feature_ids: Vec<Uuid>,
}

/// The single object that flows through logs, events, the wire, and the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub code: ErrorCode,
    pub category: ErrorCategory,
    pub severity: Severity,
    /// Registry default, stamped by `from_draft`.
    pub default_disposition: Disposition,
    /// Post-resolve; the source of truth for fatality. `None` until resolved
    /// (and permanently `None` for warn-and-continue diagnostics).
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

/// What a call site authors. Everything else is stamped from the registry
/// via `Diagnostic::from_draft` — the only construction path the runtime's
/// reporting surfaces use, so diagnostics they emit cannot disagree with
/// the registry.
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

/// Terminal fold of a DAG run, produced by `DagExecutorJoinHandle::join`
/// (engine/runtime/runtime) once every node thread has been collected.
/// `failed_nodes` holds one `Diagnostic` per node whose thread returned
/// `Err` (recovered from the structured fatal-backstop error where
/// possible, else synthesized); `aggregated_diagnostics` holds every
/// finish()-time summary emitted across all nodes, in collection order.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunSummary {
    pub failed_nodes: Vec<Diagnostic>,
    pub aggregated_diagnostics: Vec<Diagnostic>,
    pub dropped_event_count: u64,
}

/// Ranks `items` by `aggregated.count` descending (entries with no count
/// sort last; ties/None-vs-None keep their original relative order via a
/// stable sort). When `items.len() <= k` this is a no-op clone. Otherwise
/// the top `k` are kept (in ranked order) and everything else collapses
/// into a single `internal.diagnostics_overflow` marker Diagnostic whose
/// `aggregated.count` is the sum of the collapsed entries' counts (an
/// entry with no `aggregated.count` contributes 1) and whose
/// `effective_disposition` is `overflow_disposition`.
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
    /// Top-K bound for wire payloads (spec 4.7): caps both `failed_nodes`
    /// and `aggregated_diagnostics` independently at `k` entries each,
    /// ranked by `aggregated.count` descending, collapsing any remainder
    /// into one overflow-marker `Diagnostic` per list. `dropped_event_count`
    /// passes through unchanged.
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
        // passthrough: original order preserved, not re-sorted
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
        assert_eq!(capped.aggregated_diagnostics.len(), 3); // 2 kept + 1 overflow marker
        let overflow = capped.aggregated_diagnostics.last().unwrap();
        assert_eq!(overflow.code, ErrorCode::InternalDiagnosticsOverflow);
        // residual = the single collapsed entry (count 2, the smallest of the three)
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
        // k=3 == len, so passthrough — order unchanged, no sort applied
        let passthrough = summary.capped(3);
        assert_eq!(
            passthrough.aggregated_diagnostics[0].code,
            ErrorCode::GltfZeroFaceSolid
        );

        // k=2 forces a sort: the Some(4) entry must come first, the two
        // None entries retain their relative order (stable) and the later
        // of the two collapses into the overflow marker.
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
        // one collapsed None-count entry contributes 1 to the residual total
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
        assert_eq!(capped.failed_nodes.len(), 2); // 1 kept + 1 overflow marker
        let overflow = capped.failed_nodes.last().unwrap();
        assert_eq!(overflow.code, ErrorCode::InternalDiagnosticsOverflow);
        assert_eq!(overflow.aggregated.as_ref().unwrap().count, 2);
        assert_eq!(overflow.effective_disposition, Some(Disposition::Fatal));
    }
}
