use dotenv;
use serde::Deserialize;
use std::env;
use std::path::Path;
use tracing::{info, warn};

use crate::{broadcast::RedisConfig, storage::gcs::GcsConfig};

// Default configuration constants
const DEFAULT_REDIS_URL: &str = "redis://127.0.0.1:6379";
const DEFAULT_REDIS_TTL: u64 = 3600;
const DEFAULT_GCS_BUCKET: &str = "yrs-dev";
const DEFAULT_GCS_ENDPOINT: &str = "http://localhost:4443";
#[cfg(feature = "auth")]
const DEFAULT_AUTH_URL: &str = "http://localhost:8080/api/verify/ws-token";
#[cfg(feature = "auth")]
const DEFAULT_AUTH_TIMEOUT_MS: u64 = 5000;
const DEFAULT_APP_ENV: &str = "development";
const DEFAULT_ORIGINS: &str = "http://localhost:3000";

#[derive(Debug, Deserialize)]
pub struct AuthConfig {
    pub url: String,
    pub timeout_ms: u64,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub env: String,
    pub origins: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub redis: RedisConfig,
    pub gcs: GcsConfig,
    #[cfg(feature = "auth")]
    pub auth: AuthConfig,
    pub app: AppConfig,
}

impl Config {
    pub fn load() -> Self {
        // Check if .env file exists
        let env_path = Path::new(".env");
        if env_path.exists() {
            info!(path = ?env_path, "Found .env file");
            // Load .env file
            dotenv::from_path(env_path).ok();
            // Log all environment variables
            info!(
                redis_url = ?env::var("REEARTH_FLOW_REDIS_URL").ok(),
                redis_ttl = ?env::var("REEARTH_FLOW_REDIS_TTL").ok(),
                gcs_bucket = ?env::var("REEARTH_FLOW_GCS_BUCKET_NAME").ok(),
                gcs_endpoint = ?env::var("REEARTH_FLOW_GCS_ENDPOINT").ok(),
                auth_url = ?env::var("REEARTH_FLOW_AUTH_URL").ok(),
                auth_timeout = ?env::var("REEARTH_FLOW_AUTH_TIMEOUT_MS").ok(),
                app_env = ?env::var("REEARTH_FLOW_APP_ENV").ok(),
                origins = ?env::var("REEARTH_FLOW_ORIGINS").ok(),
            );

            let mut config = Config::default();

            if let Ok(url) = env::var("REEARTH_FLOW_REDIS_URL") {
                config.redis.url = url;
            }
            if let Ok(ttl_str) = env::var("REEARTH_FLOW_REDIS_TTL") {
                if let Ok(ttl) = ttl_str.parse() {
                    config.redis.ttl = ttl;
                }
            }
            if let Ok(bucket) = env::var("REEARTH_FLOW_GCS_BUCKET_NAME") {
                config.gcs.bucket_name = bucket;
            }
            if let Ok(endpoint) = env::var("REEARTH_FLOW_GCS_ENDPOINT") {
                config.gcs.endpoint = Some(endpoint);
            }
            #[cfg(feature = "auth")]
            {
                if let Ok(url) = env::var("REEARTH_FLOW_AUTH_URL") {
                    config.auth.url = url;
                }
                if let Ok(timeout_str) = env::var("REEARTH_FLOW_AUTH_TIMEOUT_MS") {
                    if let Ok(timeout) = timeout_str.parse() {
                        config.auth.timeout_ms = timeout;
                    }
                }
            }
            if let Ok(env_val) = env::var("REEARTH_FLOW_APP_ENV") {
                config.app.env = env_val;
            }
            if let Ok(origins) = env::var("REEARTH_FLOW_ORIGINS") {
                config.app.origins = origins;
            }
            config
        } else {
            warn!(path = ?env_path, "No .env file found, using default values");
            let config = Config::default();
            info!("redis: {:?}", config.redis);
            info!("gcs: {:?}", config.gcs);
            info!("app: {:?}", config.app);
            #[cfg(feature = "auth")]
            info!("auth: {:?}", config.auth);
            config
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        #[cfg(feature = "auth")]
        let config = Config {
            redis: RedisConfig {
                url: DEFAULT_REDIS_URL.to_string(),
                ttl: DEFAULT_REDIS_TTL,
            },
            gcs: GcsConfig {
                bucket_name: DEFAULT_GCS_BUCKET.to_string(),
                endpoint: Some(DEFAULT_GCS_ENDPOINT.to_string()),
            },
            auth: AuthConfig {
                url: DEFAULT_AUTH_URL.to_string(),
                timeout_ms: DEFAULT_AUTH_TIMEOUT_MS,
            },
            app: AppConfig {
                env: DEFAULT_APP_ENV.to_string(),
                origins: DEFAULT_ORIGINS.to_string(),
            },
        };

        #[cfg(not(feature = "auth"))]
        let config = Config {
            redis: RedisConfig {
                url: DEFAULT_REDIS_URL.to_string(),
                ttl: DEFAULT_REDIS_TTL,
            },
            gcs: GcsConfig {
                bucket_name: DEFAULT_GCS_BUCKET.to_string(),
                endpoint: Some(DEFAULT_GCS_ENDPOINT.to_string()),
            },
            app: AppConfig {
                env: DEFAULT_APP_ENV.to_string(),
                origins: DEFAULT_ORIGINS.to_string(),
            },
        };

        config
    }
}
