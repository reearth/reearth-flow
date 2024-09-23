use std::sync::Arc;
use std::sync::RwLock;

pub mod engine;
mod error;
mod module;
pub mod scope;
pub mod utils;

pub(crate) type ShareLock<T> = Arc<RwLock<T>>;
pub type Value = serde_json::Value;
pub type Vars = serde_json::Map<String, Value>;
pub type Result<T, E = error::Error> = std::result::Result<T, E>;
