mod action;
pub mod factory;
mod split;

pub use slog::error as slog_error;
pub use slog::info as slog_info;
pub use tracing::error as tracing_error;
pub use tracing::info as tracing_info;
use tracing::info_span;

pub use slog::{o, Discard, Drain, Logger as ActionLogger};

#[macro_export]
macro_rules! action_log {
    (parent: $parent:expr, $logger:expr, $($args:tt)*) => {{
        $crate::slog_info!($logger, $($args)*);
        let parent_clone = $parent.clone();
        $crate::tracing_info!(parent: parent_clone, $($args)*); // Use the cloned parent context
    }};
}

#[macro_export]
macro_rules! action_error_log {
    (parent: $parent:expr, $logger:expr, $($args:tt)*) => {{
        $crate::slog_error!($logger, $($args)*);
        let parent_clone = $parent.clone();
        $crate::tracing_error!(parent: parent_clone, $($args)*); // Use the cloned parent context
    }};
}

pub fn span(
    parent: tracing::Span,
    action: String,
    node_id: String,
    node_name: String,
) -> tracing::Span {
    info_span!(
        parent: parent, "action",
        "otel.name" = action.to_string().as_str(),
        "otel.kind" = "action",
        "workflow.action" = format!("{:?}", action),
        "workflow.node_id" = node_id.to_string().as_str(),
        "workflow.node_name" = node_name.as_str()
    )
}
