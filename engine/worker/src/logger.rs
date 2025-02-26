use std::env;

use once_cell::sync::Lazy;
use tracing::Level;
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

static ENABLE_JSON_LOG: Lazy<bool> = Lazy::new(|| {
    env::var("FLOW_WORKER_ENABLE_JSON_LOG")
        .ok()
        .map(|s| s.to_lowercase() == "true")
        .unwrap_or(false)
});

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
    if *ENABLE_JSON_LOG {
        let mut layer = json_subscriber::JsonLayer::stdout();
        layer.with_flattened_event();
        layer.with_level("severity");
        layer.with_current_span("span");
        layer.with_timer("time", time_format);
        tracing_subscriber::registry()
            .with(env_filter)
            .with(layer)
            .try_init()
            .map_err(crate::errors::Error::init)
    } else {
        let event_format = tracing_subscriber::fmt::format()
            .with_target(true)
            .with_timer(time_format);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .event_format(event_format)
                    .with_ansi(true),
            )
            .try_init()
            .map_err(crate::errors::Error::init)
    }
}
