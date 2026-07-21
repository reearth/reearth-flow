use std::env;

use once_cell::sync::Lazy;
use opentelemetry_sdk::{
    metrics::SdkMeterProvider,
    trace::{SdkTracerProvider, Tracer},
};
use tracing::Level;
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

static ENABLE_JSON_LOG: Lazy<bool> = Lazy::new(|| {
    env::var("FLOW_CLI_ENABLE_JSON_LOG")
        .ok()
        .map(|s| s.to_lowercase() == "true")
        .unwrap_or(false)
});

/// Gates optional OTel trace/metric export; absent or empty (the default) means zero overhead —
/// no OTel calls at all. Empty/whitespace counts as unset since deployment templating often
/// leaves optional env vars blank rather than omitting them.
static OTEL_ENABLED: Lazy<bool> = Lazy::new(|| {
    env::var("OTEL_COLLECTOR_ENDPOINT")
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false)
});

/// Holds the OTel SDK providers alive for the process lifetime. No `Drop` impl on purpose —
/// `main` always exits via `std::process::exit` (skips destructors), so `shutdown()` MUST be
/// called explicitly before exiting.
pub struct OtelGuard {
    tracer_provider: SdkTracerProvider,
    meter_provider: SdkMeterProvider,
}

impl OtelGuard {
    pub fn shutdown(&self) {
        if let Err(err) = self.tracer_provider.shutdown() {
            tracing::warn!("failed to shut down OTel tracer provider: {err}");
        }
        if let Err(err) = self.meter_provider.shutdown() {
            tracing::warn!("failed to shut down OTel meter provider: {err}");
        }
    }
}

/// Initializes OTel tracer+meter providers when `enabled`; returns `Ok(None)` without touching
/// `reearth_flow_telemetry` when disabled. Takes `enabled` as a param (not reading `OTEL_ENABLED`
/// directly) so tests can exercise the gate without the process-global `try_init()` call.
fn init_otel_providers(
    enabled: bool,
    service_name: &str,
) -> crate::errors::Result<Option<(Tracer, OtelGuard)>> {
    if !enabled {
        return Ok(None);
    }
    let (tracer, tracer_provider) = reearth_flow_telemetry::init_tracing(service_name.to_string())
        .map_err(crate::errors::Error::init)?;
    let meter_provider = reearth_flow_telemetry::init_metrics(service_name.to_string())
        .map_err(crate::errors::Error::init)?;
    Ok(Some((
        tracer,
        OtelGuard {
            tracer_provider,
            meter_provider,
        },
    )))
}

pub fn setup_logging_and_tracing() -> crate::errors::Result<Option<OtelGuard>> {
    let log_level = env::var("RUST_LOG")
        .ok()
        .and_then(|s| s.parse::<Level>().ok())
        .unwrap_or(Level::INFO);
    let env_filter = EnvFilter::builder()
        .with_default_directive(log_level.into())
        .from_env_lossy();
    let time_format = UtcTime::new(
        time::format_description::parse(
            "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]Z",
        )
        .map_err(crate::errors::Error::init)?,
    );

    let registry = tracing_subscriber::registry().with(env_filter);

    // Absent OTEL_COLLECTOR_ENDPOINT => otel_layer is `None` (a no-op `Layer` impl) and
    // init_tracing/init_metrics are never called — byte-identical to pre-OTel behavior.
    let otel = init_otel_providers(*OTEL_ENABLED, env!("CARGO_PKG_NAME"))?;
    let (otel_layer, otel_guard) = match otel {
        Some((tracer, guard)) => (
            Some(tracing_opentelemetry::layer().with_tracer(tracer)),
            Some(guard),
        ),
        None => (None, None),
    };

    if *ENABLE_JSON_LOG {
        let mut layer = json_subscriber::JsonLayer::stdout();
        layer.with_flattened_event();
        layer.with_level("severity");
        layer.with_current_span("span");
        layer.with_timer("time", time_format);
        registry
            .with(otel_layer)
            .with(layer)
            .try_init()
            .map_err(crate::errors::Error::init)?;
    } else {
        let event_format = tracing_subscriber::fmt::format()
            .with_target(true)
            .with_timer(time_format);

        registry
            .with(otel_layer)
            .with(
                tracing_subscriber::fmt::layer()
                    .event_format(event_format)
                    .with_ansi(true),
            )
            .try_init()
            .map_err(crate::errors::Error::init)?;
    }

    Ok(otel_guard)
}

#[cfg(test)]
mod otel_gate_tests {
    use super::init_otel_providers;

    // `setup_logging_and_tracing` installs a process-global subscriber via `try_init()` (only
    // succeeds once per process), so these tests exercise `init_otel_providers` instead.

    #[test]
    fn disabled_gate_never_touches_otel() {
        // When disabled, `init_otel_providers` returns `Ok(None)` without calling
        // `init_tracing`/`init_metrics` — no OTel layer or guard is ever produced.
        let result = init_otel_providers(false, "reearth-flow-cli-test");
        assert!(matches!(result, Ok(None)));
    }

    #[test]
    fn enabled_gate_builds_tracer_and_guard() {
        let result = init_otel_providers(true, "reearth-flow-cli-test");
        let (_tracer, guard) = result
            .expect("init_otel_providers should not error")
            .expect("enabled gate should produce a tracer + guard");
        // Clean up the background export threads the providers spawn.
        guard.shutdown();
    }
}
