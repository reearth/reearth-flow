use dotenv::dotenv;
use std::env;

pub struct Config {
    pub server_addr: String,
    pub auth_service_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        dotenv().ok();

        Ok(Config {
            server_addr: env::var("SERVER_ADDR").unwrap_or_else(|_| "127.0.0.1:8000".to_string()),
            auth_service_url: env::var("AUTH_SERVICE_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8081/api/validate-token".to_string()),
        })
    }
}
