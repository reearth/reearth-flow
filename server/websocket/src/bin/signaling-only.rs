use axum::{
    body::Body,
    extract::{State, WebSocketUpgrade},
    http::Response,
    routing::get,
    Router,
};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

use websocket::interface::websocket::signaling::{handle_signaling_connection, SignalingService};

#[derive(Clone)]
struct SignalingState {
    signaling: SignalingService,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_file(true)
        .with_line_number(true)
        .init();

    let port = std::env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    let addr = format!("0.0.0.0:{port}");
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind to {}: {}", addr, e);
            std::process::exit(1);
        }
    };

    info!("Starting WebRTC signaling server on {}", addr);

    let signaling = SignalingService::new();
    let state = SignalingState { signaling };

    let app = Router::new()
        .route("/signaling", get(signaling_handler))
        .with_state(state)
        .layer(
            ServiceBuilder::new().layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any),
            ),
        );

    info!(
        "WebRTC Signaling endpoint available at ws://{}/signaling",
        addr
    );
    info!("Ready to accept WebRTC connections!");

    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("Server error: {}", e);
        std::process::exit(1);
    }
}

async fn signaling_handler(
    ws: WebSocketUpgrade,
    State(state): State<SignalingState>,
) -> Response<Body> {
    let signaling = state.signaling.clone();
    ws.on_upgrade(move |socket| async move {
        if let Err(e) = handle_signaling_connection(socket, signaling).await {
            eprintln!("Signaling connection error: {:?}", e);
        }
    })
}
