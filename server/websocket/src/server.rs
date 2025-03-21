use axum::{
    body::Body,
    extract::{Path, State, WebSocketUpgrade},
    http::{Response, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};

use google_cloud_storage::{
    client::Client,
    http::buckets::insert::{BucketCreationConfig, InsertBucketRequest},
};
use std::{io::Cursor, sync::Arc};
use thrift::{
    protocol::{TBinaryInputProtocol, TBinaryOutputProtocol},
    server::TProcessor,
    transport::{TFramedReadTransport, TFramedWriteTransport},
};
use tokio::net::TcpListener;
use tracing::{debug, error, info};

#[cfg(feature = "auth")]
use crate::AuthQuery;
use crate::{doc::DocumentHandler, AppState};
#[cfg(feature = "auth")]
use axum::extract::Query;

#[derive(Clone)]
struct ServerState {
    app_state: Arc<AppState>,
    processor: Arc<dyn TProcessor + Send + Sync>,
}

pub async fn ensure_bucket(client: &Client, bucket_name: &str) -> Result<(), anyhow::Error> {
    let bucket = BucketCreationConfig {
        location: "US".to_string(),
        ..Default::default()
    };
    let request = InsertBucketRequest {
        name: bucket_name.to_string(),
        bucket,
        ..Default::default()
    };

    match client.insert_bucket(&request).await {
        Ok(_) => Ok(()),
        Err(e) if e.to_string().contains("already exists") => Ok(()),
        Err(e) => Err(e.into()),
    }
}

pub async fn start_server(state: Arc<AppState>, port: &str) -> Result<(), anyhow::Error> {
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;

    info!("Starting server on {}", addr);

    let store = state.pool.get_store();

    let handler = DocumentHandler::new(store);
    let processor = Arc::new(handler.processor());

    let server_state = ServerState {
        app_state: state,
        processor,
    };

    let app = Router::new()
        .route("/{doc_id}", get(ws_handler))
        .route(
            "/doc",
            get(|| async { "Document API - Use HTTP POST for Thrift requests" }),
        )
        .route("/doc", post(handle_thrift_request))
        .with_state(server_state);

    info!("WebSocket endpoint available at ws://{}/[doc_id]", addr);
    info!("Thrift HTTP endpoint available at http://{}/doc", addr);
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(feature = "auth")]
async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(doc_id): Path<String>,
    Query(query): Query<AuthQuery>,
    State(state): State<ServerState>,
) -> Response<Body> {
    crate::ws::ws_handler(ws, Path(doc_id), Query(query), State(state.app_state)).await
}

#[cfg(not(feature = "auth"))]
async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(doc_id): Path<String>,
    State(state): State<ServerState>,
) -> Response<Body> {
    crate::ws::ws_handler(ws, Path(doc_id), State(state.app_state)).await
}

async fn handle_thrift_request(
    State(state): State<ServerState>,
    request: axum::extract::Request,
) -> impl IntoResponse {
    let headers = request.headers().clone();
    debug!("Received Thrift request with headers: {:?}", headers);

    let body_bytes = match axum::body::to_bytes(request.into_body(), usize::MAX).await {
        Ok(bytes) => {
            debug!("Received request body of size: {} bytes", bytes.len());
            bytes
        }
        Err(e) => {
            error!("Failed to read request body: {}", e);
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from("Failed to read request body"))
                .unwrap();
        }
    };

    let mut input_buffer = Cursor::new(body_bytes.to_vec());
    let mut output_buffer = Cursor::new(Vec::new());

    let i_trans = TFramedReadTransport::new(&mut input_buffer);
    let o_trans = TFramedWriteTransport::new(&mut output_buffer);

    let mut i_prot = TBinaryInputProtocol::new(i_trans, true);
    let mut o_prot = TBinaryOutputProtocol::new(o_trans, true);

    match state.processor.process(&mut i_prot, &mut o_prot) {
        Ok(_) => {
            let response_data = output_buffer.into_inner();
            debug!("Sending response of size: {} bytes", response_data.len());

            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/x-thrift")
                .body(Body::from(response_data))
                .unwrap()
        }
        Err(e) => {
            error!("Error processing Thrift request: {}", e);
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(format!("Error processing request: {}", e)))
                .unwrap()
        }
    }
}
