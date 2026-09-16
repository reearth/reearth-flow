use std::env;
use std::sync::Mutex;

use once_cell::sync::Lazy;
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::{MetricExporter, WithExportConfig};
use opentelemetry_sdk::{
    metrics::{PeriodicReader, SdkMeterProvider},
    trace::{SdkTracerProvider, Tracer},
};

// Empty/whitespace treated as unset — deployment templating commonly leaves this env var as an empty string.
static OTEL_COLLECTOR_ENDPOINT: Lazy<Mutex<Option<String>>> = Lazy::new(|| {
    Mutex::new(
        env::var("OTEL_COLLECTOR_ENDPOINT")
            .ok()
            .filter(|v| !v.trim().is_empty()),
    )
});

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("MetricsError: {0}")]
    Metrics(String),

    #[error("TracingError: {0}")]
    Tracing(String),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Caller MUST call `shutdown()` on the returned provider before exit; dropping it (or `std::process::exit`) discards unflushed metrics.
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

/// The `Tracer` alone can't shut down the exporter — caller MUST keep the provider alive and call `shutdown()` on it before exit, or buffered spans are lost.
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
