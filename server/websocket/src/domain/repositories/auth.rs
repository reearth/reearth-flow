use async_trait::async_trait;

use crate::domain::value_objects::auth::AuthToken;

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Request failed: {0}")]
    Request(String),
}

#[async_trait]
pub trait AuthService: Send + Sync {
    async fn verify_token(&self, token: &AuthToken) -> Result<(), AuthError>;
}
