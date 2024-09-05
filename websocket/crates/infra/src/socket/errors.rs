use std::error::Error;

pub type Result<T> = std::result::Result<T, WsError>;

#[derive(Debug, Clone, PartialEq)]
pub enum WsError {
    WsError,
    RoomNotFound(String),
    JoinError(String),
}

impl Error for WsError {}

impl std::fmt::Display for WsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WsError::WsError => write!(f, "WebSocket error"),
            WsError::RoomNotFound(room_id) => write!(f, "Room not found: {}", room_id),
            WsError::JoinError(err) => write!(f, "Failed to join room: {}", err),
        }
    }
}
