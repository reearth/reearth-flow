use std::env;

use tracing::Level;
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

pub fn setup_logging_and_tracing() -> crate::errors::Result<()> {
    let log_level = env::var("RUST_LOG")
        .ok()
        .and_then(|s| s.parse::<Level>().ok())
        .unwrap_or(Level::INFO);
    let env_filter = EnvFilter::builder()
        .with_default_directive(log_level.into())
        .from_env_lossy()
        .add_directive("opendal=error".parse().unwrap());
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
                .with_ansi(true),
        )
        .try_init()
        .map_err(crate::errors::Error::init)
}
