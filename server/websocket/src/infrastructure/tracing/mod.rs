//! Tracing and OpenTelemetry configuration for Google Cloud Trace integration.
//!
//! This module provides configurable tracing that supports:
//! - Console logging (always enabled)
//! - Google Cloud Trace integration (optional, enabled via environment variables)

use opentelemetry::trace::TracerProvider as _;
use opentelemetry::KeyValue;
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_sdk::Resource;
use opentelemetry_stackdriver::StackDriverExporter;
use std::time::Duration;
use tracing::Level;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

const GCP_AUTHORIZER_TIMEOUT_SECS: u64 = 10;

/// Configuration for tracing and telemetry.
#[derive(Debug, Clone)]
pub struct TracingConfig {
    /// Whether to enable Google Cloud Trace.
    pub enable_cloud_trace: bool,
    /// Google Cloud Project ID (required if cloud trace is enabled).
    pub gcp_project_id: Option<String>,
    /// Service name for tracing.
    pub service_name: String,
    /// Log level filter.
    pub log_level: String,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enable_cloud_trace: false,
            gcp_project_id: None,
            service_name: "reearth-flow-websocket".to_string(),
            log_level: "info".to_string(),
        }
    }
}

/// Global tracer provider handle for shutdown.
static TRACER_PROVIDER: std::sync::OnceLock<TracerProvider> = std::sync::OnceLock::new();

/// Initialize tracing with the given configuration.
///
/// This function sets up:
/// - Console logging with configurable format
/// - Optional Google Cloud Trace integration via OpenTelemetry
///
/// # Arguments
///
/// * `config` - Tracing configuration
///
/// # Returns
///
/// Returns `Ok(())` if initialization succeeds, or an error if it fails.
pub async fn init_tracing(
    config: &TracingConfig,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Create environment filter
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.log_level));

    // Create fmt layer for console output
    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_file(true)
        .with_line_number(true)
        .with_level(true);

    if config.enable_cloud_trace {
        let project_id = config.gcp_project_id.clone().unwrap_or_default();

        let authorizer_timeout = Duration::from_secs(GCP_AUTHORIZER_TIMEOUT_SECS);
        let authorizer = tokio::time::timeout(
            authorizer_timeout,
            opentelemetry_stackdriver::GcpAuthorizer::new(),
        )
        .await
        .map_err(|_| {
            format!(
                "GCP authorizer initialization timed out after {} seconds",
                GCP_AUTHORIZER_TIMEOUT_SECS
            )
        })?
        .map_err(|e| format!("Failed to create GCP authorizer: {}", e))?;

        let (exporter, background_task) = tokio::time::timeout(
            authorizer_timeout,
            StackDriverExporter::builder().build(authorizer),
        )
        .await
        .map_err(|_| {
            format!(
                "StackDriver exporter initialization timed out after {} seconds",
                GCP_AUTHORIZER_TIMEOUT_SECS
            )
        })?
        .map_err(|e| format!("Failed to create StackDriver exporter: {}", e))?;

        // Spawn the background task for the exporter
        tokio::spawn(background_task);

        // Create tracer provider with resource attributes
        let mut resource_attrs = vec![
            KeyValue::new("service.name", config.service_name.clone()),
            KeyValue::new("cloud.provider", "gcp"),
            KeyValue::new("cloud.platform", "gcp_cloud_run"),
        ];
        if !project_id.is_empty() {
            resource_attrs.push(KeyValue::new("gcp.project_id", project_id.clone()));
        }
        let resource = Resource::new(resource_attrs);

        let provider = TracerProvider::builder()
            .with_simple_exporter(exporter)
            .with_resource(resource)
            .build();

        // Store provider for later shutdown
        let _ = TRACER_PROVIDER.set(provider.clone());

        // Get tracer
        let tracer = provider.tracer(config.service_name.clone());

        // Create OpenTelemetry layer
        let otel_layer = OpenTelemetryLayer::new(tracer);

        // Initialize subscriber with both layers
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .with(otel_layer)
            .init();

        if project_id.is_empty() {
            tracing::info!(
                service_name = %config.service_name,
                "Google Cloud Trace initialized (project_id not configured)"
            );
        } else {
            tracing::info!(
                project_id = %project_id,
                service_name = %config.service_name,
                "Google Cloud Trace initialized"
            );
        }
    } else {
        // Initialize subscriber with only fmt layer
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .init();

        tracing::info!("Tracing initialized (console only, Cloud Trace disabled)");
    }

    Ok(())
}

/// Shutdown the tracer provider gracefully.
///
/// This should be called before the application exits to ensure all
/// pending traces are flushed to Google Cloud Trace.
pub fn shutdown_tracing() {
    if let Some(provider) = TRACER_PROVIDER.get() {
        if let Err(e) = provider.shutdown() {
            tracing::error!("Error shutting down tracer provider: {:?}", e);
        } else {
            tracing::info!("Tracer provider shut down successfully");
        }
    }
}

/// Simple initialization for development (console only).
pub fn init_tracing_simple() {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_file(true)
        .with_line_number(true)
        .init();
}
