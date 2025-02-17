use crate::conf::AuthConfig;
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, time::Duration};

#[derive(Debug, Deserialize, Serialize)]
struct AuthResponse {
    authorized: bool,
}

#[derive(Clone, Debug)]
pub struct AuthService {
    client: Client,
    config: AuthConfig,
}

impl AuthService {
    pub fn new(config: AuthConfig) -> Self {
        {
            let client = Client::builder()
                .timeout(Duration::from_millis(config.timeout_ms))
                .build()
                .expect("Failed to create HTTP client");

            Self { client, config }
        }
    }

    pub async fn verify_token(&self, token: &str) -> Result<bool> {
        {
            let response = self
                .client
                .post(&self.config.url)
                .header("Authorization", format!("Bearer {}", token))
                .send()
                .await?;

            tracing::debug!("response: {:?}", response);

            if !response.status().is_success() {
                return Ok(false);
            }

            let auth_response = response.json::<AuthResponse>().await?;
            Ok(auth_response.authorized)
        }
    }
}
