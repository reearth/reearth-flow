use axum::body::Body;
use axum::response::{IntoResponse, Response};
use reqwest::StatusCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum JwtError {
    #[allow(dead_code)]
    #[error("Verification failed with status code: {0}")]
    VerificationFailed(StatusCode),
    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("Failed to build uri: {0}")]
    UriBuild(#[from] axum::http::uri::InvalidUri),
    #[allow(dead_code)]
    #[error("Can't find token: {0}")]
    TokenNotFound(StatusCode),
    #[error("Auth service error: {0}")]
    AuthService(#[from] services::AuthServiceError),
}

impl IntoResponse for JwtError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            JwtError::VerificationFailed(status_code) => (
                status_code,
                Body::from(format!("Verification failed: {}", status_code)),
            ),
            JwtError::Request(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Body::from(format!("Request error: {}", e)),
            ),
            JwtError::UriBuild(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Body::from(format!("Failed to build uri: {}", e)),
            ),
            JwtError::TokenNotFound(status_code) => (
                status_code,
                Body::from(format!("Can't find token: {}", status_code)),
            ),
            JwtError::AuthService(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Body::from(format!("Auth service error: {}", e)),
            ),
        };

        Response::builder()
            .status(status)
            .body(body)
            .map_err(|e| {
                tracing::error!("Failed to build response: {}", e);
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from("Internal Server Error"))
                    .unwrap_or_else(|_| {
                        Response::new(Body::from("Critical error: Unable to build any response"))
                    })
            })
            .unwrap_or_else(|fallback_response| fallback_response)
    }
}
