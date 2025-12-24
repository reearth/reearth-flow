//! Application configuration module.

/// Application-related configuration.
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Application environment (development, production, staging)
    pub env: String,
    /// Allowed CORS origins
    pub origins: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            env: "development".to_string(),
            origins: vec![
                "http://localhost:3000".to_string(),
                "https://api.flow.test.reearth.dev".to_string(),
                "https://api.flow.reearth.dev".to_string(),
                "http://localhost:8000".to_string(),
                "http://localhost:8080".to_string(),
            ],
        }
    }
}
