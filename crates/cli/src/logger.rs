use anyhow::Context;
use tracing::Level;
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

pub fn setup_logging_and_tracing(level: Level, ansi_colors: bool) -> anyhow::Result<()> {
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
        .try_init()
        .context("Failed to set up tracing.")?;
    Ok(())
}
