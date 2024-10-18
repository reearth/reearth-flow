use std::{net::SocketAddr, sync::Arc};

use app::create_router;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use {app::state::AppState, app::Config};

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

    // Load configuration from environment
    let config = Config::from_env().expect("Failed to load configuration");

    let state: Arc<AppState> = Arc::new(
        AppState::new(&config.redis_url)
            .await
            .expect("Failed to create AppState"),
    );
    let app = create_router(state);
    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
}
