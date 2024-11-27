use crate::{errors::MessageHandlerError, state::AppState};
use axum::extract::ws::Message;
use flow_websocket_infra::types::user::User;
use flow_websocket_services::{ProjectServiceError, SessionCommand};
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
) -> Result<(), MessageHandlerError> {
    match msg {
        Message::Text(t) => {
            let msg: FlowMessage = serde_json::from_str(&t)?;
            if let Some(command) = msg.session_command {
                let result =
                    handle_session_command(command.clone(), conn_state, &user, &state).await?;
                if matches!(command, SessionCommand::End { .. }) {
                    conn_state.start_cleanup();
                }
                if let Some(data) = result {
                    conn_state.send_message(Message::Binary(data)).await?;
                }
            } else {
                handle_room_event(&msg.event, room_id, &state, &user).await?;
            }
        }
        Message::Binary(d) => {
            debug!("{} sent {} bytes: {:?}", addr, d.len(), d);
            if let Some(response) = process_binary_message(d, conn_state, &user, &state).await? {
                debug!("Sending response to client {addr} {:?}", response);
                conn_state.send_message(response).await?;
            }
        }
        Message::Close(_) => {
            debug!("Client {addr} sent close message");
            conn_state.start_cleanup();
        }
        Message::Ping(data) => {
            conn_state.send_message(Message::Pong(data)).await?;
        }
        Message::Pong(_) => {}
    }
    Ok(())
}

async fn process_binary_message(
    data: Vec<u8>,
    conn_state: &ConnectionState,
    user: &User,
    state: &Arc<AppState>,
) -> Result<Option<Message>, ProjectServiceError> {
    if let Some((msg_type, payload)) = parse_message(&data) {
        let project_id = conn_state.current_project_id.lock().await.clone();
        if let Some(pid) = project_id {
            match msg_type {
                MessageType::Update => {
                    let command = SessionCommand::MergeUpdates {
                        project_id: pid,
                        data: payload.to_vec(),
                        updated_by: Some(user.id.clone()),
                    };
                    let result = handle_session_command(command, conn_state, user, state).await?;
                    Ok(result.map(Message::Binary))
                }
                MessageType::Sync => {
                    let command = SessionCommand::ProcessStateVector {
                        project_id: pid,
                        state_vector: payload.to_vec(),
                    };

                    let result = handle_session_command(command, conn_state, user, state).await?;
                    Ok(result.map(Message::Binary))
                }
            }
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}
