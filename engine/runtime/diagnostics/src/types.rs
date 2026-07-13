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
                Disposition::Fatal => Severity::Error,
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
/// and the executor context — no construction path can disagree with the registry.
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
