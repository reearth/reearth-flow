//! Tracing and telemetry configuration module.

/// Tracing and telemetry configuration.
#[derive(Debug, Clone)]
pub struct TracingConfig {
    /// Whether to enable Google Cloud Trace
    pub enable_cloud_trace: bool,
    /// Whether to enable local OTLP export
    pub enable_otlp: bool,
    /// OTLP endpoint
    pub otlp_endpoint: String,
    /// Google Cloud Project ID
    pub gcp_project_id: Option<String>,
    /// Service name for tracing
    pub service_name: String,
    /// Log level filter
    pub log_level: String,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enable_cloud_trace: true,
            enable_otlp: false,
            otlp_endpoint: "http://localhost:4317".to_string(),
            gcp_project_id: Some("reearth-oss".to_string()),
            service_name: "reearth-flow-websocket".to_string(),
            log_level: "info".to_string(),
        }
    }
}

/// Convert to infrastructure TracingConfig for use with tracing initialization
impl From<TracingConfig> for crate::infrastructure::tracing::TracingConfig {
    fn from(config: TracingConfig) -> Self {
        Self {
            enable_cloud_trace: config.enable_cloud_trace,
            enable_otlp: config.enable_otlp,
            otlp_endpoint: config.otlp_endpoint,
            gcp_project_id: config.gcp_project_id,
            service_name: config.service_name,
            log_level: config.log_level,
        }
    }
}

impl From<&TracingConfig> for crate::infrastructure::tracing::TracingConfig {
    fn from(config: &TracingConfig) -> Self {
        Self {
            enable_cloud_trace: config.enable_cloud_trace,
            enable_otlp: config.enable_otlp,
            otlp_endpoint: config.otlp_endpoint.clone(),
            gcp_project_id: config.gcp_project_id.clone(),
            service_name: config.service_name.clone(),
            log_level: config.log_level.clone(),
        }
    }
}
