use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use tracing::warn;

const API_SECRET_HEADER: &str = "X-API-Secret";

/// Middleware that validates requests against a shared API secret.
///
/// If no secret is configured (`None`), all requests are allowed — this is the
/// expected behaviour in development environments.
///
/// If a secret is configured, every request must include an `X-API-Secret`
/// header whose value matches the configured secret exactly. Requests with a
/// missing or incorrect header are rejected with `401 Unauthorized`.
pub async fn api_auth_layer(
    State(api_secret): State<Option<String>>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    if let Some(ref expected_secret) = api_secret {
        match request.headers().get(API_SECRET_HEADER) {
            Some(provided) => {
                if provided.as_bytes() != expected_secret.as_bytes() {
                    warn!("API request rejected: invalid {} header", API_SECRET_HEADER);
                    return Err(StatusCode::UNAUTHORIZED);
                }
            }
            None => {
                warn!("API request rejected: missing {} header", API_SECRET_HEADER);
                return Err(StatusCode::UNAUTHORIZED);
            }
        }
    }

    Ok(next.run(request).await)
}
