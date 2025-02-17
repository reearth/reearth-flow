use crate::{conf::AuthConfig, proto};
use anyhow::Result;
use proto::{auth_service_client::AuthServiceClient, ApiTokenVerifyRequest};
use tonic::transport::Channel;
use tracing::debug;

#[derive(Debug, Clone)]
pub struct AuthService {
    client: AuthServiceClient<Channel>,
}

impl AuthService {
    pub async fn new(config: AuthConfig) -> Result<Self> {
        debug!("Connecting to auth service at: {}", config.url);
        let channel = Channel::from_shared(config.url)?.connect().await?;
        let client = AuthServiceClient::new(channel);
        Ok(Self { client })
    }

    pub async fn verify_token(&self, token: &str) -> Result<bool> {
        debug!("Verifying token");
        let request = ApiTokenVerifyRequest {
            token: token.to_string(),
        };

        match self.client.clone().verify_api_token(request).await {
            Ok(response) => {
                let authorized = response.into_inner().authorized;
                debug!("Token verification result: {}", authorized);
                Ok(authorized)
            }
            Err(e) => {
                debug!("Token verification failed: {}", e);
                Ok(false)
            }
        }
    }
}
