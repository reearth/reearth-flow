mod action_log_parser;
pub mod errors;
pub mod logger;
pub mod pubsub;
#[cfg(feature = "new-geometry")]
pub mod render_view;
pub mod types;
mod user_facing_log_handler;
pub mod wrapper;

/// The engine version this worker was built as. Every engine change bumps it,
/// so it identifies what produced an output without anyone maintaining a
/// separate renderer version.
pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");
