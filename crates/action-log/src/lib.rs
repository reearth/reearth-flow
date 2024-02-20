mod action;
pub mod factory;
mod split;

pub use slog::{info as action_log, o, Discard, Drain, Logger};
