//! Server configuration module.

/// Server-related configuration.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// WebSocket server port
    pub ws_port: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            ws_port: "8000".to_string(),
        }
    }
}
