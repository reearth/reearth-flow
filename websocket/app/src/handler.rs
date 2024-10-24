use std::{net::SocketAddr, sync::Arc};

use super::errors::Result;
use super::state::AppState;
use axum::extract::{Path, Query};
use axum::http::{Method, StatusCode, Uri};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use flow_websocket_domain::generate_id;
use flow_websocket_services::types::ManageProjectEditSessionTaskData;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, trace};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "tag", content = "content")]
enum Event {
    UpdateClientCount { count: usize },
    ClientDisconnected,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct FlowMessage {
    event: Event,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WebSocketQuery {
    token: String,
}

pub async fn handle_upgrade(
    ws: WebSocketUpgrade,
    query: Query<WebSocketQuery>,
    Path(room_id): Path<String>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    debug!("{:?}", query);
    ws.on_upgrade(move |socket| {
        handle_socket(socket, addr, query.token.to_string(), room_id, state)
    })
}

async fn handle_socket(
    mut socket: WebSocket,
    addr: SocketAddr,
    token: String,
    room_id: String,
    state: Arc<AppState>,
) {
    if socket.send(Message::Ping(vec![4])).await.is_ok() {
        debug!("pinged to {addr}");
    } else {
        debug!("couldn't ping to {addr}");
        return;
    }

    // TODO: authentication
    if token != "nyaan" {
        debug!("Invalid token");
        return;
    }

    // Get or create session service for this project
    let edit_session_service = match state.get_or_create_session_service(&room_id).await {
        Ok(service) => service,
        Err(e) => {
            debug!("Failed to create session service: {:?}", e);
            return;
        }
    };

    // Initialize connection
    let session_id = generate_id(14, "session");
    let task_data = ManageProjectEditSessionTaskData {
        project_id: room_id.clone(),
        session_id: session_id.clone(),
        clients_count: Some(1),
        clients_disconnected_at: None,
        last_merged_at: None,
        last_snapshot_at: None,
    };

    if let Err(e) = edit_session_service.process(task_data).await {
        debug!("Failed to initialize session: {:?}", e);
        return;
    }

    // Set up cleanup closure
    let cleanup = || async {
        if let Err(e) = state.handle_client_disconnected(&room_id).await {
            debug!("Failed to cleanup session: {:?}", e);
        }
    };

    while let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            match handle_message(msg, addr, &room_id, state.clone()).await {
                Ok(Some(msg)) => {
                    let _ = socket.send(Message::Binary(msg)).await;
                    continue;
                }
                Ok(_) => continue,
                Err(e) => {
                    debug!("Error handling message: {:?}", e);
                    debug!("client {addr} disconnected");
                    cleanup().await;
                    return;
                }
            }
        } else {
            debug!("client {addr} disconnected");
            cleanup().await;
            return;
        }
    }

    // Clean up when connection closes
    cleanup().await;
}

async fn handle_message(
    msg: Message,
    addr: SocketAddr,
    room_id: &str,
    state: Arc<AppState>,
) -> Result<Option<Vec<u8>>> {
    match msg {
        Message::Text(t) => {
            let msg: FlowMessage = match serde_json::from_str(&t) {
                Ok(msg) => msg,
                Err(err) => {
                    error!("Failed to parse message: {:?}", err);
                    return Ok(None);
                }
            };

            match msg.event {
                Event::UpdateClientCount { count } => {
                    state.update_client_count(room_id, count).await?
                }
                Event::ClientDisconnected => state.handle_client_disconnected(room_id).await?,
            };
            Ok(None)
        }
        Message::Binary(d) => {
            trace!("{} sent {} bytes: {:?}", addr, d.len(), d);
            Ok(None)
        }
        Message::Close(c) => {
            if let Some(cf) = c {
                debug!(
                    ">>> {} sent close with code {} and reason `{}`",
                    addr, cf.code, cf.reason
                );
            } else {
                debug!(">>> {addr} somehow sent close message without CloseFrame");
            }
            Ok(None)
        }
        _ => Ok(None),
    }
}

pub async fn handle_error(
    method: Method,
    uri: Uri,
    error: Box<dyn std::error::Error + Send + Sync>,
) -> impl IntoResponse {
    let error_message = format!("Error occurred for request {} {}: {}", method, uri, error);
    tracing::error!(error_message);
    StatusCode::INTERNAL_SERVER_ERROR.into_response()
}
