pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod interface;

pub use domain::{
    BroadcastMessage, ConnectionId, ConnectionInfo, Document, DocumentId, MessageType,
};

pub use application::{
    AppState, BroadcastService, Config, ConfigService, DocumentAppService, DocumentService,
    WebSocketService,
};

pub use infrastructure::{BroadcastGroup, BroadcastPool, Connection, GcsStore, RedisStore};

pub use interface::{create_ws_router, document_routes, start_server};

pub type AwarenessRef = std::sync::Arc<tokio::sync::RwLock<yrs::sync::Awareness>>;
