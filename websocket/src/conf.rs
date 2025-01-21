use config::{Config as ConfigLoader, File, FileFormat};
use serde::Deserialize;

use crate::{broadcast::RedisConfig, storage::gcs::GcsConfig};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub redis: RedisConfig,
    pub gcs: GcsConfig,
}

impl Config {
    pub fn load() -> Result<Self, config::ConfigError> {
        let builder = ConfigLoader::builder()
            .add_source(File::new("conf.yaml", FileFormat::Yaml))
            // Add default values as fallback
            .set_default("redis.url", "redis://127.0.0.1:6379")?
            .set_default("redis.ttl", 3600)?
            .set_default("gcs.bucket_name", "yrs-dev")?
            .set_default("gcs.endpoint", "http://localhost:4443")?;

        let settings = builder.build()?;
        settings.try_deserialize()
    }
}

impl Default for Config {
    fn default() -> Self {
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
