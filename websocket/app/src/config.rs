use serde::Deserialize;
use std::env;
use std::fs;
use tracing::{
    info,
    log::{error, warn},
};
use url::Url;

const DEFAULT_REDIS_URL: &str = "redis://localhost:6379/0";
const DEFAULT_HOST: &str = "127.0.0.1";

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_redis_url")]
    pub redis_url: String,
    #[serde(default = "default_env")]
    pub environment: String,
    #[serde(default = "default_allowed_origins")]
    pub allowed_origins: Vec<String>,
}

fn default_host() -> String {
    DEFAULT_HOST.to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_redis_url() -> String {
    DEFAULT_REDIS_URL.to_string()
}

fn default_env() -> String {
    "development".to_string()
}

fn default_allowed_origins() -> Vec<String> {
    vec!["http://localhost:3000".to_string()]
}

impl Config {
    pub fn from_file(file_path: &str, env: &str) -> Self {
        let file_content = fs::read_to_string(file_path);
        match file_content {
            Ok(content) => {
                let configs: Result<std::collections::HashMap<String, Config>, _> =
                    serde_yaml::from_str(&content);
                match configs {
                    Ok(mut env_configs) => {
                        if let Some(config) = env_configs.remove(env) {
                            info!("Using configuration for environment: {}", env);
                            config
                        } else {
                            warn!("Environment '{}' not found in config file. Using default configuration.", env);
                            Config::default()
                        }
                    }
                    Err(err) => {
                        error!("Failed to parse config file: {:?}", err);
                        Config::default()
                    }
                }
            }
            Err(err) => {
                error!("Failed to read config file '{}': {:?}", file_path, err);
                Config::default()
            }
        }
    }

    pub fn is_test_env(&self) -> bool {
        self.environment.eq_ignore_ascii_case("test")
    }

    pub fn is_development_env(&self) -> bool {
        self.environment.eq_ignore_ascii_case("development")
    }

    pub fn is_production_env(&self) -> bool {
        self.environment.eq_ignore_ascii_case("production")
    }
}

impl Default for Config {
    fn default() -> Self {
        let allowed_origins = env::var("ALLOW_ORIGIN")
            .map(|origins| {
                let origins: Vec<String> = origins
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .filter(|s| {
                        let is_valid = Url::parse(s).is_ok();
                        if !is_valid {
                            warn!("Invalid origin URL ignored: {}", s);
                        }
                        is_valid
                    })
                    .collect();

                info!("Configured CORS allowed origins: {:?}", origins);

                if origins.is_empty() {
                    warn!("No valid origins found in ALLOW_ORIGIN, using defaults");
                    default_allowed_origins()
                } else {
                    origins
                }
            })
            .unwrap_or_else(|_| {
                info!(
                    "ALLOW_ORIGIN not set, using default origins: {:?}",
                    default_allowed_origins()
                );
                default_allowed_origins()
            });

        Config {
            host: default_host(),
            port: default_port(),
            redis_url: default_redis_url(),
            environment: default_env(),
            allowed_origins,
        }
    }
}
