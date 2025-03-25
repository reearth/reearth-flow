use crate::conf::AuthConfig;
use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::debug;

#[derive(Debug, Clone)]
pub struct AuthService {
    client: Client,
    url: String,
}

#[derive(Serialize, Deserialize)]
pub struct TokenVerifyRequest {
    pub token: String,
}

#[derive(Serialize, Deserialize)]
pub struct TokenVerifyResponse {
    pub authorized: bool,
}

impl AuthService {
    pub async fn new(config: AuthConfig) -> Result<Self> {
        debug!("Connecting to auth service at: {}", config.url);
        let client = reqwest::ClientBuilder::new()
            .no_deflate()
            .no_brotli()
            .no_gzip()
            .tcp_keepalive(None)
            .pool_max_idle_per_host(0)
            .build()?;
        Ok(Self {
            client,
            url: config.url,
        })
    }

    pub async fn verify_token(&self, token: &str) -> Result<bool> {
        debug!("Verifying token");

        let request = TokenVerifyRequest {
            token: token.to_string(),
        };

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        let url = format!("{}/auth/verify", self.url);
        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("Cache-Control", "no-cache, no-store, must-revalidate")
            .header("Pragma", "no-cache")
            .header("X-Request-Time", timestamp.to_string())
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            debug!("Token verification failed: {}", error_text);
            return Err(anyhow!("Token verification failed: {}", error_text));
        }

        let verify_response = response.json::<TokenVerifyResponse>().await?;
        debug!("Token verification result: {}", verify_response.authorized);

        if !verify_response.authorized {
            return Err(anyhow!("Token verification failed: unauthorized"));
        }

        Ok(verify_response.authorized)
    }
}
