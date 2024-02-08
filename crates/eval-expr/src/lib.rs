use std::sync::Arc;
use std::sync::RwLock;

pub mod engine;
pub mod error;
mod module;
mod scope;
mod utils;

pub(crate) type ShareLock<T> = Arc<RwLock<T>>;
pub type Value = serde_json::Value;
pub type Vars = serde_json::Map<String, Value>;
