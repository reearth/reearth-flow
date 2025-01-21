use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        Path,
    },
    response::Response,
    routing::get,
    Router,
};
use std::sync::Arc;
use websocket::broadcast_pool::BroadcastPool;
use websocket::{broadcast::BroadcastGroup, conf::Config};
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

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_file(true)
        .with_line_number(true)
        .init();

    let config = Config::load().unwrap_or_else(|err| {
        eprintln!("Failed to load config: {}, using defaults", err);
        Config::default()
    });

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

    let app = Router::new()
        .route("/:doc_id", get(ws_handler))
        .with_state(pool);

    tracing::info!("Starting server on 0.0.0.0:8080");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(doc_id): Path<String>,
    axum::extract::State(pool): axum::extract::State<Arc<BroadcastPool>>,
) -> Response {
    let doc_id = if doc_id.ends_with(":main") {
        doc_id[..doc_id.len() - 5].to_string()
    } else {
        doc_id
    };

    let bcast = pool.get_or_create_group(&doc_id).await;

    ws.on_upgrade(move |socket| handle_socket(socket, bcast, doc_id, pool))
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
