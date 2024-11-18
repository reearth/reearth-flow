use std::env;
use std::sync::Mutex;

use once_cell::sync::Lazy;
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::{MetricExporter, WithExportConfig};
use opentelemetry_sdk::{
    metrics::{MeterProviderBuilder, PeriodicReader, SdkMeterProvider},
    trace::{Config, Tracer},
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
        Some(endpoint) => {
            let exporter = MetricExporter::builder()
                .with_tonic()
                .with_endpoint(endpoint)
                .build()
                .map_err(|e| {
                    Error::Metrics(format!("Failed to build metrics controller: {}", e))
                })?;
            let reader =
                PeriodicReader::builder(exporter, opentelemetry_sdk::runtime::Tokio).build();

            SdkMeterProvider::builder()
                .with_reader(reader)
                .with_resource(opentelemetry_sdk::Resource::new(vec![
                    opentelemetry::KeyValue::new(
                        opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                        service_name,
                    ),
                ]))
                .build()
        }
        None => MeterProviderBuilder::default()
            .with_reader(
                opentelemetry_sdk::metrics::PeriodicReader::builder(
                    opentelemetry_stdout::MetricExporter::default(),
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
        Some(endpoint) => opentelemetry_sdk::trace::TracerProvider::builder()
            .with_config(
                Config::default()
                    .with_sampler(opentelemetry_sdk::trace::Sampler::AlwaysOn)
                    .with_id_generator(opentelemetry_sdk::trace::RandomIdGenerator::default())
                    .with_resource(opentelemetry_sdk::Resource::new(vec![
                        opentelemetry::KeyValue::new(
                            opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                            service_name.clone(),
                        ),
                    ])),
            )
            .with_batch_exporter(
                opentelemetry_otlp::SpanExporter::builder()
                    .with_tonic()
                    .with_endpoint(endpoint)
                    .build()
                    .map_err(|e| {
                        Error::Tracing(format!("Failed to build tracing exporter: {}", e))
                    })?,
                opentelemetry_sdk::runtime::Tokio,
            )
            .build(),
        None => opentelemetry_sdk::trace::TracerProvider::builder()
            .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
            .with_config(
                Config::default()
                    .with_sampler(opentelemetry_sdk::trace::Sampler::AlwaysOn)
                    .with_id_generator(opentelemetry_sdk::trace::RandomIdGenerator::default())
                    .with_resource(opentelemetry_sdk::Resource::new(vec![
                        opentelemetry::KeyValue::new(
                            opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                            service_name.clone(),
                        ),
                    ])),
            )
            .build(),
    };
    Ok(tracer.tracer(service_name.clone()))
}
