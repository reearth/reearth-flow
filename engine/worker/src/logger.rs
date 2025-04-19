use std::{
    env,
    io::{self, Write},
    sync::RwLock,
};

use crate::errors::Error;
use crate::pubsub::backend::PubSubBackend;
use crate::pubsub::publisher::Publisher;
use crate::types::stdout_log_event::StdoutLogEvent;
use once_cell::sync::Lazy;
use parking_lot::RwLock as ParkingRwLock;
use tokio::runtime::Handle;
use tracing::field::{Field, Visit};
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

static WORKER_FILE_WRITER: Lazy<RwLock<Option<NonBlocking>>> = Lazy::new(|| RwLock::new(None));
static WORKER_FILE_GUARD: Lazy<RwLock<Option<WorkerGuard>>> = Lazy::new(|| RwLock::new(None));

static PUBSUB_PUBLISHER: Lazy<ParkingRwLock<Option<PubSubBackend>>> =
    Lazy::new(|| ParkingRwLock::new(None));
static WORKFLOW_ID: Lazy<ParkingRwLock<Option<Uuid>>> = Lazy::new(|| ParkingRwLock::new(None));
static JOB_ID: Lazy<ParkingRwLock<Option<Uuid>>> = Lazy::new(|| ParkingRwLock::new(None));
static TOKIO_RUNTIME_HANDLE: Lazy<ParkingRwLock<Option<Handle>>> =
    Lazy::new(|| ParkingRwLock::new(None));

pub fn set_pubsub_context(
    publisher: PubSubBackend,
    workflow_id: Uuid,
    job_id: Uuid,
    handle: Handle,
) {
    *PUBSUB_PUBLISHER.write() = Some(publisher);
    *WORKFLOW_ID.write() = Some(workflow_id);
    *JOB_ID.write() = Some(job_id);
    *TOKIO_RUNTIME_HANDLE.write() = Some(handle);
    tracing::info!("Pub/Sub context and Tokio handle set for stdout log publishing.");
}

#[derive(Default)]
struct MessageExtractor(Option<String>);

impl Visit for MessageExtractor {
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.0 = Some(value.to_string());
        }
    }
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            if self.0.is_none() {
                self.0 = Some(format!("{:?}", value));
            }
        }
    }
}

#[derive(Clone)]
struct StdoutLogPublishLayer;

impl<S> Layer<S> for StdoutLogPublishLayer
where
    S: Subscriber + for<'a> LookupSpan<'a> + Send + Sync,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let handle_guard = TOKIO_RUNTIME_HANDLE.read();
        let handle = match handle_guard.as_ref() {
            Some(h) => h.clone(),
            None => return,
        };
        drop(handle_guard);
        let publisher_guard = PUBSUB_PUBLISHER.read();
        if publisher_guard.is_none() {
            return;
        }
        let maybe_publisher = publisher_guard.clone();
        drop(publisher_guard);
        let workflow_id_guard = WORKFLOW_ID.read();
        let workflow_id = match workflow_id_guard.as_ref() {
            Some(id) => *id,
            None => return,
        };
        drop(workflow_id_guard);
        let job_id_guard = JOB_ID.read();
        let job_id = match job_id_guard.as_ref() {
            Some(id) => *id,
            None => return,
        };
        drop(job_id_guard);

        if let Some(publisher) = maybe_publisher {
            let meta = event.metadata();
            let level = meta.level();
            let target = meta.target();
            let event_timestamp = chrono::Utc::now();

            let timestamp_str = event_timestamp.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

            let mut extractor = MessageExtractor::default();
            event.record(&mut extractor);
            let core_message = extractor.0.unwrap_or_else(|| "".to_string());

            let message = format!(
                "{}  {} {}: {}",
                timestamp_str,
                level,
                target,
                core_message
            );

            let log_event = StdoutLogEvent {
                workflow_id,
                job_id,
                timestamp: event_timestamp,
                level: level.to_string(),
                message,
                target: target.to_string(),
            };

            handle.spawn(async move {
                let result = match publisher {
                    PubSubBackend::Google(p) => {
                        p.publish(log_event).await.map_err(|e| e.to_string())
                    }
                    PubSubBackend::Noop(p) => {
                        p.publish(log_event).await.map_err(|e| format!("{:?}", e))
                    }
                };
                if let Err(e) = result {
                    eprintln!("Failed to publish stdout log event: {}", e);
                }
            });
        }
    }
}

#[derive(Clone)]
struct DynamicFileWriter;
impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for DynamicFileWriter {
    type Writer = Box<dyn Write + Send + 'a>;

    fn make_writer(&'a self) -> Self::Writer {
        if let Some(ref nb) = *WORKER_FILE_WRITER.read().unwrap() {
            Box::new(nb.make_writer())
        } else {
            Box::new(io::sink())
        }
    }
}

pub fn setup_logging_and_tracing() -> crate::errors::Result<()> {
    let log_level = env::var("RUST_LOG")
        .ok()
        .and_then(|s| s.parse::<Level>().ok())
        .unwrap_or(Level::INFO);
    let env_filter = EnvFilter::builder()
        .with_default_directive(log_level.into())
        .from_env_lossy()
        .add_directive("opendal=error".parse().unwrap());
    let time_format = UtcTime::new(
        time::format_description::parse(
            "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]Z",
        )
        .map_err(crate::errors::Error::init)?,
    );

    let registry = tracing_subscriber::registry().with(env_filter);
    let pubsub_layer = StdoutLogPublishLayer;

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
            .with(console_layer)
            .with(file_layer)
            .with(pubsub_layer)
            .try_init()
            .map_err(Error::init)
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
            .with(console_layer)
            .with(file_layer)
            .with(pubsub_layer)
            .try_init()
            .map_err(Error::init)
    }
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

    tracing::info!("File logging enabled: {}", log_path.to_string_lossy());
    Ok(())
}
