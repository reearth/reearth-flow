use serde::{Deserialize, Serialize};

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
