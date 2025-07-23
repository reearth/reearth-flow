use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use tracing::error;

/// Error Handler Middleware - Interface Layer
/// Provides centralized error handling for HTTP responses
pub struct ErrorHandler;

impl ErrorHandler {
    /// Handle validation errors
    pub fn handle_validation_error(message: String) -> Response {
        error!("Validation error: {}", message);
        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Validation Error",
                "message": message
            })),
        )
            .into_response()
    }

    /// Handle not found errors
    pub fn handle_not_found(resource: &str) -> Response {
        error!("{} not found", resource);
        (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "Not Found",
                "message": format!("{} not found", resource)
            })),
        )
            .into_response()
    }

    /// Handle internal server errors
    pub fn handle_internal_error(message: String) -> Response {
        error!("Internal server error: {}", message);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Internal Server Error",
                "message": "An unexpected error occurred"
            })),
        )
            .into_response()
    }
}
