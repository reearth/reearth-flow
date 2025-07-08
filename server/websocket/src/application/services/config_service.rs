use anyhow::Result;
use serde::Deserialize;
use std::env;
use std::path::Path;
use tracing::{info, warn};

const DEFAULT_REDIS_URL: &str = "redis://127.0.0.1:6379";
const DEFAULT_REDIS_TTL: u64 = 43200;
const DEFAULT_GCS_BUCKET: &str = "yrs-dev";
const DEFAULT_APP_ENV: &str = "development";
const DEFAULT_ORIGINS: &[&str] = &[
    "http://localhost:3000",
    "https://api.flow.test.reearth.dev",
    "https://api.flow.reearth.dev",
    "http://localhost:8000",
    "http://localhost:8080",
];
const DEFAULT_WS_PORT: &str = "8000";

#[derive(Debug, Deserialize, Clone)]
pub struct RedisConfig {
    pub url: String,
    pub ttl: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GcsConfig {
    pub bucket_name: String,
    pub endpoint: Option<String>,
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
    pub app: AppConfig,
    pub ws_port: String,
}

pub struct ConfigService;

impl ConfigService {
    pub fn load() -> Result<Config> {
        if dotenv::from_path(Path::new(".env")).is_ok() {
            info!("Loaded configuration from .env file");
        } else {
            warn!("No .env file found, using environment variables");
        }

        let redis = RedisConfig {
            url: env::var("REEARTH_FLOW_REDIS_URL")
                .unwrap_or_else(|_| DEFAULT_REDIS_URL.to_string()),
            ttl: env::var("REEARTH_FLOW_REDIS_TTL")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(DEFAULT_REDIS_TTL),
        };

        let gcs = GcsConfig {
            bucket_name: env::var("REEARTH_FLOW_GCS_BUCKET_NAME")
                .unwrap_or_else(|_| DEFAULT_GCS_BUCKET.to_string()),
            endpoint: env::var("REEARTH_FLOW_GCS_ENDPOINT")
                .ok()
                .filter(|s| !s.is_empty()),
        };

        let app = AppConfig {
            env: env::var("REEARTH_FLOW_APP_ENV").unwrap_or_else(|_| DEFAULT_APP_ENV.to_string()),
            origins: env::var("REEARTH_FLOW_ORIGINS")
                .map(|s| {
                    s.split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                })
                .unwrap_or_else(|_| DEFAULT_ORIGINS.iter().map(|s| s.to_string()).collect()),
        };

        let ws_port =
            env::var("REEARTH_FLOW_WS_PORT").unwrap_or_else(|_| DEFAULT_WS_PORT.to_string());

        let config = Config {
            redis,
            gcs,
            app,
            ws_port,
        };

        info!("Configuration loaded:");
        info!("Redis: {:?}", config.redis);
        info!("GCS: {:?}", config.gcs);
        info!("App: {:?}", config.app);
        info!("WebSocket Port: {}", config.ws_port);

        Ok(config)
    }
}
