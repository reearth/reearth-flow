use anyhow::Result;
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub ttl: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GcsConfig {
    pub bucket_name: String,
    pub endpoint: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    pub url: String,
    pub timeout: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub origins: Vec<String>,
    pub environment: String,
    pub ws_port: String,
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub redis: RedisConfig,
    pub gcs: GcsConfig,
    pub auth: Option<AuthConfig>,
    pub server: ServerConfig,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        let redis = RedisConfig {
            url: env::var("REEARTH_FLOW_REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            ttl: env::var("REEARTH_FLOW_REDIS_TTL")
                .unwrap_or_else(|_| "43200".to_string())
                .parse()
                .unwrap_or(43200),
        };

        let gcs = GcsConfig {
            bucket_name: env::var("REEARTH_FLOW_GCS_BUCKET_NAME")
                .unwrap_or_else(|_| "yrs-dev".to_string()),
            endpoint: env::var("REEARTH_FLOW_GCS_ENDPOINT").ok(),
        };

        let auth = if cfg!(feature = "auth") {
            Some(AuthConfig {
                url: env::var("REEARTH_FLOW_THRIFT_AUTH_URL")
                    .unwrap_or_else(|_| "http://localhost:8080".to_string()),
                timeout: env::var("REEARTH_FLOW_AUTH_TIMEOUT")
                    .ok()
                    .and_then(|s| s.parse().ok()),
            })
        } else {
            None
        };

        let server = ServerConfig {
            origins: env::var("REEARTH_FLOW_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:3000,https://api.flow.test.reearth.dev,https://api.flow.reearth.dev,http://localhost:8000,http://localhost:8080".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            environment: env::var("REEARTH_FLOW_APP_ENV")
                .unwrap_or_else(|_| "development".to_string()),
            ws_port: env::var("REEARTH_FLOW_WS_PORT")
                .unwrap_or_else(|_| "8000".to_string()),
        };

        Ok(Self {
            redis,
            gcs,
            auth,
            server,
        })
    }
}
