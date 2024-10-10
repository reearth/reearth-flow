use chrono::{DateTime, Utc};
use flow_websocket_domain::{
    generate_id,
    project::ProjectEditingSession,
    repository::{
        ProjectEditingSessionRepository, ProjectSnapshotRepository, RedisDataManager,
        SnapshotDataRepository,
    },
    snapshot::{Metadata, ObjectDelete, ObjectTenant, SnapshotInfo},
    types::{data::SnapshotData, snapshot::ProjectSnapshot},
};
use mockall::automock;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

use crate::{types::ManageProjectEditSessionTaskData, ProjectServiceError};

const MAX_EMPTY_SESSION_DURATION: i64 = 10_000; // 10 seconds
const MAX_SNAPSHOT_DELTA: i64 = 300_000; // 5 minutes

pub struct ManageEditSessionService<R, S, D, M>
where
    R: ProjectEditingSessionRepository<Error = ProjectServiceError> + Send + Sync,
    S: ProjectSnapshotRepository<Error = ProjectServiceError> + Send + Sync,
    D: SnapshotDataRepository<Error = ProjectServiceError> + Send + Sync,
    M: RedisDataManager<Error = ProjectServiceError> + Send + Sync,
{
    session_repository: Arc<R>,
    snapshot_repository: Arc<S>,
    snapshot_data_repository: Arc<D>,
    redis_data_manager: Arc<M>,
}

#[automock]
impl<R, S, D, M> ManageEditSessionService<R, S, D, M>
where
    R: ProjectEditingSessionRepository<Error = ProjectServiceError> + Send + Sync + 'static,
    S: ProjectSnapshotRepository<Error = ProjectServiceError> + Send + Sync + 'static,
    D: SnapshotDataRepository<Error = ProjectServiceError> + Send + Sync + 'static,
    M: RedisDataManager<Error = ProjectServiceError> + Send + Sync + 'static,
{
    pub fn new(
        session_repository: Arc<R>,
        snapshot_repository: Arc<S>,
        snapshot_data_repository: Arc<D>,
        redis_data_manager: Arc<M>,
    ) -> Self {
        Self {
            session_repository,
            snapshot_repository,
            snapshot_data_repository,
            redis_data_manager,
        }
    }

    pub async fn process(
        &self,
        mut data: ManageProjectEditSessionTaskData,
    ) -> Result<(), ProjectServiceError> {
        if let Some(mut session) = self
            .session_repository
            .get_active_session(&data.project_id)
            .await?
        {
            session
                .load_session(&*self.snapshot_repository, &data.session_id)
                .await?;

            self.update_client_count(&mut data).await?;
            self.merge_updates(&mut session, &mut data).await?;
            self.create_snapshot_if_required(&mut session, &mut data)
                .await?;
            self.end_editing_session_if_conditions_met(&mut session, &data)
                .await?;
            self.complete_job_if_met_requirements(&session, &data)
                .await?;

            self.session_repository.update_session(session).await?;
        }

        Ok(())
    }

    async fn update_client_count(
        &self,
        data: &mut ManageProjectEditSessionTaskData,
    ) -> Result<(), ProjectServiceError> {
        let current_client_count = self.session_repository.get_client_count().await?;
        let old_client_count = data.clients_count.unwrap_or(0);
        data.clients_count = Some(current_client_count);

        if current_client_count == 0
            && old_client_count != current_client_count
            && data.clients_disconnected_at.is_none()
        {
            data.clients_disconnected_at = Some(Utc::now());
        } else if current_client_count > 0 {
            data.clients_disconnected_at = None;
        }

        Ok(())
    }

    async fn merge_updates(
        &self,
        session: &mut ProjectEditingSession,
        data: &mut ManageProjectEditSessionTaskData,
    ) -> Result<(), ProjectServiceError> {
        session
            .merge_updates(&*self.redis_data_manager, false)
            .await?;
        data.last_merged_at = Some(Utc::now());
        Ok(())
    }

    async fn create_snapshot_if_required(
        &self,
        session: &mut ProjectEditingSession,
        data: &mut ManageProjectEditSessionTaskData,
    ) -> Result<(), ProjectServiceError> {
        if let Some(last_snapshot_at) = data.last_snapshot_at {
            let current_time = Utc::now();
            if (current_time - last_snapshot_at).num_milliseconds() > MAX_SNAPSHOT_DELTA {
                self.create_snapshot(session, current_time).await?;
                data.last_snapshot_at = Some(current_time);
            }
        }
        Ok(())
    }

    async fn create_snapshot(
        &self,
        session: &mut ProjectEditingSession,
        current_time: DateTime<Utc>,
    ) -> Result<(), ProjectServiceError> {
        let state = session.get_state_update(&*self.redis_data_manager).await?;

        let metadata = Metadata::new(
            generate_id(14, "snap"),
            session.project_id.clone(),
            session.session_id.clone(),
            String::new(),
            String::new(),
        );

        let snapshot_state = SnapshotInfo::new(
            None,
            vec![],
            ObjectTenant {
                id: session.tenant.id.clone(),
                key: session.tenant.key.clone(),
            },
            ObjectDelete {
                deleted: false,
                delete_after: None,
            },
            Some(current_time),
            Some(current_time),
        );

        let snapshot = ProjectSnapshot::new(metadata, snapshot_state);
        let snapshot_data = SnapshotData::new(session.project_id.clone(), state, None, None);

        self.snapshot_repository.create_snapshot(snapshot).await?;
        self.snapshot_data_repository
            .create_snapshot_data(snapshot_data)
            .await?;

        Ok(())
    }

    async fn end_editing_session_if_conditions_met(
        &self,
        session: &mut ProjectEditingSession,
        data: &ManageProjectEditSessionTaskData,
    ) -> Result<(), ProjectServiceError> {
        let client_count = self.session_repository.get_client_count().await?;

        if let Some(clients_disconnected_at) = data.clients_disconnected_at {
            let current_time = Utc::now();
            let clients_disconnection_elapsed_time =
                (current_time - clients_disconnected_at).num_milliseconds();

            if clients_disconnection_elapsed_time > MAX_EMPTY_SESSION_DURATION && client_count == 0
            {
                session
                    .end_session(&*self.redis_data_manager, &*self.snapshot_repository)
                    .await?;
            }
        }
        Ok(())
    }

    async fn complete_job_if_met_requirements(
        &self,
        session: &ProjectEditingSession,
        data: &ManageProjectEditSessionTaskData,
    ) -> Result<(), ProjectServiceError> {
        if session.active_editing_session().await?.as_ref() == Some(&data.session_id) {
            sleep(Duration::from_secs(5)).await;
        }
        Ok(())
    }
}
