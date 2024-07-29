use std::error::Error;

pub type Result<T> = std::result::Result<T, WsError>;

#[derive(Debug, Clone, PartialEq)]
pub enum WsError {
    WsError,
}

impl Error for WsError {}

impl std::fmt::Display for WsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WebSocket error")
    }
}
