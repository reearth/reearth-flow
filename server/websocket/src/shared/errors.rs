use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AppError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("resource not found: {0}")]
    NotFound(String),
    #[error("operation not supported: {0}")]
    UnsupportedOperation(String),
    #[error("unexpected error: {0}")]
    Unexpected(String),
}

impl AppError {
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput(message.into())
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound(message.into())
    }

    pub fn unsupported(message: impl Into<String>) -> Self {
        Self::UnsupportedOperation(message.into())
    }

    pub fn unexpected(message: impl Into<String>) -> Self {
        Self::Unexpected(message.into())
    }
}
