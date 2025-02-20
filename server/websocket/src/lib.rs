use std::sync::Arc;
use tokio::sync::RwLock;

#[cfg(feature = "auth")]
pub mod auth;

mod broadcast;
pub mod conf;
pub mod conn;
pub mod grpc;
pub mod storage;
pub mod ws;

pub use broadcast::group;
pub use broadcast::pool;

pub type AwarenessRef = Arc<RwLock<yrs::sync::Awareness>>;

// Generated protobuf code
pub mod proto {
    tonic::include_proto!("proto");
}

// New modules
pub mod server;

// Types
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
}

#[cfg(not(feature = "auth"))]
#[derive(Clone, Debug)]
pub struct AppState {
    pub pool: Arc<BroadcastPool>,
}

#[cfg(feature = "auth")]
pub use auth::AuthService;

pub use conf::Config;
pub use group::BroadcastGroup;
pub use pool::BroadcastPool;
pub use server::{ensure_bucket, start_server};
pub use storage::gcs::GcsStore;
