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
