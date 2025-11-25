#[cfg(feature = "auth")]
use crate::domain::value_objects::conf::DEFAULT_AUTH_URL;
use crate::domain::value_objects::conf::{
    DEFAULT_APP_ENV, DEFAULT_ENABLE_CLOUD_TRACE, DEFAULT_GCS_BUCKET, DEFAULT_LOG_LEVEL,
    DEFAULT_ORIGINS, DEFAULT_REDIS_STREAM_MAX_LENGTH, DEFAULT_REDIS_STREAM_MAX_MESSAGE_AGE,
    DEFAULT_REDIS_STREAM_TRIM_INTERVAL, DEFAULT_REDIS_TTL, DEFAULT_REDIS_URL, DEFAULT_SERVICE_NAME,
    DEFAULT_WS_PORT,
};
use dotenv;
use serde::Deserialize;
use std::env;
use std::path::Path;
use thiserror::Error;
use tracing::{info, warn};

use crate::infrastructure::tracing::TracingConfig;
use crate::{infrastructure::gcs::GcsConfig, RedisConfig};

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Environment variable error: {0}")]
    EnvVar(#[from] env::VarError),
    #[error("Invalid value for {key}: {value}")]
    InvalidValue { key: String, value: String },
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub env: String,
    pub origins: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub redis: RedisConfig,
    pub gcs: GcsConfig,
    #[cfg(feature = "auth")]
    pub auth: AuthConfig,
    pub app: AppConfig,
    pub ws_port: String,
    pub tracing: TracingConfig,
}

impl Config {
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }

    pub fn load() -> Result<Self, ConfigError> {
        let mut builder = Self::builder();

        if dotenv::from_path(Path::new(".env")).is_ok() {
            info!("Loaded configuration from .env file");
        } else {
            warn!("No .env file found, using environment variables");
        }

        if let Ok(url) = env::var("REEARTH_FLOW_REDIS_URL") {
            builder = builder.redis_url(url);
        }
        if let Ok(bucket) = env::var("REEARTH_FLOW_GCS_BUCKET_NAME") {
            builder = builder.gcs_bucket(bucket);
        }
        if let Ok(endpoint) = env::var("REEARTH_FLOW_GCS_ENDPOINT") {
            builder = builder.gcs_endpoint(Some(endpoint));
        }
        #[cfg(feature = "auth")]
        if let Ok(url) = env::var("REEARTH_FLOW_THRIFT_AUTH_URL") {
            builder = builder.auth_url(url);
        }

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

        if let Ok(ws_port) = env::var("REEARTH_FLOW_WS_PORT") {
            builder = builder.ws_port(ws_port);
        }
        if let Ok(grpc_port) = env::var("REEARTH_FLOW_GRPC_PORT") {
            builder = builder.grpc_port(grpc_port);
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

        // Tracing configuration
        if let Ok(enable) = env::var("REEARTH_FLOW_ENABLE_CLOUD_TRACE") {
            let enabled = enable.to_lowercase() == "true" || enable == "1";
            builder = builder.enable_cloud_trace(enabled);
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

        info!("Final configuration:");
        info!("Redis: {:?}", config.redis);
        info!("GCS: {:?}", config.gcs);
        info!("App: {:?}", config.app);
        info!("WebSocket Port: {}", config.ws_port);
        info!("Tracing: {:?}", config.tracing);
        #[cfg(feature = "auth")]
        info!("Auth: {:?}", config.auth);

        Ok(config)
    }
}

#[derive(Default)]
pub struct ConfigBuilder {
    redis_url: Option<String>,
    redis_ttl: Option<u64>,
    redis_stream_trim_interval: Option<u64>,
    redis_stream_max_message_age: Option<u64>,
    redis_stream_max_length: Option<u64>,
    gcs_bucket: Option<String>,
    gcs_endpoint: Option<String>,
    #[cfg(feature = "auth")]
    auth_url: Option<String>,
    #[cfg(feature = "auth")]
    auth_timeout: Option<u64>,
    app_env: Option<String>,
    app_origins: Option<Vec<String>>,
    ws_port: Option<String>,
    grpc_port: Option<String>,
    // Tracing configuration
    enable_cloud_trace: Option<bool>,
    gcp_project_id: Option<String>,
    service_name: Option<String>,
    log_level: Option<String>,
}

impl ConfigBuilder {
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

    pub fn gcs_bucket(mut self, bucket: String) -> Self {
        self.gcs_bucket = Some(bucket);
        self
    }

    pub fn gcs_endpoint(mut self, endpoint: Option<String>) -> Self {
        self.gcs_endpoint = endpoint;
        self
    }

    #[cfg(feature = "auth")]
    pub fn auth_url(mut self, url: String) -> Self {
        self.auth_url = Some(url);
        self
    }

    #[cfg(feature = "auth")]
    pub fn auth_timeout(mut self, timeout: u64) -> Self {
        self.auth_timeout = Some(timeout);
        self
    }

    pub fn app_env(mut self, env: String) -> Self {
        self.app_env = Some(env);
        self
    }

    pub fn app_origins(mut self, origins: Vec<String>) -> Self {
        self.app_origins = Some(origins);
        self
    }

    pub fn ws_port(mut self, port: String) -> Self {
        self.ws_port = Some(port);
        self
    }

    pub fn grpc_port(mut self, port: String) -> Self {
        self.grpc_port = Some(port);
        self
    }

    pub fn enable_cloud_trace(mut self, enable: bool) -> Self {
        self.enable_cloud_trace = Some(enable);
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

    pub fn build(self) -> Config {
        Config {
            redis: RedisConfig {
                url: self
                    .redis_url
                    .unwrap_or_else(|| DEFAULT_REDIS_URL.to_string()),
                ttl: self.redis_ttl.unwrap_or(DEFAULT_REDIS_TTL),
                stream_trim_interval: self
                    .redis_stream_trim_interval
                    .unwrap_or(DEFAULT_REDIS_STREAM_TRIM_INTERVAL),
                stream_max_message_age: self
                    .redis_stream_max_message_age
                    .unwrap_or(DEFAULT_REDIS_STREAM_MAX_MESSAGE_AGE),
                stream_max_length: self
                    .redis_stream_max_length
                    .unwrap_or(DEFAULT_REDIS_STREAM_MAX_LENGTH),
            },
            gcs: GcsConfig {
                bucket_name: self
                    .gcs_bucket
                    .unwrap_or_else(|| DEFAULT_GCS_BUCKET.to_string()),
                endpoint: self.gcs_endpoint.filter(|e| !e.is_empty()),
            },
            #[cfg(feature = "auth")]
            auth: AuthConfig {
                url: self
                    .auth_url
                    .unwrap_or_else(|| DEFAULT_AUTH_URL.to_string()),
            },
            app: AppConfig {
                env: self.app_env.unwrap_or_else(|| DEFAULT_APP_ENV.to_string()),
                origins: self
                    .app_origins
                    .unwrap_or_else(|| DEFAULT_ORIGINS.iter().map(|s| s.to_string()).collect()),
            },
            ws_port: self.ws_port.unwrap_or_else(|| DEFAULT_WS_PORT.to_string()),
            tracing: TracingConfig {
                enable_cloud_trace: self
                    .enable_cloud_trace
                    .unwrap_or(DEFAULT_ENABLE_CLOUD_TRACE),
                gcp_project_id: self.gcp_project_id,
                service_name: self
                    .service_name
                    .unwrap_or_else(|| DEFAULT_SERVICE_NAME.to_string()),
                log_level: self
                    .log_level
                    .unwrap_or_else(|| DEFAULT_LOG_LEVEL.to_string()),
            },
        }
    }
}
