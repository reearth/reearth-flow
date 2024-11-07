use std::env;

use once_cell::sync::Lazy;
use tracing::Level;
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

static ENABLE_JSON_LOG: Lazy<bool> = Lazy::new(|| {
    env::var("FLOW_WORKER_ENABLE_JSON_LOG")
        .ok()
        .map(|s| s == "true")
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
    if *ENABLE_JSON_LOG {
        tracing_subscriber::fmt()
            .json()
            .with_ansi(false)
            .with_target(false)
            .with_env_filter(env_filter)
            .try_init()
            .map_err(crate::errors::Error::init)
    } else {
        let event_format = tracing_subscriber::fmt::format()
            .with_target(true)
            .with_timer(UtcTime::new(
                time::format_description::parse(
                    "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]Z",
                )
                .expect("Time format invalid."),
            ));

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
