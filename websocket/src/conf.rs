use config::{Config as ConfigLoader, File, FileFormat};
use serde::Deserialize;

use crate::{broadcast::RedisConfig, storage::gcs::GcsConfig};

#[derive(Debug, Deserialize)]
pub struct AuthConfig {
    pub url: String,
    pub timeout_ms: u64,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub redis: RedisConfig,
    pub gcs: GcsConfig,
    #[cfg(feature = "auth")]
    pub auth: AuthConfig,
}

impl Config {
    pub fn load() -> Result<Self, config::ConfigError> {
        let yaml_file = "conf.yaml";
        let mut builder = ConfigLoader::builder();

        // Set default values first
        builder = builder
            .set_default("redis.url", "redis://127.0.0.1:6379")?
            .set_default("redis.ttl", 3600)?
            .set_default("gcs.bucket_name", "yrs-dev")?
            .set_default("gcs.endpoint", "http://localhost:4443")?;

        #[cfg(feature = "auth")]
        {
            builder = builder
                .set_default("auth.url", "http://localhost:8080/auth")?
                .set_default("auth.timeout_ms", 5000)?;
        }

        // If yaml exists, it will override the defaults
        if std::path::Path::new(yaml_file).exists() {
            builder = builder.add_source(File::new(yaml_file, FileFormat::Yaml));
        }

        let settings = builder.build()?;
        settings.try_deserialize()
    }
}

impl Default for Config {
    fn default() -> Self {
        #[cfg(feature = "auth")]
        {
            Config {
                redis: RedisConfig {
                    url: "redis://127.0.0.1:6379".to_string(),
                    ttl: 3600,
                },
                gcs: GcsConfig {
                    bucket_name: "yrs-dev".to_string(),
                    endpoint: Some("http://localhost:4443".to_string()),
                },
                auth: AuthConfig {
                    url: "http://localhost:8080/auth".to_string(),
                    timeout_ms: 5000,
                },
            }
        }

        #[cfg(not(feature = "auth"))]
        {
            Config {
                redis: RedisConfig {
                    url: "redis://127.0.0.1:6379".to_string(),
                    ttl: 3600,
                },
                gcs: GcsConfig {
                    bucket_name: "yrs-dev".to_string(),
                    endpoint: Some("http://localhost:4443".to_string()),
                },
            }
        }
    }
}
