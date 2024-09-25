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

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_from_env() {
        env::set_var("SERVER_ADDR", "192.168.1.1:9000");
        env::set_var(
            "AUTH_SERVICE_URL",
            "http://192.168.1.1:8080/api/validate-token",
        );

        let config = Config::from_env().unwrap();
        assert_eq!(config.server_addr, "192.168.1.1:9000");
        assert_eq!(
            config.auth_service_url,
            "http://192.168.1.1:8080/api/validate-token"
        );

        env::remove_var("SERVER_ADDR");
        env::remove_var("AUTH_SERVICE_URL");

        let config = Config::from_env().unwrap();
        assert_eq!(config.server_addr, "127.0.0.1:8000");
        assert_eq!(
            config.auth_service_url,
            "http://127.0.0.1:8081/api/validate-token"
        );
    }
}
