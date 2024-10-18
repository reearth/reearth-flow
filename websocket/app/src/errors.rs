use thiserror::Error;

pub type Result<T> = std::result::Result<T, WsError>;

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
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("WebSocket connection closed")]
    ConnectionClosed,
}
