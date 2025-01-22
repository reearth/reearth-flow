use crate::conf::AuthConfig;
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize)]
struct AuthRequest {
    token: String,
    doc_id: String,
}

#[derive(Debug, Deserialize)]
struct AuthResponse {
    authorized: bool,
}

pub struct AuthService {
    #[cfg(feature = "auth")]
    client: Client,
    #[cfg(feature = "auth")]
    config: AuthConfig,
}

impl AuthService {
    pub fn new(config: AuthConfig) -> Self {
        #[cfg(feature = "auth")]
        {
            let client = Client::builder()
                .timeout(Duration::from_millis(config.timeout_ms))
                .build()
                .expect("Failed to create HTTP client");

            Self { client, config }
        }

        #[cfg(not(feature = "auth"))]
        {
            Self {}
        }
    }

    pub async fn verify_token(&self, token: &str, doc_id: &str) -> Result<bool> {
        #[cfg(feature = "auth")]
        {
            let request = AuthRequest {
                token: token.to_string(),
                doc_id: doc_id.to_string(),
            };

            let response = self
                .client
                .post(&self.config.url)
                .json(&request)
                .send()
                .await?;

            if !response.status().is_success() {
                return Ok(false);
            }

            let auth_response = response.json::<AuthResponse>().await?;
            Ok(auth_response.authorized)
        }

        #[cfg(not(feature = "auth"))]
        {
            Ok(true)
        }
    }
}
