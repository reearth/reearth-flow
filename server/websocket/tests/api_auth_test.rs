/// Integration tests for the HTTP Document API authentication middleware.
///
/// These tests spin up a real Axum server backed by a testcontainers Redis
/// instance, and send real HTTP requests via reqwest. The auth middleware is
/// exercised end-to-end — no mocks, no reimplemented logic.
///
/// For the auth-rejection cases (401), the middleware rejects before any handler
/// runs, so GCS/Redis are never touched. For the auth-pass case (correct
/// secret), the request reaches the document handler which returns a real HTTP
/// response (typically 500 because there's no real GCS, but crucially NOT 401).
use std::sync::Arc;

use reqwest::StatusCode;
use testcontainers_modules::{
    redis::{Redis, REDIS_PORT},
    testcontainers::runners::AsyncRunner,
};
use tokio::net::TcpListener;

use websocket::infrastructure::gcs::{GcsConfig, GcsStore};
use websocket::infrastructure::redis::RedisStore;
use websocket::infrastructure::repository::document::DocumentRepositoryImpl;
use websocket::infrastructure::websocket::{BroadcastPool, CollaborativeStorage};
use websocket::presentation::http::middleware::api_auth_layer;
use websocket::presentation::http::router::document_routes;
use websocket::{AppState, DocumentUseCase};

/// Build a real AppState pointing at the given Redis URL and a dummy GCS endpoint.
async fn build_state(redis_url: &str, api_secret: Option<String>) -> Arc<AppState> {
    let gcs_store = GcsStore::new_with_config(GcsConfig {
        bucket_name: "test-bucket".to_string(),
        endpoint: Some("http://127.0.0.1:1".to_string()), // bogus, never called for 401 paths
    })
    .await
    .expect("GCS store init should succeed with anonymous endpoint");
    let gcs_store = Arc::new(gcs_store);

    let redis_config = websocket::RedisConfig {
        url: redis_url.to_string(),
        ttl: 3600,
        stream_trim_interval: 300,
        stream_max_message_age: 86400000,
        stream_max_length: 10000,
    };
    let redis_store = RedisStore::new(redis_config)
        .await
        .expect("Redis store init should succeed");
    let redis_store = Arc::new(redis_store);

    let collaborative_storage = Arc::new(CollaborativeStorage::new(
        Arc::clone(&gcs_store),
        Arc::clone(&redis_store),
    ));
    let pool = Arc::new(BroadcastPool::new(
        Arc::clone(&gcs_store),
        Arc::clone(&redis_store),
    ));
    let document_repository = Arc::new(DocumentRepositoryImpl::new(
        Arc::clone(&gcs_store),
        Arc::clone(&collaborative_storage),
    ));
    let document_usecase = Arc::new(DocumentUseCase::new(document_repository));
    let websocket_usecase = Arc::new(websocket::WebsocketUseCase::new(Arc::clone(&pool)));

    #[cfg(feature = "auth")]
    let auth_usecase = {
        let auth_service =
            websocket::infrastructure::auth::create_auth_service(websocket::conf::AuthConfig {
                url: "http://127.0.0.1:1".to_string(), // bogus, never called for these tests
            })
            .await
            .expect("auth service init should succeed");
        Arc::new(websocket::application::usecases::auth::VerifyTokenUseCase::new(auth_service))
    };

    Arc::new(AppState {
        pool,
        document_usecase,
        websocket_usecase,
        #[cfg(feature = "auth")]
        auth_usecase,
        instance_id: "test-instance".to_string(),
        api_secret,
    })
}

/// Start a real HTTP server on a random port and return the base URL.
/// Returns the server task handle and the base URL.
async fn start_test_server(state: Arc<AppState>) -> (tokio::task::JoinHandle<()>, String) {
    use axum::{middleware::from_fn_with_state, Router};

    let app = Router::new()
        .nest(
            "/api",
            document_routes().layer(from_fn_with_state(state.api_secret.clone(), api_auth_layer)),
        )
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let base_url = format!("http://127.0.0.1:{}", addr.port());

    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (handle, base_url)
}

// ---------- Tests with secret configured ----------

#[tokio::test]
async fn rejects_request_with_no_header_when_secret_configured() {
    let redis = Redis::default().start().await.unwrap();
    let host = redis.get_host().await.unwrap();
    let port = redis.get_host_port_ipv4(REDIS_PORT).await.unwrap();
    let redis_url = format!("redis://{}:{}", host, port);

    let state = build_state(&redis_url, Some("test-secret-42".into())).await;
    let (_handle, base_url) = start_test_server(state).await;

    let resp = reqwest::get(format!("{}/api/document/some-doc", base_url))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn rejects_request_with_wrong_secret() {
    let redis = Redis::default().start().await.unwrap();
    let host = redis.get_host().await.unwrap();
    let port = redis.get_host_port_ipv4(REDIS_PORT).await.unwrap();
    let redis_url = format!("redis://{}:{}", host, port);

    let state = build_state(&redis_url, Some("test-secret-42".into())).await;
    let (_handle, base_url) = start_test_server(state).await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/api/document/some-doc", base_url))
        .header("X-API-Secret", "wrong-secret")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn allows_request_with_correct_secret() {
    let redis = Redis::default().start().await.unwrap();
    let host = redis.get_host().await.unwrap();
    let port = redis.get_host_port_ipv4(REDIS_PORT).await.unwrap();
    let redis_url = format!("redis://{}:{}", host, port);

    let state = build_state(&redis_url, Some("test-secret-42".into())).await;
    let (_handle, base_url) = start_test_server(state).await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/api/document/some-doc", base_url))
        .header("X-API-Secret", "test-secret-42")
        .send()
        .await
        .unwrap();
    // Passes middleware — handler runs but likely returns 500 (no real GCS).
    // The key assertion: it's NOT 401.
    assert_ne!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "Correct secret should pass the middleware"
    );
}

// ---------- Tests with no secret (dev mode) ----------

#[tokio::test]
async fn allows_request_without_header_when_no_secret_configured() {
    let redis = Redis::default().start().await.unwrap();
    let host = redis.get_host().await.unwrap();
    let port = redis.get_host_port_ipv4(REDIS_PORT).await.unwrap();
    let redis_url = format!("redis://{}:{}", host, port);

    let state = build_state(&redis_url, None).await;
    let (_handle, base_url) = start_test_server(state).await;

    let resp = reqwest::get(format!("{}/api/document/some-doc", base_url))
        .await
        .unwrap();
    // No secret configured = dev mode = no auth check.
    assert_ne!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "Dev mode should not reject requests"
    );
}
