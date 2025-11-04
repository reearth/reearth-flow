use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::conf::AuthConfig;
use crate::domain::repositories::auth::{AuthError, AuthService};
use crate::domain::value_objects::auth::AuthToken;

#[derive(Clone, Debug)]
pub struct AuthHttpClient {
    client: Client,
    config: AuthConfig,
}

impl AuthHttpClient {
    pub async fn new(config: AuthConfig) -> Result<Self> {
        let client = reqwest::ClientBuilder::new().build()?;
        Ok(Self { client, config })
    }
}

#[derive(Serialize, Deserialize)]
struct TokenVerifyRequest {
    token: String,
}

#[derive(Serialize, Deserialize)]
struct TokenVerifyResponse {
    authorized: bool,
}

#[async_trait]
impl AuthService for AuthHttpClient {
    async fn verify_token(&self, token: &AuthToken) -> Result<(), AuthError> {
        let request = TokenVerifyRequest {
            token: token.value().to_string(),
        };

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        let url = format!("{}/auth/verify", self.config.url);
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
            .await
            .map_err(|err| AuthError::Request(err.to_string()))?;

        if response.status().is_success() {
            let verify_response = response
                .json::<TokenVerifyResponse>()
                .await
                .map_err(|err| AuthError::Request(err.to_string()))?;

            if verify_response.authorized {
                Ok(())
            } else {
                Err(AuthError::Unauthorized)
            }
        } else if response.status().as_u16() == 401 {
            Err(AuthError::Unauthorized)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "failed to read error".to_string());
            Err(AuthError::Request(error_text))
        }
    }
}

pub async fn create_auth_service(config: AuthConfig) -> Result<Arc<dyn AuthService>> {
    let client = AuthHttpClient::new(config).await?;
    Ok(Arc::new(client))
}
