use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use tracing::{info, warn};
use std::time::Instant;

/// Logging Middleware - Interface Layer
/// Provides request/response logging for HTTP endpoints
pub struct LoggingMiddleware;

impl LoggingMiddleware {
    /// Log HTTP requests and responses
    pub async fn log_request(request: Request, next: Next) -> Response {
        let start = Instant::now();
        let method = request.method().clone();
        let uri = request.uri().clone();
        let version = request.version();

        info!(
            "HTTP Request: {} {} {:?}",
            method,
            uri,
            version
        );

        let response = next.run(request).await;
        let duration = start.elapsed();
        let status = response.status();

        if status.is_success() {
            info!(
                "HTTP Response: {} {} - {} in {:?}",
                method,
                uri,
                status,
                duration
            );
        } else {
            warn!(
                "HTTP Response: {} {} - {} in {:?}",
                method,
                uri,
                status,
                duration
            );
        }

        response
    }
}
