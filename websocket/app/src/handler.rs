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
use flow_websocket_domain::user::User;
use flow_websocket_services::manage_project_edit_session::SessionCommand;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{debug, error, trace};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "tag", content = "content")]
enum Event {
    Join { room_id: String },
    Leave,
    Emit { data: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct FlowMessage {
    event: Event,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WebSocketQuery {
    token: String,
    user_id: String,
    user_email: String,
    user_name: String,
    tenant_id: String,
}

pub async fn handle_upgrade(
    ws: WebSocketUpgrade,
    query: Query<WebSocketQuery>,
    Path(room_id): Path<String>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    debug!("{:?}", query);

    let user = User {
        id: query.user_id.clone(),
        email: query.user_email.clone(),
        name: query.user_name.clone(),
        tenant_id: query.tenant_id.clone(),
    };

    ws.on_upgrade(move |socket| {
        handle_socket(socket, addr, query.token.to_string(), room_id, state, user)
    })
}

async fn handle_socket(
    mut socket: WebSocket,
    addr: SocketAddr,
    token: String,
    room_id: String,
    state: Arc<AppState>,
    user: User,
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
            match handle_message(msg, addr, &room_id, state.clone(), user.clone()).await {
                Ok(Some(msg)) => {
                    let _ = socket.send(Message::Binary(msg.into())).await;
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
    user: User,
) -> Result<Option<Message>> {
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
                Event::Emit { data } => state.emit(&data).await?,
            };
            Ok(None)
        }
        Message::Binary(d) => {
            // TODO: handle binary message
            trace!("{} sent {} bytes: {:?}", addr, d.len(), d);
            if d.len() < 3 {
                return Ok(None);
            };

            let rooms = state.rooms.try_lock()?;
            let _room = rooms
                .get(room_id)
                .ok_or_else(|| WsError::RoomNotFound(room_id.to_string()))?;

            let (tx, _rx) = mpsc::channel(32);

            tx.send(SessionCommand::Start {
                project_id: room_id.to_string(),
                user: user.clone(),
            })
            .await
            .unwrap();
            Ok(None)
        }
        Message::Close(c) => {
            // TODO: handle close message
            if let Some(cf) = c {
                println!(
                    ">>> {} sent close with code {} and reason `{}`",
                    addr, cf.code, cf.reason
                );

                let (tx, rx) = mpsc::channel(32);
                tx.send(SessionCommand::End {
                    project_id: room_id.to_string(),
                    user,
                })
                .await
                .unwrap();
                tokio::spawn({
                    let service = state.service.clone();
                    async move {
                        if let Err(e) = service.process(rx).await {
                            error!("Error processing session end: {:?}", e);
                        }
                    }
                });
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
}
