use std::sync::Arc;
use tokio::sync::RwLock;

#[cfg(feature = "auth")]
pub mod auth;

pub mod application;
pub mod conf;
pub mod domain;
pub mod infrastructure;
pub mod interface;
pub mod tools;
pub mod ws;
pub use infrastructure::redis::RedisStore;

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
    pub document_service: Arc<DocumentService>,
    pub websocket_service: Arc<WebsocketService>,
    pub auth: Arc<AuthService>,
    pub instance_id: String,
}

#[cfg(not(feature = "auth"))]
#[derive(Clone, Debug)]
pub struct AppState {
    pub pool: Arc<BroadcastPool>,
    pub document_service: Arc<DocumentService>,
    pub websocket_service: Arc<WebsocketService>,
    pub instance_id: String,
}

#[cfg(feature = "auth")]
pub use auth::AuthService;

pub use conf::Config;
pub use domain::value_objects::conf::{
    DEFAULT_APP_ENV, DEFAULT_GCS_BUCKET, DEFAULT_ORIGINS, DEFAULT_REDIS_TTL, DEFAULT_REDIS_URL,
    DEFAULT_WS_PORT,
};
pub use domain::value_objects::http::*;
pub use domain::value_objects::sub::Subscription;

pub use application::services::broadcast_pool::BroadcastPool;
pub use application::services::document_service::{DocumentService, DocumentServiceError};
pub use application::services::websocket_service::{WebsocketService, WebsocketServiceError};
pub use domain::entity::broadcast::BroadcastGroup;
#[cfg(feature = "auth")]
pub use domain::value_objects::conf::DEFAULT_AUTH_URL;
pub use domain::value_objects::redis::{
    RedisConfig, RedisField, RedisFields, RedisPool, RedisStreamMessage, RedisStreamResult,
    RedisStreamResults, StreamMessages, MESSAGE_TYPE_AWARENESS, MESSAGE_TYPE_SYNC, OID_LOCK_KEY,
};
pub use infrastructure::gcs::GcsStore;
pub use interface::http::handlers::document_handler::DocumentHandler;
pub use interface::http::router::document_routes;
pub use server::{ensure_bucket, start_server};
