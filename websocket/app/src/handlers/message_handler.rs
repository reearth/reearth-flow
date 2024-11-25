use crate::{errors::WsError, state::AppState};
use axum::extract::ws::Message;
use flow_websocket_infra::types::user::User;
use flow_websocket_services::manage_project_edit_session::SessionCommand;
use std::{net::SocketAddr, sync::Arc};
use tracing::{debug, warn};

use super::{
    room_handler::{handle_room_event, handle_session_command},
    types::{parse_message, FlowMessage, MessageType},
};

pub async fn handle_message(
    msg: Message,
    addr: SocketAddr,
    room_id: &str,
    project_id: Option<String>,
    state: Arc<AppState>,
    user: User,
) -> Result<Option<Message>, WsError> {
    match msg {
        Message::Text(t) => {
            let msg: FlowMessage = serde_json::from_str(&t)?;

            if let Some(command) = msg.session_command {
                handle_session_command(command, project_id, &user, &state).await?;
            } else {
                handle_room_event(&msg.event, room_id, &state, &user).await?;
            }
            Ok(None)
        }
        Message::Binary(d) => {
            debug!("{} sent {} bytes: {:?}", addr, d.len(), d);

            if let Some(project_id) = project_id {
                if let Some((msg_type, payload)) = parse_message(&d) {
                    match msg_type {
                        MessageType::UPDATE => {
                            state.command_tx.send(SessionCommand::MergeUpdates {
                                project_id: project_id.clone(),
                                data: payload.to_vec(),
                                updated_by: Some(user.id.clone()),
                            })?;
                        }
                        MessageType::SYNC => {
                            state.command_tx.send(SessionCommand::ProcessStateVector {
                                project_id: project_id.clone(),
                                state_vector: payload.to_vec(),
                            })?;
                        }
                        _ => {
                            warn!("Unsupported message type: {:?}", msg_type);
                        }
                    }
                } else {
                    warn!("Invalid binary message format from {}", addr);
                }
            }
            Ok(None)
        }
        Message::Close(_) => {
            debug!("Client {addr} sent close message");
            if let Some(project_id) = project_id {
                state.command_tx.send(SessionCommand::End {
                    project_id: project_id.clone(),
                    user: user.clone(),
                })?;
            }
            Ok(None)
        }
        _ => Ok(None),
    }
}
