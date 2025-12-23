//! Authentication configuration module.

/// Authentication-related configuration.
#[cfg(feature = "auth")]
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// Authentication service URL
    pub url: String,
}

#[cfg(feature = "auth")]
impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:8080".to_string(),
        }
    }
}

