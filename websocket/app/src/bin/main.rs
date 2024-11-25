use axum::http::{self, HeaderValue};
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::CorsLayer;
use tracing::error;

use app::create_router;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use {app::state::AppState, app::Config};

const CONFIG_FILE_PATH: &str = "./conf/conf.yaml";

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from .env file if it exists
    dotenv::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "trace,tower_http=debug,".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());

    // Load configuration from environment
    let config = Config::from_file(CONFIG_FILE_PATH, &env);

    let state: Arc<AppState> = Arc::new(
        AppState::new(config.redis_url.clone())
            .await
            .expect("Failed to create AppState"),
    );

    // Add CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(
            config
                .allowed_origins
                .iter()
                .filter_map(|origin| {
                    origin
                        .parse::<HeaderValue>()
                        .map_err(|e| {
                            error!("Invalid origin {}: {}", origin, e);
                            e
                        })
                        .ok()
                })
                .collect::<Vec<_>>(),
        )
        .allow_methods([
            http::Method::GET,
            http::Method::POST,
            http::Method::PUT,
            http::Method::DELETE,
            http::Method::OPTIONS,
        ])
        .allow_headers([
            http::header::CONTENT_TYPE,
            http::header::AUTHORIZATION,
            http::header::UPGRADE,
            http::header::CONNECTION,
            http::header::SEC_WEBSOCKET_KEY,
            http::header::SEC_WEBSOCKET_VERSION,
            http::header::SEC_WEBSOCKET_PROTOCOL,
        ])
        .allow_credentials(true);

    let app = create_router(state).layer(cors);
    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
}
