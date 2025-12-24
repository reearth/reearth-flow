//! Google Cloud Storage configuration module.

/// GCS-related configuration.
#[derive(Debug, Clone)]
pub struct GcsConfig {
    /// GCS bucket name
    pub bucket_name: String,
    /// Optional endpoint override (for local development with fake-gcs-server)
    pub endpoint: Option<String>,
}

impl Default for GcsConfig {
    fn default() -> Self {
        Self {
            bucket_name: "yrs-dev".to_string(),
            endpoint: None,
        }
    }
}

/// Convert to infrastructure GcsConfig for use with GcsStore
impl From<GcsConfig> for crate::infrastructure::gcs::GcsConfig {
    fn from(config: GcsConfig) -> Self {
        Self {
            bucket_name: config.bucket_name,
            endpoint: config.endpoint,
        }
    }
}

impl From<&GcsConfig> for crate::infrastructure::gcs::GcsConfig {
    fn from(config: &GcsConfig) -> Self {
        Self {
            bucket_name: config.bucket_name.clone(),
            endpoint: config.endpoint.clone(),
        }
    }
}
