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

/// Gates optional OTel trace/metric export. Absent (the default) means
/// `setup_logging_and_tracing` never calls into `reearth_flow_telemetry`
/// at all, so the subscriber stack it builds is byte-identical to the
/// pre-OTel behavior — zero overhead, no extra layer, no background
/// exporter threads.
static OTEL_ENABLED: Lazy<bool> = Lazy::new(|| env::var("OTEL_COLLECTOR_ENDPOINT").is_ok());

/// Holds the OTel SDK providers alive for the process lifetime. Dropping
/// them without calling `shutdown()` silently discards any spans/metrics
/// still buffered for export — the classic OTel wiring bug. There is
/// deliberately no `Drop` impl: `main` always exits via
/// `std::process::exit`, which does not run destructors, so a `Drop` impl
/// would give a false sense of safety. Callers MUST call `shutdown()`
/// explicitly before exiting.
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

/// Initializes the OTel SDK tracer + meter providers for `service_name`
/// when `enabled`, returning the `Tracer` to bridge into the
/// `tracing_subscriber` registry plus the `OtelGuard` that owns both
/// providers. Returns `Ok(None)` without calling
/// `reearth_flow_telemetry::init_tracing`/`init_metrics` at all when
/// `enabled` is false — this is the seam that guarantees the
/// `OTEL_COLLECTOR_ENDPOINT`-absent path never touches OTel.
///
/// Takes `enabled` as a plain argument (rather than reading the memoized
/// `OTEL_ENABLED` static directly) so the gate can be exercised by a test
/// without the process-global, once-only
/// `tracing_subscriber::registry()...try_init()` call that
/// `setup_logging_and_tracing` performs around it.
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

    // Absent OTEL_COLLECTOR_ENDPOINT => otel_layer is `None`, which is a
    // no-op `Layer` impl (tracing_subscriber's `impl<L, S> Layer<S> for
    // Option<L>`), and `init_tracing`/`init_metrics` are never called —
    // the subscriber stack below is then byte-identical to before this
    // wiring existed.
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

    // `setup_logging_and_tracing` installs a process-global tracing
    // subscriber via `try_init()`, which can only succeed once per
    // process — a second call in another test would observably fail.
    // That makes the full function resistant to direct unit testing.
    // These tests instead exercise `init_otel_providers`, the pure gate
    // function it delegates to, which carries the actual "absent env var
    // => no OTel calls at all" safety property under test.

    #[test]
    fn disabled_gate_never_touches_otel() {
        // This is the binding safety property from the OTel wiring plan:
        // when disabled, `init_otel_providers` returns `Ok(None)` without
        // calling `reearth_flow_telemetry::init_tracing`/`init_metrics`,
        // so no OTel layer and no provider guard are ever produced.
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
