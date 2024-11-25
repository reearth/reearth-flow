use flow_websocket_infra::types::user::User;
use flow_websocket_services::manage_project_edit_session::SessionCommand;

use crate::{errors::WsError, state::AppState};
use std::sync::Arc;

use super::types::Event;

pub async fn handle_room_event(
    event: &Event,
    room_id: &str,
    state: &Arc<AppState>,
    user: &User,
) -> Result<(), WsError> {
    match event {
        Event::Create { room_id } => {
            state.make_room(room_id.clone()).await?;
        }
        Event::Join { room_id } => {
            state.join(room_id, &user.id).await?;
        }
        Event::Leave => {
            state.leave(room_id, &user.id).await?;
        }
        Event::Emit { data } => {
            state.emit(data).await;
        }
    }
    Ok(())
}

pub async fn handle_session_command(
    command: SessionCommand,
    project_id: Option<String>,
    user: &User,
    state: &Arc<AppState>,
) -> Result<(), WsError> {
    let command = if let Some(pid) = project_id {
        match command {
            SessionCommand::Start { .. } => SessionCommand::Start {
                project_id: pid.clone(),
                user: user.clone(),
            },
            SessionCommand::End { .. } => SessionCommand::End {
                project_id: pid.clone(),
                user: user.clone(),
            },
            SessionCommand::Complete { .. } => SessionCommand::Complete {
                project_id: pid.clone(),
                user: user.clone(),
            },
            SessionCommand::CheckStatus { .. } => SessionCommand::CheckStatus {
                project_id: pid.clone(),
            },
            SessionCommand::AddTask { .. } => SessionCommand::AddTask {
                project_id: pid.clone(),
            },
            SessionCommand::RemoveTask { .. } => SessionCommand::RemoveTask {
                project_id: pid.clone(),
            },
            SessionCommand::ListAllSnapshotsVersions { .. } => {
                SessionCommand::ListAllSnapshotsVersions {
                    project_id: pid.clone(),
                }
            }
            SessionCommand::MergeUpdates { data, .. } => SessionCommand::MergeUpdates {
                project_id: pid.clone(),
                data,
                updated_by: Some(user.id.clone()),
            },
            SessionCommand::ProcessStateVector { state_vector, .. } => {
                SessionCommand::ProcessStateVector {
                    project_id: pid.clone(),
                    state_vector,
                }
            }
            _ => command,
        }
    } else {
        match command {
            SessionCommand::CreateWorkspace { workspace } => {
                SessionCommand::CreateWorkspace { workspace }
            }
            SessionCommand::DeleteWorkspace { workspace_id } => {
                SessionCommand::DeleteWorkspace { workspace_id }
            }
            SessionCommand::UpdateWorkspace { workspace } => {
                SessionCommand::UpdateWorkspace { workspace }
            }
            SessionCommand::ListWorkspaceProjectsIds { workspace_id } => {
                SessionCommand::ListWorkspaceProjectsIds { workspace_id }
            }
            SessionCommand::CreateProject { project } => SessionCommand::CreateProject { project },
            SessionCommand::DeleteProject { project_id } => {
                SessionCommand::DeleteProject { project_id }
            }
            SessionCommand::UpdateProject { project } => SessionCommand::UpdateProject { project },
            _ => {
                return Ok(());
            }
        }
    };

    state.command_tx.send(command).map_err(WsError::from)?;
    Ok(())
}
