//! Layered configuration module for the websocket server.
//!
//! This module provides a hierarchical configuration system that loads settings
//! from environment variables with the following priority (highest to lowest):
//!
//! 1. Environment variables (REEARTH_FLOW_*)
//! 2. .env file (if present)
//! 3. Default values
//!
//! # Configuration Structure
//!
//! ```text
//! src/config/
//! ├── mod.rs        # Main config module and loader
//! ├── server.rs     # Server configuration (ports)
//! ├── app.rs        # Application configuration (env, origins)
//! ├── redis.rs      # Redis configuration
//! ├── gcs.rs        # Google Cloud Storage configuration
//! ├── auth.rs       # Authentication configuration
//! └── tracing.rs    # Tracing and telemetry configuration
//! ```
//!
//! # Example
//!
//! ```rust,no_run
//! use websocket::config::Config;
//!
//! let config = Config::load().expect("Failed to load config");
//! println!("Server port: {}", config.server.ws_port);
//! ```

mod app;
#[cfg(feature = "auth")]
mod auth;
mod gcs;
mod redis;
mod server;
mod tracing;

pub use app::AppConfig;
#[cfg(feature = "auth")]
pub use auth::AuthConfig;
pub use gcs::GcsConfig;
pub use redis::RedisConfig;
pub use server::ServerConfig;
pub use tracing::TracingConfig;

use ::tracing::{info, warn};
use std::env;
use std::path::Path;
use thiserror::Error;

/// Configuration loading error.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Environment variable error: {0}")]
    EnvVar(#[from] env::VarError),
    #[error("Invalid value for {key}: {value}")]
    InvalidValue { key: String, value: String },
}

/// Main configuration struct that aggregates all configuration sections.
#[derive(Debug, Clone, Default)]
pub struct Config {
    /// Server configuration (ports, etc.)
    pub server: ServerConfig,
    /// Application configuration (environment, origins, etc.)
    pub app: AppConfig,
    /// Redis configuration
    pub redis: RedisConfig,
    /// Google Cloud Storage configuration
    pub gcs: GcsConfig,
    /// Authentication configuration
    #[cfg(feature = "auth")]
    pub auth: AuthConfig,
    /// Tracing and telemetry configuration
    pub tracing: TracingConfig,
}

impl Config {
    /// Create a new ConfigBuilder for building configuration.
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }

    /// Load configuration from environment variables.
    ///
    /// Configuration is loaded in the following order:
    /// 1. Default values are set
    /// 2. .env file is loaded (if present)
    /// 3. Environment variables override defaults
    ///
    /// # Environment Variables
    ///
    /// ## Server
    /// - `REEARTH_FLOW_WS_PORT`: WebSocket server port (default: "8000")
    ///
    /// ## Application
    /// - `REEARTH_FLOW_APP_ENV`: Environment name (default: "development")
    /// - `REEARTH_FLOW_ORIGINS`: Comma-separated list of allowed origins
    ///
    /// ## Redis
    /// - `REEARTH_FLOW_REDIS_URL`: Redis connection URL
    /// - `REEARTH_FLOW_REDIS_STREAM_TRIM_INTERVAL`: Stream trim interval in seconds
    /// - `REEARTH_FLOW_REDIS_STREAM_MAX_MESSAGE_AGE`: Max message age in milliseconds
    /// - `REEARTH_FLOW_REDIS_STREAM_MAX_LENGTH`: Max stream length
    ///
    /// ## GCS
    /// - `REEARTH_FLOW_GCS_BUCKET_NAME`: GCS bucket name
    /// - `REEARTH_FLOW_GCS_ENDPOINT`: Optional GCS endpoint override
    ///
    /// ## Auth (when auth feature enabled)
    /// - `REEARTH_FLOW_THRIFT_AUTH_URL`: Authentication service URL
    ///
    /// ## Tracing
    /// - `REEARTH_FLOW_ENABLE_CLOUD_TRACE`: Enable Google Cloud Trace (true/false)
    /// - `REEARTH_FLOW_ENABLE_OTLP`: Enable OTLP export (true/false)
    /// - `REEARTH_FLOW_OTLP_ENDPOINT`: OTLP endpoint URL
    /// - `REEARTH_FLOW_GCP_PROJECT_ID`: GCP project ID
    /// - `REEARTH_FLOW_SERVICE_NAME`: Service name for tracing
    /// - `REEARTH_FLOW_LOG_LEVEL`: Log level (trace, debug, info, warn, error)
    pub fn load() -> Result<Self, ConfigError> {
        let mut builder = Self::builder();

        // Load .env file if present
        if dotenv::from_path(Path::new(".env")).is_ok() {
            info!("Loaded configuration from .env file");
        } else {
            warn!("No .env file found, using environment variables");
        }

        // Server configuration
        if let Ok(ws_port) = env::var("REEARTH_FLOW_WS_PORT") {
            builder = builder.ws_port(ws_port);
        }

        // Application configuration
        if let Ok(env_val) = env::var("REEARTH_FLOW_APP_ENV") {
            builder = builder.app_env(env_val);
        }
        if let Ok(origins) = env::var("REEARTH_FLOW_ORIGINS") {
            let origins_vec: Vec<String> = origins
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            builder = builder.app_origins(origins_vec);
        }

        // Redis configuration
        if let Ok(url) = env::var("REEARTH_FLOW_REDIS_URL") {
            builder = builder.redis_url(url);
        }
        if let Ok(interval) = env::var("REEARTH_FLOW_REDIS_STREAM_TRIM_INTERVAL") {
            if let Ok(interval_secs) = interval.parse::<u64>() {
                builder = builder.redis_stream_trim_interval(interval_secs);
            }
        }
        if let Ok(max_age) = env::var("REEARTH_FLOW_REDIS_STREAM_MAX_MESSAGE_AGE") {
            if let Ok(max_age_ms) = max_age.parse::<u64>() {
                builder = builder.redis_stream_max_message_age(max_age_ms);
            }
        }
        if let Ok(max_length) = env::var("REEARTH_FLOW_REDIS_STREAM_MAX_LENGTH") {
            if let Ok(max_len) = max_length.parse::<u64>() {
                builder = builder.redis_stream_max_length(max_len);
            }
        }

        // GCS configuration
        if let Ok(bucket) = env::var("REEARTH_FLOW_GCS_BUCKET_NAME") {
            builder = builder.gcs_bucket(bucket);
        }
        if let Ok(endpoint) = env::var("REEARTH_FLOW_GCS_ENDPOINT") {
            builder = builder.gcs_endpoint(Some(endpoint));
        }

        // Auth configuration
        #[cfg(feature = "auth")]
        if let Ok(url) = env::var("REEARTH_FLOW_THRIFT_AUTH_URL") {
            builder = builder.auth_url(url);
        }

        // Tracing configuration
        if let Ok(enable) = env::var("REEARTH_FLOW_ENABLE_CLOUD_TRACE") {
            let enabled = enable.to_lowercase() == "true" || enable == "1";
            builder = builder.enable_cloud_trace(enabled);
        }
        if let Ok(enable) = env::var("REEARTH_FLOW_ENABLE_OTLP") {
            let enabled = enable.to_lowercase() == "true" || enable == "1";
            builder = builder.enable_otlp(enabled);
        }
        if let Ok(endpoint) = env::var("REEARTH_FLOW_OTLP_ENDPOINT") {
            builder = builder.otlp_endpoint(endpoint);
        }
        if let Ok(project_id) = env::var("REEARTH_FLOW_GCP_PROJECT_ID") {
            builder = builder.gcp_project_id(Some(project_id));
        }
        if let Ok(service_name) = env::var("REEARTH_FLOW_SERVICE_NAME") {
            builder = builder.service_name(service_name);
        }
        if let Ok(log_level) = env::var("REEARTH_FLOW_LOG_LEVEL") {
            builder = builder.log_level(log_level);
        }

        let config = builder.build();

        info!("Configuration loaded successfully");
        info!("Server: {:?}", config.server);
        info!("App: {:?}", config.app);
        info!("Redis: {:?}", config.redis);
        info!("GCS: {:?}", config.gcs);
        info!("Tracing: {:?}", config.tracing);
        #[cfg(feature = "auth")]
        info!("Auth: {:?}", config.auth);

        Ok(config)
    }

    /// Get the WebSocket port.
    pub fn ws_port(&self) -> &str {
        &self.server.ws_port
    }
}

/// Builder for constructing Config with custom values.
#[derive(Default)]
pub struct ConfigBuilder {
    // Server
    ws_port: Option<String>,
    // App
    app_env: Option<String>,
    app_origins: Option<Vec<String>>,
    // Redis
    redis_url: Option<String>,
    redis_ttl: Option<u64>,
    redis_stream_trim_interval: Option<u64>,
    redis_stream_max_message_age: Option<u64>,
    redis_stream_max_length: Option<u64>,
    // GCS
    gcs_bucket: Option<String>,
    gcs_endpoint: Option<String>,
    // Auth
    #[cfg(feature = "auth")]
    auth_url: Option<String>,
    // Tracing
    enable_cloud_trace: Option<bool>,
    enable_otlp: Option<bool>,
    otlp_endpoint: Option<String>,
    gcp_project_id: Option<String>,
    service_name: Option<String>,
    log_level: Option<String>,
}

impl ConfigBuilder {
    // Server configuration
    pub fn ws_port(mut self, port: String) -> Self {
        self.ws_port = Some(port);
        self
    }

    // App configuration
    pub fn app_env(mut self, env: String) -> Self {
        self.app_env = Some(env);
        self
    }

    pub fn app_origins(mut self, origins: Vec<String>) -> Self {
        self.app_origins = Some(origins);
        self
    }

    // Redis configuration
    pub fn redis_url(mut self, url: String) -> Self {
        self.redis_url = Some(url);
        self
    }

    pub fn redis_ttl(mut self, ttl: u64) -> Self {
        self.redis_ttl = Some(ttl);
        self
    }

    pub fn redis_stream_trim_interval(mut self, interval: u64) -> Self {
        self.redis_stream_trim_interval = Some(interval);
        self
    }

    pub fn redis_stream_max_message_age(mut self, max_age: u64) -> Self {
        self.redis_stream_max_message_age = Some(max_age);
        self
    }

    pub fn redis_stream_max_length(mut self, max_length: u64) -> Self {
        self.redis_stream_max_length = Some(max_length);
        self
    }

    // GCS configuration
    pub fn gcs_bucket(mut self, bucket: String) -> Self {
        self.gcs_bucket = Some(bucket);
        self
    }

    pub fn gcs_endpoint(mut self, endpoint: Option<String>) -> Self {
        self.gcs_endpoint = endpoint;
        self
    }

    // Auth configuration
    #[cfg(feature = "auth")]
    pub fn auth_url(mut self, url: String) -> Self {
        self.auth_url = Some(url);
        self
    }

    // Tracing configuration
    pub fn enable_cloud_trace(mut self, enable: bool) -> Self {
        self.enable_cloud_trace = Some(enable);
        self
    }

    pub fn enable_otlp(mut self, enable: bool) -> Self {
        self.enable_otlp = Some(enable);
        self
    }

    pub fn otlp_endpoint(mut self, endpoint: String) -> Self {
        self.otlp_endpoint = Some(endpoint);
        self
    }

    pub fn gcp_project_id(mut self, project_id: Option<String>) -> Self {
        self.gcp_project_id = project_id;
        self
    }

    pub fn service_name(mut self, name: String) -> Self {
        self.service_name = Some(name);
        self
    }

    pub fn log_level(mut self, level: String) -> Self {
        self.log_level = Some(level);
        self
    }

    /// Build the final Config struct.
    pub fn build(self) -> Config {
        Config {
            server: ServerConfig {
                ws_port: self
                    .ws_port
                    .unwrap_or_else(|| ServerConfig::default().ws_port),
            },
            app: AppConfig {
                env: self.app_env.unwrap_or_else(|| AppConfig::default().env),
                origins: self
                    .app_origins
                    .unwrap_or_else(|| AppConfig::default().origins),
            },
            redis: RedisConfig {
                url: self.redis_url.unwrap_or_else(|| RedisConfig::default().url),
                ttl: self.redis_ttl.unwrap_or_else(|| RedisConfig::default().ttl),
                stream_trim_interval: self
                    .redis_stream_trim_interval
                    .unwrap_or_else(|| RedisConfig::default().stream_trim_interval),
                stream_max_message_age: self
                    .redis_stream_max_message_age
                    .unwrap_or_else(|| RedisConfig::default().stream_max_message_age),
                stream_max_length: self
                    .redis_stream_max_length
                    .unwrap_or_else(|| RedisConfig::default().stream_max_length),
            },
            gcs: GcsConfig {
                bucket_name: self
                    .gcs_bucket
                    .unwrap_or_else(|| GcsConfig::default().bucket_name),
                endpoint: self.gcs_endpoint.filter(|e| !e.is_empty()),
            },
            #[cfg(feature = "auth")]
            auth: AuthConfig {
                url: self.auth_url.unwrap_or_else(|| AuthConfig::default().url),
            },
            tracing: TracingConfig {
                enable_cloud_trace: self
                    .enable_cloud_trace
                    .unwrap_or_else(|| TracingConfig::default().enable_cloud_trace),
                enable_otlp: self
                    .enable_otlp
                    .unwrap_or_else(|| TracingConfig::default().enable_otlp),
                otlp_endpoint: self
                    .otlp_endpoint
                    .unwrap_or_else(|| TracingConfig::default().otlp_endpoint),
                gcp_project_id: self
                    .gcp_project_id
                    .or_else(|| TracingConfig::default().gcp_project_id),
                service_name: self
                    .service_name
                    .unwrap_or_else(|| TracingConfig::default().service_name),
                log_level: self
                    .log_level
                    .unwrap_or_else(|| TracingConfig::default().log_level),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.server.ws_port, "8000");
        assert_eq!(config.app.env, "development");
        assert_eq!(config.redis.url, "redis://127.0.0.1:6379");
    }

    #[test]
    fn test_config_builder() {
        let config = Config::builder()
            .ws_port("9000".to_string())
            .app_env("production".to_string())
            .redis_url("redis://localhost:6380".to_string())
            .build();

        assert_eq!(config.server.ws_port, "9000");
        assert_eq!(config.app.env, "production");
        assert_eq!(config.redis.url, "redis://localhost:6380");
    }
}
