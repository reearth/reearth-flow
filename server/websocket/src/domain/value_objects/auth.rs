use thiserror::Error;

#[derive(Debug, Clone)]
pub struct AuthToken(String);

#[derive(Debug, Error)]
pub enum AuthTokenError {
    #[error("token cannot be empty")]
    Empty,
}

impl AuthToken {
    pub fn new(token: impl Into<String>) -> Result<Self, AuthTokenError> {
        let token = token.into();
        if token.trim().is_empty() {
            Err(AuthTokenError::Empty)
        } else {
            Ok(Self(token))
        }
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for AuthToken {
    fn as_ref(&self) -> &str {
        self.value()
    }
}
