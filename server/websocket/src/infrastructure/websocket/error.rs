use std::fmt;

/// WebSocket错误类型
#[derive(Debug)]
pub struct WebSocketError(pub String);

impl fmt::Display for WebSocketError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WebSocket error: {}", self.0)
    }
}

impl std::error::Error for WebSocketError {}

impl From<axum::Error> for WebSocketError {
    fn from(err: axum::Error) -> Self {
        WebSocketError(err.to_string())
    }
}

impl From<anyhow::Error> for WebSocketError {
    fn from(err: anyhow::Error) -> Self {
        WebSocketError(err.to_string())
    }
}
