use std::env;
use std::sync::Mutex;

use once_cell::sync::Lazy;
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::{MetricExporter, WithExportConfig};
use opentelemetry_sdk::{
    metrics::{PeriodicReader, SdkMeterProvider},
    trace::{SdkTracerProvider, Tracer},
};

static OTEL_COLLECTOR_ENDPOINT: Lazy<Mutex<Option<String>>> =
    Lazy::new(|| Mutex::new(env::var("OTEL_COLLECTOR_ENDPOINT").ok()));

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("MetricsError: {0}")]
    Metrics(String),

    #[error("TracingError: {0}")]
    Tracing(String),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Builds an OTel `SdkMeterProvider` for `service_name`. Exports over
/// OTLP/gRPC to `OTEL_COLLECTOR_ENDPOINT` when that env var is set,
/// otherwise falls back to a periodic stdout exporter — this fallback is
/// NOT a no-op, so callers that want a true no-op when the endpoint is
/// absent must gate the call to `init_metrics` itself rather than relying
/// on this function's internal branch.
///
/// The returned provider owns the periodic export reader (its own
/// background thread) and MUST be kept alive for the process lifetime.
/// Call `shutdown()` on it explicitly before the process exits — letting
/// it drop (or exiting via `std::process::exit`, which skips `Drop`)
/// discards any metrics not yet flushed.
pub fn init_metrics(service_name: String) -> Result<SdkMeterProvider> {
    let metrics = match OTEL_COLLECTOR_ENDPOINT.lock().unwrap().clone() {
        Some(endpoint) => {
            let exporter = MetricExporter::builder()
                .with_tonic()
                .with_endpoint(endpoint)
                .build()
                .map_err(|e| Error::Metrics(format!("Failed to build metrics controller: {e}")))?;
            let reader = PeriodicReader::builder(exporter).build();

            SdkMeterProvider::builder()
                .with_reader(reader)
                .with_resource(
                    opentelemetry_sdk::Resource::builder()
                        .with_attribute(opentelemetry::KeyValue::new(
                            opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                            service_name,
                        ))
                        .build(),
                )
                .build()
        }
        None => SdkMeterProvider::builder()
            .with_reader(
                opentelemetry_sdk::metrics::PeriodicReader::builder(
                    opentelemetry_stdout::MetricExporter::default(),
                )
                .build(),
            )
            .with_resource(
                opentelemetry_sdk::Resource::builder()
                    .with_attribute(opentelemetry::KeyValue::new(
                        opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                        service_name,
                    ))
                    .build(),
            )
            .build(),
    };
    Ok(metrics)
}

/// Builds an OTel `Tracer` for `service_name`, exporting over OTLP/gRPC
/// to `OTEL_COLLECTOR_ENDPOINT` when set, otherwise falling back to a
/// stdout exporter (see the `init_metrics` doc comment — that fallback
/// is not a no-op either).
///
/// Returns the `Tracer` (bridge it into a `tracing_subscriber` registry
/// via `tracing-opentelemetry`'s `layer().with_tracer(..)`) together with
/// the underlying `SdkTracerProvider`. The `Tracer` itself holds a clone
/// of the provider internally, so it alone would keep the provider's
/// batch exporter alive — but it exposes no `shutdown()`. The provider is
/// therefore returned separately, and the caller MUST keep it alive for
/// the process lifetime and call `shutdown()` on it explicitly before
/// exit: dropping it (or exiting via `std::process::exit`, which skips
/// `Drop`) silently discards any spans still buffered for export.
pub fn init_tracing(service_name: String) -> Result<(Tracer, SdkTracerProvider)> {
    let tracer_provider = match OTEL_COLLECTOR_ENDPOINT.lock().unwrap().clone() {
        Some(endpoint) => opentelemetry_sdk::trace::SdkTracerProvider::builder()
            .with_sampler(opentelemetry_sdk::trace::Sampler::AlwaysOn)
            .with_id_generator(opentelemetry_sdk::trace::RandomIdGenerator::default())
            .with_resource(
                opentelemetry_sdk::Resource::builder()
                    .with_attribute(opentelemetry::KeyValue::new(
                        opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                        service_name.clone(),
                    ))
                    .build(),
            )
            .with_batch_exporter(
                opentelemetry_otlp::SpanExporter::builder()
                    .with_tonic()
                    .with_endpoint(endpoint)
                    .build()
                    .map_err(|e| {
                        Error::Tracing(format!("Failed to build tracing exporter: {e}"))
                    })?,
            )
            .build(),
        None => opentelemetry_sdk::trace::SdkTracerProvider::builder()
            .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
            .with_sampler(opentelemetry_sdk::trace::Sampler::AlwaysOn)
            .with_id_generator(opentelemetry_sdk::trace::RandomIdGenerator::default())
            .with_resource(
                opentelemetry_sdk::Resource::builder()
                    .with_attribute(opentelemetry::KeyValue::new(
                        opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                        service_name.clone(),
                    ))
                    .build(),
            )
            .build(),
    };
    let tracer = tracer_provider.tracer(service_name);
    Ok((tracer, tracer_provider))
}
