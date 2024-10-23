use std::{net::SocketAddr, sync::Arc};

use super::errors::{Result, WsError};
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
use serde::{Deserialize, Serialize};
use tracing::{debug, error, trace};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "tag", content = "content")]
enum Event {
    Join { room_id: String },
    Leave,
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
        println!("pinned to {addr}");
    } else {
        println!("couldn't ping to {addr}");
        return;
    }

    // TODO: authentication
    if token != "nyaan" {
        return;
    }

    debug!("{:?}", state.make_room(room_id.clone()));
    if let Err(e) = state.join(&room_id).await {
        debug!("Failed to join room: {:?}", e);
        return;
    }

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
                    return;
                }
            }
        } else {
            println!("client {addr} disconnected");
        }
    }
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
                Event::Join { room_id } => state.join(&room_id).await?,
                Event::Leave => state.leave(room_id).await?,
                Event::UpdateClientCount { count } => {
                    state.update_client_count(room_id, count).await?
                }
                Event::ClientDisconnected => state.handle_client_disconnected(room_id).await?,
            };
            Ok(None)
        }
        Message::Binary(d) => {
            trace!("{} sent {} bytes: {:?}", addr, d.len(), d);
            // Handle binary messages if needed
            Ok(None)
        }
        Message::Close(c) => {
            if let Some(cf) = c {
                println!(
                    ">>> {} sent close with code {} and reason `{}`",
                    addr, cf.code, cf.reason
                );
            } else {
                println!(">>> {addr} somehow sent close message without CloseFrame");
            }
            Ok(None)
        }
        // reply to ping automatically
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

impl AppState {
    async fn _on_disconnect(&self) {
        unimplemented!()
    }

    async fn join(&self, room_id: &str) -> Result<()> {
        let mut rooms = self.rooms.try_lock()?;
        let room = rooms
            .get_mut(room_id)
            .ok_or_else(|| WsError::RoomNotFound(room_id.to_string()))?;
        room.join("brabrabra".to_string()).await?;
        Ok(())
    }

    async fn leave(&self, _room_id: &str) -> Result<()> {
        unimplemented!()
    }

    async fn emit(&self, _data: &str) -> Result<()> {
        unimplemented!()
    }

    async fn _timeout(&self) -> Result<()> {
        unimplemented!()
    }

    async fn update_client_count(&self, room_id: &str, count: usize) -> Result<()> {
        let edit_session_service = self.edit_session_service.clone();
        // Create and process task data
        let task_data = ManageProjectEditSessionTaskData {
            project_id: room_id.to_string(),
            session_id: self.session_id.clone(),
            clients_count: Some(count),
            clients_disconnected_at: None,
            last_merged_at: None,
            last_snapshot_at: None,
        };
        edit_session_service.process(task_data).await?;
        Ok(())
    }

    async fn handle_client_disconnected(&self, room_id: &str) -> Result<()> {
        let edit_session_service = self.edit_session_service.clone();
        let task_data = ManageProjectEditSessionTaskData {
            project_id: room_id.to_string(),
            session_id: self.session_id.clone(),
            clients_count: Some(0),
            clients_disconnected_at: Some(chrono::Utc::now()),
            last_merged_at: None,
            last_snapshot_at: None,
        };
        edit_session_service.process(task_data).await?;
        Ok(())
    }
}
