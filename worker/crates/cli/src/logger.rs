use tracing::Level;
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

use reearth_flow_telemetry::{init_metrics, init_tracing};

pub fn setup_logging_and_tracing(level: Level, ansi_colors: bool) -> crate::Result<()> {
    let metrics_provider =
        init_metrics("reearth-flow-worker".to_string()).map_err(crate::Error::init)?;
    let tracer = init_tracing("reearth-flow-worker".to_string()).map_err(crate::Error::init)?;
    let otel_trace_layer = tracing_opentelemetry::layer().with_tracer(tracer);
    let otel_metrics_layer = tracing_opentelemetry::MetricsLayer::new(metrics_provider);

    let env_filter = EnvFilter::builder()
        .with_default_directive(level.into())
        .from_env_lossy();
    let registry = tracing_subscriber::registry().with(env_filter);
    let event_format = tracing_subscriber::fmt::format()
        .with_target(true)
        .with_timer(UtcTime::new(
            time::format_description::parse(
                "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]Z",
            )
            .expect("Time format invalid."),
        ));
    registry
        .with(
            tracing_subscriber::fmt::layer()
                .event_format(event_format)
                .with_ansi(ansi_colors),
        )
        .with(otel_trace_layer)
        .with(otel_metrics_layer)
        .try_init()
        .map_err(crate::Error::init)?;
    Ok(())
}
