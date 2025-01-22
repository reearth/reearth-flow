use std::sync::Arc;
use tokio::sync::RwLock;

#[cfg(feature = "auth")]
pub mod auth;

pub mod broadcast;
pub mod broadcast_pool;
pub mod conf;
pub mod conn;
pub mod storage;
pub mod ws;

pub type AwarenessRef = Arc<RwLock<yrs::sync::Awareness>>;
