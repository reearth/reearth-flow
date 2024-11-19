use axum::{
    body::Body,
    http::{Request, Response, StatusCode},
    middleware,
};

pub async fn auth_middleware(
    req: Request<Body>,
    next: middleware::Next,
) -> Result<Response<Body>, StatusCode> {
    if let Some(auth_header) = req.headers().get("Authorization") {
        // Extract the token
        let auth_header = auth_header.to_str().map_err(|_| StatusCode::BAD_REQUEST)?;
        if !auth_header.starts_with("Bearer ") {
            return Err(StatusCode::UNAUTHORIZED);
        }
        let _token = &auth_header[7..];

        // TODO: Implement token validation
        // Example: validate_token(token).await?;

        Ok(next.run(req).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        middleware,
        response::Response,
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    async fn mock_handler(_: Request<Body>) -> Result<Response<Body>, StatusCode> {
        Ok(Response::new(Body::from("Success")))
    }

    #[tokio::test]
    async fn test_auth_middleware_with_valid_token() {
        // Create a router with middleware and mock handler
        let app = Router::new()
            .route("/", get(mock_handler))
            .layer(middleware::from_fn(auth_middleware));

        // Create a request with a valid authorization header
        let request = Request::builder()
            .uri("/")
            .header("Authorization", "Bearer valid_token")
            .body(Body::empty())
            .unwrap();

        // Send the request
        let response = app.oneshot(request).await.unwrap();

        // Assert that the response is successful
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_auth_middleware_without_token() {
        // Create a router with middleware and mock handler
        let app = Router::new()
            .route("/", get(mock_handler))
            .layer(middleware::from_fn(auth_middleware));

        // Create a request without an authorization header
        let request = Request::builder().uri("/").body(Body::empty()).unwrap();

        // Send the request
        let response = app.oneshot(request).await.unwrap();

        // Assert that the response is unauthorized
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_auth_middleware_with_invalid_token() {
        // Create a router with middleware and mock handler
        let app = Router::new()
            .route("/", get(mock_handler))
            .layer(middleware::from_fn(auth_middleware));

        // Create a request with an invalid authorization header
        let request = Request::builder()
            .uri("/")
            .header("Authorization", "InvalidTokenFormat")
            .body(Body::empty())
            .unwrap();

        // Send the request
        let response = app.oneshot(request).await.unwrap();

        // Assert that the response is unauthorized
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
