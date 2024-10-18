use std::error::Error;

pub type Result<T> = std::result::Result<T, WsError>;

#[derive(Debug, Clone, PartialEq)]
pub enum WsError {
    Error,
    _RoomNotFound(String),
    _JoinError(String),
}

impl Error for WsError {}

impl std::fmt::Display for WsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WsError::Error => write!(f, "WebSocket error"),
            WsError::_RoomNotFound(room_id) => write!(f, "Room not found: {}", room_id),
            WsError::_JoinError(err) => write!(f, "Failed to join room: {}", err),
        }
    }
}
