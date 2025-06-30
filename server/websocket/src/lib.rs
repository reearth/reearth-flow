use std::sync::Arc;
use tokio::sync::RwLock;

#[cfg(feature = "auth")]
pub mod infrastructure;

pub mod auth;
pub mod domain;

mod broadcast;
pub mod conn;
pub mod doc;
pub mod tools;
pub mod ws;

pub use broadcast::group;
pub use broadcast::pool;

pub type AwarenessRef = Arc<RwLock<yrs::sync::Awareness>>;

pub mod server;

#[cfg(feature = "auth")]
#[derive(Debug, serde::Deserialize)]
pub struct AuthQuery {
    #[serde(default)]
    pub token: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct RollbackQuery {
    pub clock: u32,
    #[cfg(feature = "auth")]
    #[serde(default)]
    pub token: String,
}

#[cfg(feature = "auth")]
#[derive(Clone, Debug)]
pub struct AppState {
    pub pool: Arc<BroadcastPool>,
    pub auth: Arc<AuthService>,
    pub instance_id: String,
}

#[cfg(not(feature = "auth"))]
#[derive(Clone, Debug)]
pub struct AppState {
    pub pool: Arc<BroadcastPool>,
    pub instance_id: String,
}

#[cfg(feature = "auth")]
pub use auth::AuthService;

pub use broadcast::sub::Subscription;
pub use group::BroadcastGroup;
pub use infrastructure::config::AppConfig;
pub use infrastructure::persistence::gcs::GcsStore;
pub use infrastructure::persistence::kv::DocOps;
pub use pool::BroadcastPool;
pub use server::{ensure_bucket, start_server};
