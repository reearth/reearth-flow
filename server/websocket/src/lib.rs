use std::sync::Arc;
use tokio::sync::RwLock;

pub mod application;
pub mod config;
pub mod domain;
pub mod infrastructure;
pub mod presentation;
pub mod shared;

pub use infrastructure::redis::RedisStore;
pub use infrastructure::tracing;

pub type AwarenessRef = Arc<RwLock<yrs::sync::Awareness>>;

pub use infrastructure::websocket::BroadcastPool;
pub type WebsocketUseCase = application::usecases::websocket::WebsocketUseCase<BroadcastPool>;

pub use presentation::http::server::{ensure_bucket, start_server};

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
    pub document_usecase: Arc<DocumentUseCase>,
    pub websocket_usecase: Arc<WebsocketUseCase>,
    pub auth_usecase: Arc<application::usecases::auth::VerifyTokenUseCase>,
    pub instance_id: String,
}

#[cfg(not(feature = "auth"))]
#[derive(Clone, Debug)]
pub struct AppState {
    pub pool: Arc<BroadcastPool>,
    pub document_usecase: Arc<DocumentUseCase>,
    pub websocket_usecase: Arc<WebsocketUseCase>,
    pub instance_id: String,
}

pub use config::Config;
pub use domain::entities::doc::HistoryItem;

pub use application::usecases::document::{DocumentUseCase, DocumentUseCaseError};
pub use application::usecases::websocket::WebsocketUseCaseError;
pub use domain::value_objects::redis::{
    RedisConfig, RedisConnectionManager, RedisField, RedisFields, RedisPool, RedisStreamMessage,
    RedisStreamResult, RedisStreamResults, StreamMessages, MESSAGE_TYPE_AWARENESS,
    MESSAGE_TYPE_SYNC, OID_LOCK_KEY,
};
pub use domain::value_objects::websocket::{ConnectionCounter, ShutdownHandle, Subscription};
pub use infrastructure::gcs::GcsStore;
pub use infrastructure::websocket::{BroadcastGroup, CollaborativeStorage};
pub use presentation::http::handlers::document_handler::DocumentHandler;
pub use presentation::http::router::document_routes;
