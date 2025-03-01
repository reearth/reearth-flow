use crate::conf::AuthConfig;
use crate::thrift::{APITokenVerifyRequest, APITokenVerifyResponse};
use anyhow::{anyhow, Result};
use reqwest::Client;
use serde_json::json;
use tracing::debug;

#[derive(Debug, Clone)]
pub struct AuthService {
    client: Client,
    url: String,
}

impl AuthService {
    pub async fn new(config: AuthConfig) -> Result<Self> {
        debug!("Connecting to auth service at: {}", config.url);
        let client = Client::new();
        Ok(Self {
            client,
            url: config.url,
        })
    }

    pub async fn verify_token(&self, token: &str) -> Result<bool> {
        debug!("Verifying token");

        let thrift_request = APITokenVerifyRequest::new(token.to_string());

        let json_request = json!({
            "method": "VerifyAPIToken",
            "type": "CALL",
            "seqid": 1,
            "args": {
                "request": {
                    "token": thrift_request.token
                }
            }
        });

        let url = format!("{}/AuthService", self.url);
        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(&json_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            debug!("Token verification failed: {}", error_text);
            return Err(anyhow!("Token verification failed: {}", error_text));
        }

        let json_response = response.json::<serde_json::Value>().await?;

        let authorized = json_response
            .get("value")
            .and_then(|v| v.get("success"))
            .and_then(|v| v.get("authorized"))
            .and_then(|v| v.as_bool())
            .ok_or_else(|| {
                debug!("Invalid response format: {:?}", json_response);
                anyhow!("Invalid response format")
            })?;

        let thrift_response = APITokenVerifyResponse::new(authorized);

        debug!("Token verification result: {}", authorized);

        if !authorized {
            return Err(anyhow!("Token verification failed: unauthorized"));
        }

        Ok(thrift_response.authorized.unwrap_or(false))
    }
}
