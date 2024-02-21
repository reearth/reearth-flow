mod action;
pub mod factory;
mod split;

pub use slog::info as slog_info;
pub use tracing::info as tracing_info;

pub use slog::{o, Discard, Drain, Logger as ActionLogger};

#[macro_export]
macro_rules! action_log {
    ($logger:expr, $($args:tt)*) => {{
        $crate::slog_info!($logger, $($args)*);
        $crate::tracing_info!($($args)*);
    }};
}
