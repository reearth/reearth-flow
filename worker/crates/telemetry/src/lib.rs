use std::env;
use std::sync::Mutex;

use once_cell::sync::Lazy;
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    metrics::{MeterProviderBuilder, SdkMeterProvider},
    trace::Tracer,
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

pub fn init_metrics(service_name: String) -> Result<SdkMeterProvider> {
    let metrics = match OTEL_COLLECTOR_ENDPOINT.lock().unwrap().clone() {
        Some(endpoint) => opentelemetry_otlp::new_pipeline()
            .metrics(opentelemetry_sdk::runtime::Tokio)
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(endpoint),
            )
            .with_resource(opentelemetry_sdk::Resource::new(vec![
                opentelemetry::KeyValue::new(
                    opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                    service_name,
                ),
            ]))
            .build()
            .map_err(|e| Error::Metrics(format!("Failed to build metrics controller: {}", e)))?,
        None => MeterProviderBuilder::default()
            .with_reader(
                opentelemetry_sdk::metrics::PeriodicReader::builder(
                    opentelemetry_stdout::MetricsExporter::default(),
                    opentelemetry_sdk::runtime::Tokio,
                )
                .build(),
            )
            .with_resource(opentelemetry_sdk::Resource::new(vec![
                opentelemetry::KeyValue::new(
                    opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                    service_name,
                ),
            ]))
            .build(),
    };
    Ok(metrics)
}

pub fn init_tracing(service_name: String) -> Result<Tracer> {
    let tracer = match OTEL_COLLECTOR_ENDPOINT.lock().unwrap().clone() {
        Some(endpoint) => opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(endpoint),
            )
            .with_trace_config(
                opentelemetry_sdk::trace::config()
                    .with_sampler(opentelemetry_sdk::trace::Sampler::AlwaysOn)
                    .with_id_generator(opentelemetry_sdk::trace::RandomIdGenerator::default())
                    .with_resource(opentelemetry_sdk::Resource::new(vec![
                        opentelemetry::KeyValue::new(
                            opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                            service_name,
                        ),
                    ])),
            )
            .install_batch(opentelemetry_sdk::runtime::Tokio)
            .map_err(|e| Error::Tracing(format!("Failed to build metrics controller: {}", e)))?,
        None => opentelemetry_sdk::trace::TracerProvider::builder()
            .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
            .with_config(
                opentelemetry_sdk::trace::config()
                    .with_sampler(opentelemetry_sdk::trace::Sampler::AlwaysOn)
                    .with_id_generator(opentelemetry_sdk::trace::RandomIdGenerator::default())
                    .with_resource(opentelemetry_sdk::Resource::new(vec![
                        opentelemetry::KeyValue::new(
                            opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                            service_name.clone(),
                        ),
                    ])),
            )
            .build()
            .tracer(service_name.clone()),
    };
    Ok(tracer)
}
