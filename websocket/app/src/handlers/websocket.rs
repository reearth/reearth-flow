use std::time::Duration;
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
use futures_util::{SinkExt, StreamExt};
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::time::interval;
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

pub async fn handle_socket(
    socket: WebSocket,
    addr: SocketAddr,
    room_id: String,
    state: Arc<AppState>,
    project_id: Option<String>,
    user: User,
) {
    let (sender, mut receiver) = socket.split();
    let sender = Arc::new(Mutex::new(sender));
    let is_cleaning_up = Arc::new(AtomicBool::new(false));

    // Create cleanup channel
    let (cleanup_tx, mut cleanup_rx) = mpsc::channel(1);

    let cleanup = {
        let is_cleaning_up = is_cleaning_up.clone();
        let room_id = room_id.clone();
        let user = user.clone();
        let project_id = project_id.clone();
        let state = state.clone();

        move || {
            // Ensure cleanup only runs once
            if is_cleaning_up
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                let cleanup_tx = cleanup_tx.clone();
                tokio::spawn(async move {
                    // Leave room
                    if let Err(e) = state.leave(&room_id, &user.id).await {
                        debug!("Cleanup error during leave: {:?}", e);
                    }

                    // Handle project-specific cleanup
                    if let Some(project_id) = project_id {
                        // Send End command
                        if let Err(e) = state
                            .command_tx
                            .send(SessionCommand::End {
                                project_id: project_id.to_string(),
                                user: user.clone(),
                            })
                            .await
                        {
                            debug!("Failed to send End command during cleanup: {:?}", e);
                        }

                        // Send RemoveTask command
                        if let Err(e) = state
                            .command_tx
                            .send(SessionCommand::RemoveTask {
                                project_id: project_id.to_string(),
                            })
                            .await
                        {
                            debug!("Failed to send RemoveTask command during cleanup: {:?}", e);
                        }
                    }

                    // Signal cleanup completion
                    let _ = cleanup_tx.send(()).await;
                });
            }
        }
    };

    // Heartbeat task
    let heartbeat_task = {
        let sender = sender.clone();
        let cleanup = cleanup.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                if sender
                    .lock()
                    .await
                    .send(Message::Ping(vec![1]))
                    .await
                    .is_err()
                {
                    debug!("Ping failed for client {addr}");
                    cleanup();
                    break;
                }
            }
        })
    };

    // Main message loop
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(msg) => {
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
                        if sender
                            .lock()
                            .await
                            .send(Message::Binary(msg.into()))
                            .await
                            .is_err()
                        {
                            debug!("Failed to send message to client {addr}");
                            continue;
                        }
                    }
                    Ok(_) => continue,
                    Err(e) => {
                        debug!("Error handling message: {:?}", e);
                        debug!("client {addr} disconnected");
                        continue;
                    }
                }
            }
            Err(_) => {
                debug!("client {addr} disconnected");
                cleanup();
                break;
            }
        }
    }

    // Wait for cleanup to complete
    let _ = cleanup_rx.recv().await;
    heartbeat_task.abort();
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

            //handle session command
            if let Some(command) = msg.session_command {
                state.command_tx.send(command).await?;
            } else {
                handle_room_event(&msg.event, room_id, &state, &user).await?;
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

#[inline]
async fn handle_room_event(
    event: &Event,
    room_id: &str,
    state: &Arc<AppState>,
    user: &User,
) -> Result<(), WsError> {
    match event {
        Event::Create { room_id } => state.make_room(room_id.clone()).await?,
        Event::Join { room_id } => state.join(room_id, &user.id).await?,
        Event::Leave => state.leave(room_id, &user.id).await?,
        Event::Emit { data } => state.emit(data).await,
    }
    Ok(())
}
