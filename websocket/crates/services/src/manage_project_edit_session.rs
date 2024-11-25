use super::SessionCommand;
use chrono::Utc;
use flow_websocket_infra::persistence::editing_session::ProjectEditingSession;
use flow_websocket_infra::persistence::project_repository::ProjectRepositoryError;
use flow_websocket_infra::persistence::redis::errors::FlowProjectRedisDataManagerError;
use flow_websocket_infra::persistence::repository::{
    ProjectEditingSessionImpl, ProjectImpl, ProjectSnapshotImpl, RedisDataManagerImpl,
    WorkspaceImpl,
};
use flow_websocket_infra::types::user::User;
use mockall::automock;
use std::time::Duration;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::broadcast;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::debug;

use crate::project::ProjectService;
use crate::{types::ManageProjectEditSessionTaskData, ProjectServiceError};

const MAX_EMPTY_SESSION_DURATION: Duration = Duration::from_secs(10);
const JOB_COMPLETION_DELAY: Duration = Duration::from_secs(5);

#[derive(Debug, Clone)]
pub struct ManageEditSessionService<R, S, M, P, W>
where
    R: ProjectEditingSessionImpl<Error = ProjectRepositoryError> + Send + Sync + Clone + 'static,
    S: ProjectSnapshotImpl<Error = ProjectRepositoryError> + Send + Sync + Clone + 'static,
    M: RedisDataManagerImpl<Error = FlowProjectRedisDataManagerError>
        + Send
        + Sync
        + Clone
        + 'static,
    P: ProjectImpl<Error = ProjectRepositoryError> + Send + Sync + Clone + 'static,
    W: WorkspaceImpl<Error = ProjectRepositoryError> + Send + Sync + Clone + 'static,
{
    pub project_service: ProjectService<R, S, M, P, W>,
    tasks: Arc<Mutex<HashMap<String, ManageProjectEditSessionTaskData>>>,
}

#[automock]
impl<R, S, M, P, W> ManageEditSessionService<R, S, M, P, W>
where
    R: ProjectEditingSessionImpl<Error = ProjectRepositoryError> + Send + Sync + Clone + 'static,
    S: ProjectSnapshotImpl<Error = ProjectRepositoryError> + Send + Sync + Clone + 'static,
    M: RedisDataManagerImpl<Error = FlowProjectRedisDataManagerError>
        + Send
        + Sync
        + Clone
        + 'static,
    P: ProjectImpl<Error = ProjectRepositoryError> + Send + Sync + Clone + 'static,
    W: WorkspaceImpl<Error = ProjectRepositoryError> + Send + Sync + Clone + 'static,
{
    pub fn new(
        session_repository: Arc<R>,
        snapshot_repository: Arc<S>,
        redis_data_manager: Arc<M>,
        project_repository: Arc<P>,
        workspace_repository: Arc<W>,
    ) -> Self {
        Self {
            project_service: ProjectService::new(
                session_repository,
                snapshot_repository,
                redis_data_manager,
                project_repository,
                workspace_repository,
            ),
            tasks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn process(&self, mut command_rx: broadcast::Receiver<SessionCommand>) {
        loop {
            tokio::select! {
                result = command_rx.recv() => {
                    if let Err(e) = self.handle_command(result).await {
                        debug!("Error handling command: {:?}", e);
                    }
                },
                _ = tokio::time::sleep(Duration::from_secs(1)) => {
                    if let Err(e) = self.check_tasks_conditions().await {
                        debug!("Error checking task conditions: {:?}", e);
                    }
                }
            }
        }
    }

    async fn handle_command(
        &self,
        result: Result<SessionCommand, broadcast::error::RecvError>,
    ) -> Result<Option<Vec<u8>>, ProjectServiceError> {
        match result {
            Ok(command) => match command {
                SessionCommand::Start { project_id, user } => {
                    self.handle_session_start(&project_id, user).await?;
                    Ok(None)
                }
                SessionCommand::End { project_id, user } => {
                    self.handle_session_end(&project_id, user).await?;
                    Ok(None)
                }
                SessionCommand::Complete { project_id, user } => {
                    if let Some(mut session) = self.get_latest_session(&project_id).await? {
                        self.complete_job_if_met_requirements(&mut session).await?;
                        debug!(
                            "Job completed by user: {} for project: {}",
                            user.id, project_id
                        );
                    }
                    Ok(None)
                }
                SessionCommand::MergeUpdates {
                    project_id,
                    data,
                    updated_by,
                } => {
                    self.project_service
                        .merge_updates(&project_id, data, updated_by)
                        .await?;
                    Ok(None)
                }
                SessionCommand::ProcessStateVector {
                    project_id,
                    state_vector,
                } => {
                    let updates = self
                        .project_service
                        .process_state_vector(&project_id, state_vector)
                        .await?;
                    debug!("Processed state vector for project: {}", project_id);
                    Ok(updates)
                }
                SessionCommand::CheckStatus { project_id } => {
                    debug!("Checking session status for project: {}", project_id);
                    Ok(None)
                }
                SessionCommand::AddTask { project_id } => {
                    self.add_task(&project_id).await?;
                    Ok(None)
                }
                SessionCommand::RemoveTask { project_id } => {
                    self.remove_task(&project_id).await?;
                    Ok(None)
                }
                SessionCommand::ListAllSnapshotsVersions { project_id } => {
                    let versions = self
                        .project_service
                        .list_all_snapshots_versions(&project_id)
                        .await?;
                    debug!(
                        "Snapshots versions for project {}: {:?}",
                        project_id, versions
                    );
                    Ok(None)
                }
                // Workspace related commands
                SessionCommand::CreateWorkspace { workspace } => {
                    self.project_service.create_workspace(workspace).await?;
                    Ok(None)
                }
                SessionCommand::DeleteWorkspace { workspace_id } => {
                    self.project_service.delete_workspace(&workspace_id).await?;
                    Ok(None)
                }
                SessionCommand::UpdateWorkspace { workspace } => {
                    self.project_service.update_workspace(workspace).await?;
                    Ok(None)
                }
                SessionCommand::ListWorkspaceProjectsIds { workspace_id } => {
                    let projects = self
                        .project_service
                        .list_workspace_projects_ids(&workspace_id)
                        .await?;
                    debug!("Projects for workspace {}: {:?}", workspace_id, projects);
                    Ok(None)
                }
                // Project related commands
                SessionCommand::CreateProject { project } => {
                    self.project_service.create_project(project).await?;
                    Ok(None)
                }
                SessionCommand::DeleteProject { project_id } => {
                    self.project_service.delete_project(&project_id).await?;
                    Ok(None)
                }
                SessionCommand::UpdateProject { project } => {
                    self.project_service.update_project(project).await?;
                    Ok(None)
                }
            },
            Err(broadcast::error::RecvError::Closed) => {
                debug!("Command channel closed");
                sleep(Duration::from_secs(1)).await;
                Ok(None)
            }
            Err(broadcast::error::RecvError::Lagged(n)) => {
                debug!("Receiver lagged behind by {} messages", n);
                Ok(None)
            }
        }
    }

    async fn check_tasks_conditions(&self) -> Result<(), ProjectServiceError> {
        let tasks = self.tasks.lock().await;
        for (project_id, data) in tasks.iter() {
            if let Some(mut session) = self.get_latest_session(project_id).await? {
                if let Ok(()) = self
                    .end_editing_session_if_conditions_met(&mut session, data)
                    .await
                {
                    debug!(
                        "Session ended by condition check for project: {}",
                        project_id
                    );
                }
            }
        }
        Ok(())
    }

    async fn handle_session_start(
        &self,
        project_id: &str,
        user: User,
    ) -> Result<(), ProjectServiceError> {
        let session = self
            .project_service
            .get_or_create_editing_session(project_id, user.clone())
            .await?;

        if session.session_id.is_some() {
            debug!("Session exists/created for project: {}", project_id);
            if let Some(task_data) = self.get_task_data(project_id).await {
                self.update_client_count(&task_data, true).await;
            }
        }
        Ok(())
    }

    async fn handle_session_end(
        &self,
        project_id: &str,
        _user: User,
    ) -> Result<(), ProjectServiceError> {
        if let Some(task_data) = self.get_task_data(project_id).await {
            self.update_client_count(&task_data, false).await;

            if let Some(mut session) = self.get_latest_session(project_id).await? {
                debug!("Checking if job is complete for project: {}", project_id);
                if let Err(e) = self.complete_job_if_met_requirements(&mut session).await {
                    debug!("Failed to complete job: {:?}", e);
                }
            }
        }
        Ok(())
    }

    async fn update_client_count(
        &self,
        task_data: &ManageProjectEditSessionTaskData,
        is_increment: bool,
    ) {
        let mut count = task_data.client_count.write().await;
        if let Some(current_count) = *count {
            *count = Some(if is_increment {
                current_count + 1
            } else {
                current_count.saturating_sub(1)
            });

            debug!(
                "Client count {} to: {:?}",
                if is_increment {
                    "increased"
                } else {
                    "decreased"
                },
                *count
            );

            if !is_increment && *count == Some(0) {
                let mut disconnected_at = task_data.clients_disconnected_at.write().await;
                *disconnected_at = Some(Utc::now());
                debug!("All clients disconnected at: {:?}", *disconnected_at);
            }
        }
    }

    async fn add_task(&self, project_id: &str) -> Result<(), ProjectServiceError> {
        let task_data = ManageProjectEditSessionTaskData::new(project_id.to_string());
        let mut tasks = self.tasks.lock().await;
        tasks.insert(project_id.to_string(), task_data.clone());
        debug!("Added task for project: {}", project_id);
        Ok(())
    }

    async fn remove_task(&self, project_id: &str) -> Result<(), ProjectServiceError> {
        let mut tasks = self.tasks.lock().await;
        tasks.remove(project_id);
        debug!("Removed task for project: {}", project_id);
        Ok(())
    }

    async fn get_task_data(&self, project_id: &str) -> Option<ManageProjectEditSessionTaskData> {
        let tasks = self.tasks.lock().await;
        tasks.get(project_id).cloned()
    }

    pub async fn get_latest_session(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectEditingSession>, ProjectServiceError> {
        let ret = self
            .project_service
            .session_repository
            .get_active_session(project_id)
            .await?;
        Ok(ret)
    }

    pub async fn end_editing_session_if_conditions_met(
        &self,
        session: &mut ProjectEditingSession,
        data: &ManageProjectEditSessionTaskData,
    ) -> Result<(), ProjectServiceError> {
        if let Some(client_count) = *data.client_count.read().await {
            if client_count == 0 {
                if let Some(clients_disconnected_at) = *data.clients_disconnected_at.read().await {
                    let current_time = Utc::now();
                    let clients_disconnection_elapsed_time = current_time - clients_disconnected_at;

                    if clients_disconnection_elapsed_time
                        .to_std()
                        .map_err(ProjectServiceError::ChronoDurationConversionError)?
                        > MAX_EMPTY_SESSION_DURATION
                    {
                        self.project_service
                            .end_session("system".to_string(), session.clone())
                            .await?;
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn complete_job_if_met_requirements(
        &self,
        session: &mut ProjectEditingSession,
    ) -> Result<(), ProjectServiceError> {
        self.project_service
            .end_session("system".to_string(), session.clone())
            .await?;

        debug!("Job completed for project: {}", session.project_id);

        sleep(JOB_COMPLETION_DELAY).await;

        Ok(())
    }
}
