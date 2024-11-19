use std::{net::SocketAddr, sync::Arc};

use super::types::{Event, FlowMessage, WebSocketQuery};
use crate::errors::WsError;
use crate::state::AppState;
use axum::extract::{Path, Query};
use axum::response::Response;
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
) -> Response {
    debug!("{:?}", query);

    let user = User::new(query.user_id().to_string(), None, None);

    ws.on_upgrade(move |socket| {
        handle_socket(socket, addr, room_id, state, query.project_id(), user)
    })
    .into_response()
}

async fn handle_socket(
    mut socket: WebSocket,
    addr: SocketAddr,
    room_id: String,
    state: Arc<AppState>,
    project_id: Option<String>,
    user: User,
) {
    if !test_connection(&mut socket, &addr).await {
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

    let cleanup = || {
        let state = state.clone();
        let room_id = room_id.clone();
        let user = user.clone();
        let project_id = project_id.clone();

        tokio::spawn(async move {
            if let Err(e) = state.leave(&room_id, &user.id).await {
                debug!("Cleanup error during leave: {:?}", e);
            }
            if let Some(project_id) = project_id {
                let _ = state
                    .command_tx
                    .send(SessionCommand::End {
                        project_id: project_id.to_string(),
                        user: user.clone(),
                    })
                    .await;
                let _ = state
                    .command_tx
                    .send(SessionCommand::RemoveTask {
                        project_id: project_id.to_string(),
                    })
                    .await;
            }
        });
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
                        cleanup();
                        return;
                    }
                }
                Ok(_) => continue,
                Err(e) => {
                    debug!("Error handling message: {:?}", e);
                    debug!("client {addr} disconnected");
                    cleanup();
                    return;
                }
            }
        } else {
            debug!("client {addr} disconnected");
            cleanup();
            return;
        }
    }
}

async fn test_connection(socket: &mut WebSocket, addr: &SocketAddr) -> bool {
    if socket.send(Message::Ping(vec![4])).await.is_err() {
        debug!("Connection failed for {addr}: ping failed");
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
                Event::Create { room_id } => state.make_room(room_id).await?,
                Event::Join { room_id } => state.join(&room_id, &user.id).await?,
                Event::Leave => state.leave(room_id, &user.id).await?,
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
                        project_id: project_id.to_string(),
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
                            project_id: project_id.to_string(),
                            user: user.clone(),
                        })
                        .await?;

                    state
                        .command_tx
                        .send(SessionCommand::RemoveTask {
                            project_id: project_id.to_string(),
                        })
                        .await?;
                }
            }
            Ok(None)
        }
        _ => Ok(None),
    }
}
