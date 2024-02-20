mod action;
pub mod factory;
mod split;

use slog::info as slog_info;
pub use slog::{o, Discard, Drain, Logger as ActionLogger};
use std::sync::Arc;
use tracing::info as tracing_info;

pub fn action_log(logger: Arc<ActionLogger>, message: &str) {
    slog_info!(logger, "{}", message);
    tracing_info!(message);
}
