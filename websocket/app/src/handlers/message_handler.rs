use crate::{errors::WsError, state::AppState};
use axum::extract::ws::Message;
use flow_websocket_infra::types::user::User;
use flow_websocket_services::SessionCommand;
use std::{net::SocketAddr, sync::Arc};
use tracing::debug;

use super::{
    room_handler::{handle_room_event, handle_session_command},
    socket_handler::ConnectionState,
    types::{parse_message, FlowMessage, MessageType},
};

pub async fn handle_message(
    msg: Message,
    addr: SocketAddr,
    room_id: &str,
    conn_state: &ConnectionState,
    state: Arc<AppState>,
    user: User,
) -> Result<Option<Message>, WsError> {
    match msg {
        Message::Text(t) => {
            let msg: FlowMessage = serde_json::from_str(&t)?;

            if let Some(command) = msg.session_command {
                handle_session_command(command.clone(), conn_state, &user, &state).await?;
                if matches!(command, SessionCommand::End { .. }) {
                    conn_state.start_cleanup();
                }
            } else {
                handle_room_event(&msg.event, room_id, &state, &user).await?;
            }
            Ok(None)
        }
        Message::Binary(d) => {
            debug!("{} sent {} bytes: {:?}", addr, d.len(), d);

            if let Some(response) = process_binary_message(d, conn_state, &user, &state).await? {
                conn_state.send_message(response).await?;
            }
            Ok(None)
        }
        Message::Close(_) => {
            debug!("Client {addr} sent close message");
            conn_state.start_cleanup();
            Ok(None)
        }
        Message::Ping(data) => Ok(Some(Message::Pong(data))),
        Message::Pong(_) => Ok(None),
    }
}

async fn process_binary_message(
    data: Vec<u8>,
    conn_state: &ConnectionState,
    user: &User,
    state: &Arc<AppState>,
) -> Result<Option<Message>, WsError> {
    if let Some((msg_type, payload)) = parse_message(&data) {
        let project_id = conn_state.current_project_id.lock().await.clone();
        if let Some(pid) = project_id {
            match msg_type {
                MessageType::Update => {
                    state.command_tx.send(SessionCommand::MergeUpdates {
                        project_id: pid,
                        data: payload.to_vec(),
                        updated_by: Some(user.id.clone()),
                    })?;
                }
                MessageType::Sync => {
                    state.command_tx.send(SessionCommand::ProcessStateVector {
                        project_id: pid,
                        state_vector: payload.to_vec(),
                    })?;
                }
            }
        }
    }
    Ok(None)
}
