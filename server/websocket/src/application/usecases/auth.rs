use std::sync::Arc;

use crate::domain::repositories::auth::{AuthError, AuthService};
use crate::domain::value_objects::auth::AuthToken;

pub struct VerifyTokenUseCase {
    auth_service: Arc<dyn AuthService>,
}

impl VerifyTokenUseCase {
    pub fn new(auth_service: Arc<dyn AuthService>) -> Self {
        Self { auth_service }
    }

    pub async fn execute(&self, token: &AuthToken) -> Result<(), AuthError> {
        self.auth_service.verify_token(token).await
    }
}

impl std::fmt::Debug for VerifyTokenUseCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VerifyTokenUseCase")
            .field("auth_service", &"Arc<dyn AuthService>")
            .finish()
    }
}
