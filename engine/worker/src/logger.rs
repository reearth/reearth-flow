use std::{
    env,
    fs::OpenOptions,
    io::{self, Write},
    sync::RwLock,
};

use crate::errors::Error;
use once_cell::sync::Lazy;
use tracing::Level;
use tracing_appender::non_blocking::NonBlocking;
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

static ENABLE_JSON_LOG: Lazy<bool> = Lazy::new(|| {
    env::var("FLOW_WORKER_ENABLE_JSON_LOG")
        .ok()
        .map(|s| s.to_lowercase() == "true")
        .unwrap_or(false)
});

static WORKER_FILE_WRITER: Lazy<RwLock<Option<NonBlocking>>> = Lazy::new(|| RwLock::new(None));

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
    let registry = tracing_subscriber::registry();
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
            .with(env_filter)
            .with(console_layer)
            .with(file_layer)
            .try_init()
            .map_err(Error::init)
    } else {
        let event_format = tracing_subscriber::fmt::format()
            .with_target(true)
            .with_timer(time_format.clone());

        let console_layer = tracing_subscriber::fmt::layer()
            .event_format(event_format)
            .with_ansi(true)
            .with_writer(std::io::stdout);

        let file_layer = tracing_subscriber::fmt::layer()
            .with_timer(time_format)
            .with_ansi(false)
            .with_writer(DynamicFileWriter);

        registry
            .with(env_filter)
            .with(console_layer)
            .with(file_layer)
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
    let (non_blocking, _guard) = tracing_appender::non_blocking(file);

    {
        let mut writer_guard = WORKER_FILE_WRITER.write().unwrap();
        writer_guard.replace(non_blocking);
    }

    tracing::info!("File logging enabled: {}", log_path.to_string_lossy());
    Ok(())
}
