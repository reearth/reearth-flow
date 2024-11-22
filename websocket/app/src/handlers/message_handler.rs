use crate::{errors::WsError, state::AppState};
use axum::extract::ws::Message;
use flow_websocket_infra::types::user::User;
use flow_websocket_services::manage_project_edit_session::SessionCommand;
use std::{net::SocketAddr, sync::Arc};
use tracing::{debug, trace};

use super::{room_handler::handle_room_event, types::FlowMessage};

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
                state.command_tx.send(command).await?;
            } else {
                handle_room_event(&msg.event, room_id, &state, &user).await?;
            }
            Ok(None)
        }
        Message::Binary(d) => {
            trace!("{} sent {} bytes: {:?}", addr, d.len(), d);
            if d.len() >= 3 {
                if let Some(project_id) = project_id {
                    state
                        .command_tx
                        .send(SessionCommand::MergeUpdates {
                            project_id: project_id.clone(),
                            data: d,
                            updated_by: Some(user.id.clone()),
                        })
                        .await?;
                }
            }
            Ok(None)
        }
        Message::Close(_) => {
            debug!("Client {addr} sent close message");
            if let Some(project_id) = project_id {
                state
                    .command_tx
                    .send(SessionCommand::End {
                        project_id: project_id.clone(),
                        user: user.clone(),
                    })
                    .await?;
            }
            Ok(None)
        }
        _ => Ok(None),
    }
}
