#[cfg(feature = "auth")]
use crate::domain::value_objects::conf::DEFAULT_AUTH_URL;
use crate::domain::value_objects::conf::{
    DEFAULT_APP_ENV, DEFAULT_GCS_BUCKET, DEFAULT_ORIGINS, DEFAULT_REDIS_TTL, DEFAULT_REDIS_URL,
    DEFAULT_WS_PORT,
};
use dotenv;
use serde::Deserialize;
use std::env;
use std::path::Path;
use thiserror::Error;
use tracing::{info, warn};

use crate::{storage::gcs::GcsConfig, RedisConfig};

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

        let config = builder.build();

        info!("Final configuration:");
        info!("Redis: {:?}", config.redis);
        info!("GCS: {:?}", config.gcs);
        info!("App: {:?}", config.app);
        info!("WebSocket Port: {}", config.ws_port);
        #[cfg(feature = "auth")]
        info!("Auth: {:?}", config.auth);

        Ok(config)
    }
}

#[derive(Default)]
pub struct ConfigBuilder {
    redis_url: Option<String>,
    redis_ttl: Option<u64>,
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

    pub fn build(self) -> Config {
        Config {
            redis: RedisConfig {
                url: self
                    .redis_url
                    .unwrap_or_else(|| DEFAULT_REDIS_URL.to_string()),
                ttl: self.redis_ttl.unwrap_or(DEFAULT_REDIS_TTL),
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
        }
    }
}
