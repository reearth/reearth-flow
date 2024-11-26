use flow_websocket_infra::persistence::{
    gcs::gcs_client::GcsError, redis::errors::FlowProjectRedisDataManagerError,
};
use flow_websocket_services::SessionCommand;
use thiserror::Error;
use tokio::sync::broadcast;

#[derive(Debug, Error)]
pub enum WsError {
    #[error("Room not found: {0}")]
    RoomNotFound(String),
    #[error("Failed to join room: {0}")]
    JoinError(String),
    #[error(transparent)]
    LockError(#[from] tokio::sync::TryLockError),
    #[error(transparent)]
    BroadcastError(#[from] tokio::sync::broadcast::error::SendError<String>),
    #[error(transparent)]
    BroadcastSessionError(#[from] broadcast::error::SendError<SessionCommand>),
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error(transparent)]
    Redis(#[from] redis::RedisError),
    #[error(transparent)]
    UpdateDecode(#[from] yrs::encoding::read::Error),
    #[error(transparent)]
    AwarenessUpdate(#[from] yrs::sync::awareness::Error),
    #[error(transparent)]
    MpscSendError(#[from] tokio::sync::mpsc::error::SendError<SessionCommand>),
    #[error(transparent)]
    Pool(#[from] FlowProjectRedisDataManagerError),
    #[cfg(feature = "local-storage")]
    #[error(transparent)]
    LocalStorage(#[from] flow_websocket_infra::persistence::local_storage::LocalStorageError),
    #[cfg(feature = "local-storage")]
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[cfg(feature = "gcs-storage")]
    #[error(transparent)]
    GcsStorage(#[from] GcsError),
    #[error(transparent)]
    Room(#[from] crate::room::RoomError),
    #[error(transparent)]
    Axum(#[from] axum::Error),
    #[error(transparent)]
    SessionService(#[from] flow_websocket_services::ProjectServiceError),
}
