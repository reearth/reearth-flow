use std::{
    env,
    io::{self, Write},
    sync::{Arc, RwLock},
};

use crate::errors::Error;
use crate::pubsub::backend::PubSubBackend;
pub use crate::user_facing_log_handler::{UserFacingLogHandler, UserFacingLogLayer};
use once_cell::sync::{Lazy, OnceCell};
use opentelemetry_sdk::{
    metrics::SdkMeterProvider,
    trace::{SdkTracerProvider, Tracer},
};
use tokio::runtime::Handle;
use tracing::Level;
use tracing::{Event, Subscriber};
use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::layer::Context;
use tracing_subscriber::prelude::*;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Layer;
use uuid::Uuid;

static ENABLE_JSON_LOG: Lazy<bool> = Lazy::new(|| {
    env::var("FLOW_WORKER_ENABLE_JSON_LOG")
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

static WORKER_FILE_WRITER: Lazy<RwLock<Option<NonBlocking>>> = Lazy::new(|| RwLock::new(None));
static WORKER_FILE_GUARD: Lazy<RwLock<Option<WorkerGuard>>> = Lazy::new(|| RwLock::new(None));

pub static USER_FACING_LOG_FILE_WRITER: Lazy<RwLock<Option<NonBlocking>>> =
    Lazy::new(|| RwLock::new(None));
static USER_FACING_LOG_FILE_GUARD: Lazy<RwLock<Option<WorkerGuard>>> =
    Lazy::new(|| RwLock::new(None));

static PUBSUB_PUBLISHER: OnceCell<PubSubBackend> = OnceCell::new();
static WORKFLOW_ID: OnceCell<Uuid> = OnceCell::new();
static JOB_ID: OnceCell<Uuid> = OnceCell::new();
static TOKIO_RUNTIME_HANDLE: OnceCell<Handle> = OnceCell::new();
pub static USER_FACING_LOG_HANDLER: OnceCell<Arc<UserFacingLogHandler>> = OnceCell::new();

pub fn set_pubsub_context(
    publisher: PubSubBackend,
    workflow_id: Uuid,
    job_id: Uuid,
    handle: Handle,
) -> Result<(), &'static str> {
    PUBSUB_PUBLISHER
        .set(publisher.clone())
        .map_err(|_| "PubSub context already initialized")?;
    WORKFLOW_ID
        .set(workflow_id)
        .map_err(|_| "Workflow ID already initialized")?;
    JOB_ID
        .set(job_id)
        .map_err(|_| "Job ID already initialized")?;
    TOKIO_RUNTIME_HANDLE
        .set(handle.clone())
        .map_err(|_| "Tokio handle already initialized")?;

    tracing::info!("Pub/Sub context and Tokio handle set for stdout log publishing.");

    // Initialize user-facing log handler
    let handler = Arc::new(UserFacingLogHandler::new(
        workflow_id,
        job_id,
        publisher,
        handle,
    ));
    USER_FACING_LOG_HANDLER
        .set(handler)
        .map_err(|_| "User-facing log handler already initialized")?;
    tracing::info!("User-facing log handler initialized");

    Ok(())
}

#[derive(Clone)]
struct GlobalUserFacingLogLayer;

impl<S> Layer<S> for GlobalUserFacingLogLayer
where
    S: Subscriber + for<'a> LookupSpan<'a> + Send + Sync,
{
    fn on_new_span(
        &self,
        attrs: &tracing::span::Attributes<'_>,
        id: &tracing::span::Id,
        ctx: Context<'_, S>,
    ) {
        if let Some(handler) = USER_FACING_LOG_HANDLER.get() {
            let layer = UserFacingLogLayer::new(handler.clone());
            layer.on_new_span(attrs, id, ctx);
        }
    }

    fn on_close(&self, id: tracing::span::Id, ctx: Context<'_, S>) {
        if let Some(handler) = USER_FACING_LOG_HANDLER.get() {
            let layer = UserFacingLogLayer::new(handler.clone());
            layer.on_close(id, ctx);
        }
    }

    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        if let Some(handler) = USER_FACING_LOG_HANDLER.get() {
            let layer = UserFacingLogLayer::new(handler.clone());
            layer.on_event(event, ctx);
        }
    }
}

#[derive(Clone)]
struct DynamicFileWriter;
impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for DynamicFileWriter {
    type Writer = Box<dyn Write + Send + 'a>;

    fn make_writer(&'a self) -> Self::Writer {
        let guard = WORKER_FILE_WRITER.read().unwrap();
        if let Some(nb) = guard.as_ref() {
            Box::new(nb.make_writer())
        } else {
            Box::new(io::sink())
        }
    }
}

#[derive(Clone)]
pub struct DynamicUserFacingLogFileWriter;
impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for DynamicUserFacingLogFileWriter {
    type Writer = Box<dyn Write + Send + 'a>;

    fn make_writer(&'a self) -> Self::Writer {
        let guard = USER_FACING_LOG_FILE_WRITER.read().unwrap();
        if let Some(nb) = guard.as_ref() {
            Box::new(nb.make_writer())
        } else {
            Box::new(io::sink())
        }
    }
}

/// Holds the OTel SDK providers alive for the process lifetime. No `Drop` impl on purpose —
/// every caller here exits via `std::process::exit` (skips destructors), so `shutdown()` MUST
/// be called explicitly before exiting.
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
    let (tracer, tracer_provider) =
        reearth_flow_telemetry::init_tracing(service_name.to_string()).map_err(Error::init)?;
    let meter_provider =
        reearth_flow_telemetry::init_metrics(service_name.to_string()).map_err(Error::init)?;
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
    let user_facing_layer = GlobalUserFacingLogLayer;

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
        let mut console_layer = json_subscriber::JsonLayer::stdout();
        console_layer.with_flattened_event();
        console_layer.with_level("severity");
        console_layer.with_current_span("span");
        console_layer.with_timer("time", time_format.clone());

        let mut file_layer = json_subscriber::JsonLayer::new(DynamicFileWriter);
        file_layer.with_flattened_event();
        file_layer.with_level("severity");
        file_layer.with_current_span("span");
        file_layer.with_timer("time", time_format);

        registry
            .with(otel_layer)
            .with(console_layer)
            .with(file_layer)
            .with(user_facing_layer)
            .try_init()
            .map_err(Error::init)?;
    } else {
        let file_event_format = tracing_subscriber::fmt::format()
            .with_target(true)
            .with_timer(time_format.clone())
            .with_ansi(false);

        let console_layer = tracing_subscriber::fmt::layer()
            .event_format(file_event_format.clone())
            .with_ansi(true)
            .with_writer(std::io::stdout);

        let file_layer = tracing_subscriber::fmt::layer()
            .event_format(file_event_format.clone())
            .with_writer(DynamicFileWriter);

        registry
            .with(otel_layer)
            .with(console_layer)
            .with(file_layer)
            .with(user_facing_layer)
            .try_init()
            .map_err(Error::init)?;
    }

    Ok(otel_guard)
}

pub fn enable_file_logging(job_id: uuid::Uuid) -> crate::errors::Result<()> {
    let worker_uri = reearth_flow_common::dir::setup_job_directory("workers", "worker", job_id)
        .map_err(Error::init)?;
    let path_ref = worker_uri.path();
    let worker_path = path_ref.as_path();
    std::fs::create_dir_all(worker_path)
        .map_err(|e| Error::init(format!("Failed to create worker dir: {e}")))?;
    let log_path = worker_path.join("worker.log");
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(Error::init)?;
    let (non_blocking, guard) = tracing_appender::non_blocking(file);

    {
        let mut writer_guard = WORKER_FILE_WRITER.write().unwrap();
        writer_guard.replace(non_blocking);
    }
    {
        let mut guard_lock = WORKER_FILE_GUARD.write().unwrap();
        guard_lock.replace(guard);
    }

    // Also create user-facing log file
    enable_user_facing_log_file(job_id)?;

    tracing::info!("File logging enabled: {}", log_path.to_string_lossy());
    Ok(())
}

pub fn enable_user_facing_log_file(job_id: uuid::Uuid) -> crate::errors::Result<()> {
    let log_uri =
        reearth_flow_common::dir::setup_job_directory("workers", "user-facing-log", job_id)
            .map_err(Error::init)?;
    let path_ref = log_uri.path();
    let log_path = path_ref.as_path();
    std::fs::create_dir_all(log_path)
        .map_err(|e| Error::init(format!("Failed to create user-facing-log dir: {e}")))?;

    let log_file_path = log_path.join("user-facing.log");
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file_path)
        .map_err(Error::init)?;

    let (non_blocking, guard) = tracing_appender::non_blocking(file);

    {
        let mut writer_guard = USER_FACING_LOG_FILE_WRITER.write().unwrap();
        writer_guard.replace(non_blocking);
    }
    {
        let mut guard_lock = USER_FACING_LOG_FILE_GUARD.write().unwrap();
        guard_lock.replace(guard);
    }

    tracing::info!(
        "User-facing log file enabled: {}",
        log_file_path.to_string_lossy()
    );
    Ok(())
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
        let result = init_otel_providers(false, "reearth-flow-worker-test");
        assert!(matches!(result, Ok(None)));
    }

    #[test]
    fn enabled_gate_builds_tracer_and_guard() {
        let result = init_otel_providers(true, "reearth-flow-worker-test");
        let (_tracer, guard) = result
            .expect("init_otel_providers should not error")
            .expect("enabled gate should produce a tracer + guard");
        // Clean up the background export threads the providers spawn.
        guard.shutdown();
    }
}
