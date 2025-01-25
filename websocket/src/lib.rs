use std::sync::Arc;
use tokio::sync::RwLock;

#[cfg(feature = "auth")]
pub mod auth;

mod broadcast;
pub mod conf;
pub mod conn;
pub mod storage;
pub mod ws;

pub use broadcast::group;
pub use broadcast::pool;

pub type AwarenessRef = Arc<RwLock<yrs::sync::Awareness>>;
