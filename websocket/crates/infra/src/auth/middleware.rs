use crate::auth::error::JwtError;
use crate::auth::jwt::Jwt;
use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use services::AuthServiceClient;
use tracing::{debug, error, info};

pub async fn auth_middleware<B>(
    client: AuthServiceClient,
    req: Request<Body>,
    next: Next,
) -> Result<Response, JwtError> {
    debug!("Entering auth_middleware");

    let token = match extract_token(&req) {
        Some(t) => {
            info!("Token extracted successfully");
            t
        }
        None => {
            error!("Token not found in request");
            return Err(JwtError::TokenNotFound(StatusCode::UNAUTHORIZED));
        }
    };

    let jwt = Jwt::new(token.clone(), client);
    debug!("Attempting to verify token");

    match jwt.verify().await {
        Ok(_) => {
            info!("Token verified successfully");
            let response = next.run(req).await;
            debug!("Exiting auth_middleware with successful response");
            Ok(response)
        }
        Err(e) => {
            error!("Token verification failed: {:?}", e);
            Err(e)
        }
    }
}

#[inline]
fn extract_token<B>(req: &Request<B>) -> Option<String> {
    req.headers()
        .get("Authorization")
        .and_then(|auth_header| auth_header.to_str().ok())
        .and_then(|auth_str| auth_str.strip_prefix("Bearer ").map(str::to_string))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        middleware::from_fn,
        routing::get,
        Router,
    };
    use hyper::header::AUTHORIZATION;
    use tower::ServiceExt;
    use wiremock::{
        matchers::{header, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    #[test]
    fn test_extract_token() {
        let req = Request::builder()
            .header(AUTHORIZATION, "Bearer test_token")
            .body(Body::empty())
            .unwrap();
        assert_eq!(extract_token(&req), Some("test_token".to_string()));

        let req_no_auth = Request::builder().body(Body::empty()).unwrap();
        assert_eq!(extract_token(&req_no_auth), None);

        let req_invalid_auth = Request::builder()
            .header(AUTHORIZATION, "Invalid test_token")
            .body(Body::empty())
            .unwrap();
        assert_eq!(extract_token(&req_invalid_auth), None);
    }

    #[tokio::test]
    async fn test_auth_middleware_with_valid_token() {
        let mock_server = MockServer::start().await;
        let auth_service_url = mock_server.uri();
        std::env::set_var("AUTH_SERVICE_URL", &auth_service_url);

        Mock::given(method("GET"))
            .and(path("/auth"))
            .and(header(AUTHORIZATION, "Bearer valid_token"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let uri = format!("{}/auth", auth_service_url);

        let app = Router::new()
            .route("/", get(|| async { "Protected" }))
            .layer(from_fn(move |req, next| {
                let client = AuthServiceClient::new(&uri).unwrap();
                auth_middleware::<Body>(client, req, next)
            }));

        let request = Request::builder()
            .uri("/")
            .method("GET")
            .header(AUTHORIZATION, "Bearer valid_token")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(&body[..], b"Protected");

        mock_server.verify().await;
        std::env::remove_var("AUTH_SERVICE_URL");
    }

    #[tokio::test]
    async fn test_auth_middleware_with_invalid_token() {
        let mock_server = MockServer::start().await;
        let auth_service_url = mock_server.uri();
        std::env::set_var("AUTH_SERVICE_URL", &auth_service_url);

        Mock::given(method("GET"))
            .and(path("/auth"))
            .and(header(AUTHORIZATION, "Bearer invalid_token"))
            .respond_with(ResponseTemplate::new(401))
            .expect(1)
            .mount(&mock_server)
            .await;

        let uri = format!("{}/auth", auth_service_url);

        let app = Router::new()
            .route("/", get(|| async { "Protected" }))
            .layer(from_fn(move |req, next| {
                let client = AuthServiceClient::new(&uri).unwrap();
                auth_middleware::<Body>(client, req, next)
            }));
        let request = Request::builder()
            .uri("/")
            .method("GET")
            .header(AUTHORIZATION, "Bearer invalid_token")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        mock_server.verify().await;
        std::env::remove_var("AUTH_SERVICE_URL");
    }
}
