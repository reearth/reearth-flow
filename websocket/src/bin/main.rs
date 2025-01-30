use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        Path,
    },
    response::Response,
    routing::get,
    Router,
};

#[cfg(feature = "auth")]
use axum::body::Bytes;
#[cfg(feature = "auth")]
use axum::extract::Query;
#[cfg(feature = "auth")]
use axum::response::IntoResponse;
#[cfg(feature = "auth")]
use serde::Deserialize;
use std::sync::Arc;
#[cfg(feature = "auth")]
use websocket::auth::AuthService;
use websocket::conf::Config;
use websocket::group::BroadcastGroup;
use websocket::pool::BroadcastPool;
//use websocket::storage::sqlite::SqliteStore;
use google_cloud_storage::{
    client::Client,
    http::buckets::insert::{BucketCreationConfig, InsertBucketRequest},
};
use websocket::storage::gcs::GcsStore;
use websocket::ws::WarpConn;

//const DB_PATH: &str = "examples/code-mirror/yrs.db";
//const REDIS_URL: &str = "redis://127.0.0.1:6379";
//const REDIS_TTL: u64 = 3600; // Cache TTL in seconds
const BUCKET_NAME: &str = "yrs-dev";
const PORT: &str = "8000";

#[cfg(feature = "auth")]
#[derive(Debug, Deserialize)]
struct AuthQuery {
    #[serde(default)]
    token: String,
}

async fn ensure_bucket(client: &Client) -> Result<(), anyhow::Error> {
    let bucket = BucketCreationConfig {
        location: "US".to_string(),
        ..Default::default()
    };
    let request = InsertBucketRequest {
        name: BUCKET_NAME.to_string(),
        bucket,
        ..Default::default()
    };

    match client.insert_bucket(&request).await {
        Ok(_) => Ok(()),
        Err(e) if e.to_string().contains("already exists") => Ok(()),
        Err(e) => Err(e.into()),
    }
}

#[cfg(feature = "auth")]
#[derive(Clone)]
struct AppState {
    pool: Arc<BroadcastPool>,
    auth: Arc<AuthService>,
}

#[cfg(not(feature = "auth"))]
#[derive(Clone)]
struct AppState {
    pool: Arc<BroadcastPool>,
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(doc_id): Path<String>,
    #[cfg(feature = "auth")] Query(auth): Query<AuthQuery>,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Response {
    let doc_id = if doc_id.ends_with(":main") {
        doc_id[..doc_id.len() - 5].to_string()
    } else {
        doc_id.clone()
    };

    #[cfg(feature = "auth")]
    {
        match state.auth.verify_token(&auth.token).await {
            Ok(true) => (),
            Ok(false) => {
                tracing::warn!(
                    "Authentication failed for doc_id: {}, token: {}",
                    doc_id,
                    auth.token
                );
                return Bytes::from("Unauthorized").into_response();
            }
            Err(e) => {
                tracing::error!("Authentication error: {}", e);
                return Bytes::from("Internal Server Error").into_response();
            }
        }
    }

    let bcast = state.pool.get_or_create_group(&doc_id).await;
    ws.on_upgrade(move |socket| handle_socket(socket, bcast, doc_id, state.pool.clone()))
}

async fn handle_socket(
    socket: WebSocket,
    bcast: Arc<BroadcastGroup>,
    doc_id: String,
    pool: Arc<BroadcastPool>,
) {
    bcast.increment_connections();
    let conn = WarpConn::new(bcast, socket);
    if let Err(e) = conn.await {
        tracing::error!("WebSocket connection error: {}", e);
    }
    pool.remove_connection(&doc_id).await;
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_file(true)
        .with_line_number(true)
        .init();

    let config = Config::load();

    // Initialize SQLite store
    //let store = Arc::new(SqliteStore::new(DB_PATH).expect("Failed to open SQLite database"));
    //tracing::info!("SQLite store initialized at: {}", DB_PATH);

    let store = GcsStore::new_with_config(config.gcs)
        .await
        .expect("Failed to create GCS store");

    // Ensure bucket exists
    ensure_bucket(&store.client)
        .await
        .expect("Failed to create bucket");

    let store = Arc::new(store);
    tracing::info!("GCS store initialized");

    // Create broadcast pool
    let pool = Arc::new(BroadcastPool::new(store, config.redis));
    tracing::info!("Broadcast pool initialized");

    let state = {
        #[cfg(feature = "auth")]
        {
            let auth = Arc::new(AuthService::new(config.auth));
            tracing::info!("Auth service initialized");
            AppState { pool, auth }
        }
        #[cfg(not(feature = "auth"))]
        {
            AppState { pool }
        }
    };

    let app = Router::new()
        .route("/:doc_id", get(ws_handler))
        .with_state(state);

    tracing::info!("Starting server on 0.0.0.0:{}", PORT);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", PORT))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
