use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_redis_url")]
    pub redis_url: String,
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_redis_url() -> String {
    "redis://127.0.0.1:6379/0".to_string()
}

impl Config {
    pub fn from_env() -> Result<Self, envy::Error> {
        match envy::from_env::<Config>() {
            Ok(config) => Ok(config),
            Err(error) => {
                eprintln!(
                    "Warning: Failed to load configuration from environment: {:?}",
                    error
                );
                eprintln!("Using default configuration");
                Ok(Config {
                    host: default_host(),
                    port: default_port(),
                    redis_url: default_redis_url(),
                })
            }
        }
    }
}
