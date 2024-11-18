use std::{net::SocketAddr, sync::Arc};

use super::types::{Event, FlowMessage, WebSocketQuery};
use crate::errors::WsError;
use crate::state::AppState;
use axum::extract::{Path, Query};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use flow_websocket_infra::types::user::User;
use flow_websocket_services::manage_project_edit_session::SessionCommand;
use tokio::sync::mpsc;
use tracing::{debug, error, trace};

pub async fn handle_upgrade(
    ws: WebSocketUpgrade,
    query: Query<WebSocketQuery>,
    Path(room_id): Path<String>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    debug!("{:?}", query);

    let user = User::new(query.user_id.clone(), None, None);

    ws.on_upgrade(move |socket| {
        handle_socket(
            socket,
            addr,
            query.token.to_string(),
            room_id,
            state,
            query.project_id.clone(),
            user,
        )
    })
}

async fn handle_socket(
    mut socket: WebSocket,
    addr: SocketAddr,
    token: String,
    room_id: String,
    state: Arc<AppState>,
    project_id: Option<String>,
    user: User,
) {
    if !verify_connection(&mut socket, &addr, &token).await {
        return;
    }

    if !initialize_room(&state, &room_id, &user).await {
        return;
    }

    let (_tx, rx) = mpsc::channel(32);

    // Spawn service processor
    tokio::spawn({
        let service = state.service.clone();
        async move {
            if let Err(e) = service.process(rx).await {
                error!("Error processing session commands: {:?}", e);
            }
        }
    });

    let cleanup = || async {
        state.leave(&room_id, &user.id).await;
        if let Some(project_id) = project_id.clone() {
            let _ = state
                .command_tx
                .send(SessionCommand::End {
                    project_id: project_id.clone(),
                    user: user.clone(),
                })
                .await;
            let _ = state
                .command_tx
                .send(SessionCommand::RemoveTask { project_id })
                .await;
        }
    };

    while let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            match handle_message(
                msg,
                addr,
                &room_id,
                project_id.clone(),
                state.clone(),
                user.clone(),
            )
            .await
            {
                Ok(Some(msg)) => {
                    if socket.send(Message::Binary(msg.into())).await.is_err() {
                        debug!("Failed to send message to client {addr}");
                        cleanup().await;
                        return;
                    }
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
}

async fn verify_connection(socket: &mut WebSocket, addr: &SocketAddr, token: &str) -> bool {
    if socket.send(Message::Ping(vec![4])).await.is_err() || token != "nyaan" {
        debug!("Connection failed for {addr}: ping failed or invalid token");
        return false;
    }
    true
}

async fn initialize_room(state: &Arc<AppState>, room_id: &str, user: &User) -> bool {
    match state.make_room(room_id.to_string()) {
        Ok(_) => debug!("Room created/exists: {}", room_id),
        Err(e) => {
            debug!("Failed to create room: {:?}", e);
            return false;
        }
    }

    if let Err(e) = state.join(room_id, &user.id).await {
        debug!("Failed to join room: {:?}", e);
        return false;
    }

    true
}

async fn handle_message(
    msg: Message,
    addr: SocketAddr,
    room_id: &str,
    project_id: Option<String>,
    state: Arc<AppState>,
    user: User,
) -> Result<Option<Message>, WsError> {
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
                Event::Join { room_id } => state.join(&room_id, &user.id).await?,
                Event::Leave => state.leave(room_id, &user.id).await,
                Event::Emit { data } => state.emit(&data).await,
            };

            if let Some(command) = msg.session_command {
                state.command_tx.send(command).await?;
            }

            Ok(None)
        }
        Message::Binary(d) => {
            trace!("{} sent {} bytes: {:?}", addr, d.len(), d);
            if d.len() < 3 {
                return Ok(None);
            };

            let rooms = state.rooms.try_lock()?;
            let _room = rooms
                .get(room_id)
                .ok_or_else(|| WsError::RoomNotFound(room_id.to_string()))?;

            if let Some(project_id) = project_id {
                state
                    .command_tx
                    .send(SessionCommand::MergeUpdates {
                        project_id,
                        data: d,
                        updated_by: Some(user.id.clone()),
                    })
                    .await?;
            }

            Ok(None)
        }
        Message::Close(c) => {
            if let Some(cf) = c {
                debug!(
                    ">>> {} sent close with code {} and reason `{}`",
                    addr, cf.code, cf.reason
                );

                if let Some(project_id) = project_id {
                    state
                        .command_tx
                        .send(SessionCommand::End {
                            project_id: project_id.clone(),
                            user: user.clone(),
                        })
                        .await?;

                    state
                        .command_tx
                        .send(SessionCommand::RemoveTask { project_id })
                        .await?;
                }
            }
            Ok(None)
        }
        _ => Ok(None),
    }
}
