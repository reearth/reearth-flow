use thiserror::Error;
use tokio::sync::broadcast;

#[derive(Error, Debug)]
pub enum RoomError {
    #[error("User {0} already exists in the room")]
    UserAlreadyExists(String),
    #[error("User {0} not found in the room")]
    UserNotFound(String),
    #[error("Failed to broadcast message: {0}")]
    BroadcastError(#[from] broadcast::error::SendError<String>),
    #[error(transparent)]
    LockError(#[from] tokio::sync::TryLockError),
    #[error("Room not found: {0}")]
    RoomNotFound(String),
}

#[derive(Error, Debug)]
pub enum AppStateError {
    #[error(transparent)]
    Redis(#[from] redis::RedisError),
    #[error(transparent)]
    LockError(#[from] tokio::sync::TryLockError),
    #[error(transparent)]
    Gcs(#[from] flow_websocket_infra::persistence::gcs::gcs_client::GcsError),
    #[cfg(feature = "local-storage")]
    #[error(transparent)]
    LocalStorage(#[from] flow_websocket_infra::persistence::local_storage::LocalStorageError),
    #[cfg(feature = "local-storage")]
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    RedisDataManager(
        #[from] flow_websocket_infra::persistence::redis::errors::FlowProjectRedisDataManagerError,
    ),
}

#[derive(Error, Debug)]
pub enum MessageHandlerError {
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    ProjectService(#[from] flow_websocket_services::ProjectServiceError),
    #[error(transparent)]
    Axum(#[from] axum::Error),
    #[error(transparent)]
    Room(#[from] RoomError),
}
