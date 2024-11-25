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
    let command = match command {
        SessionCommand::Start { .. } => {
            if let Some(pid) = project_id {
                SessionCommand::Start {
                    project_id: pid,
                    user: user.clone(),
                }
            } else {
                return Ok(());
            }
        }
        SessionCommand::End { .. } => {
            if let Some(pid) = project_id {
                SessionCommand::End {
                    project_id: pid,
                    user: user.clone(),
                }
            } else {
                return Ok(());
            }
        }
        SessionCommand::Complete { .. } => {
            if let Some(pid) = project_id {
                SessionCommand::Complete {
                    project_id: pid,
                    user: user.clone(),
                }
            } else {
                return Ok(());
            }
        }
        SessionCommand::CheckStatus { .. } => {
            if let Some(pid) = project_id {
                SessionCommand::CheckStatus { project_id: pid }
            } else {
                return Ok(());
            }
        }
        SessionCommand::AddTask { .. } => {
            if let Some(pid) = project_id {
                SessionCommand::AddTask { project_id: pid }
            } else {
                return Ok(());
            }
        }
        SessionCommand::RemoveTask { .. } => {
            if let Some(pid) = project_id {
                SessionCommand::RemoveTask { project_id: pid }
            } else {
                return Ok(());
            }
        }
        SessionCommand::ListAllSnapshotsVersions { .. } => {
            if let Some(pid) = project_id {
                SessionCommand::ListAllSnapshotsVersions { project_id: pid }
            } else {
                return Ok(());
            }
        }
        SessionCommand::MergeUpdates { data, .. } => {
            if let Some(pid) = project_id {
                SessionCommand::MergeUpdates {
                    project_id: pid,
                    data,
                    updated_by: Some(user.id.clone()),
                }
            } else {
                return Ok(());
            }
        }
        SessionCommand::ProcessStateVector { state_vector, .. } => {
            if let Some(pid) = project_id {
                SessionCommand::ProcessStateVector {
                    project_id: pid,
                    state_vector,
                }
            } else {
                return Ok(());
            }
        }
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
    };

    state.command_tx.send(command).map_err(WsError::from)?;
    Ok(())
}
