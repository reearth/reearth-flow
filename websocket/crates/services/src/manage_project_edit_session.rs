use chrono::{DateTime, Utc};
use flow_websocket_domain::project::ProjectEditingSession;
use flow_websocket_domain::repository::{
    ProjectEditingSessionRepository, ProjectSnapshotRepository,
};
use flow_websocket_domain::snapshot::Metadata;
use flow_websocket_domain::utils::generate_id;
use std::error::Error;
use std::sync::Arc;
use tokio::time::sleep;

const MAX_EMPTY_SESSION_DURATION: i64 = 10 * 1000; // 10 seconds
const MAX_SNAPSHOT_DELTA: i64 = 5 * 60 * 1000; // 5 minutes

pub struct ManageEditSessionService<E: Error + Send + Sync> {
    session_repository: Arc<dyn ProjectEditingSessionRepository<E>>,
    snapshot_repository: Arc<dyn ProjectSnapshotRepository<E>>,
}

impl<E: Error + Send + Sync> ManageEditSessionService<E> {
    pub fn new(
        session_repository: Arc<dyn ProjectEditingSessionRepository<E>>,
        snapshot_repository: Arc<dyn ProjectSnapshotRepository<E>>,
    ) -> Self {
        Self {
            session_repository,
            snapshot_repository,
        }
    }

    pub async fn process(
        &self,
        mut data: ManageProjectEditSessionTaskData,
    ) -> Result<(), Box<dyn Error>> {
        let mut session = self
            .session_repository
            .get_active_session(&data.project_id)
            .await?
            .ok_or_else(|| "No active session found".to_string())?;
        session.load_session(&data.session_id).await?;

        self.update_client_count(&mut session, &mut data).await?;
        self.merge_updates(&mut session, &mut data).await?;
        self.create_snapshot_if_required(&mut session, &mut data)
            .await?;
        self.end_editing_session_if_conditions_met(&mut session, &data)
            .await?;
        self.complete_job_if_met_requirements(&session, &data)
            .await?;

        Ok(())
    }

    async fn update_client_count(
        &self,
        session: &mut ProjectEditingSession,
        data: &mut ManageProjectEditSessionTaskData,
    ) -> Result<(), Box<dyn Error>> {
        let current_client_count = session.get_client_count().await?;
        let old_client_count = data.clients_count.unwrap_or(0);
        data.clients_count = Some(current_client_count);

        if current_client_count == 0
            && old_client_count != current_client_count
            && data.clients_disconnected_at.is_none()
        {
            data.clients_disconnected_at = Some(Utc::now());
        }

        if current_client_count > 0 {
            data.clients_disconnected_at = None;
        }

        Ok(())
    }

    async fn merge_updates(
        &self,
        session: &mut ProjectEditingSession,
        data: &mut ManageProjectEditSessionTaskData,
    ) -> Result<(), Box<dyn Error>> {
        session.merge_updates().await?;
        data.last_merged_at = Some(Utc::now());
        Ok(())
    }

    async fn create_snapshot_if_required(
        &self,
        session: &mut ProjectEditingSession,
        data: &mut ManageProjectEditSessionTaskData,
    ) -> Result<(), Box<dyn Error>> {
        if let Some(last_snapshot_at) = data.last_snapshot_at {
            let current_time = Utc::now();
            let snapshot_time_delta = (current_time - last_snapshot_at).num_milliseconds();

            if snapshot_time_delta > MAX_SNAPSHOT_DELTA {
                let (state, _) = session.get_state_update().await?;

                let metadata = Metadata::new(
                    generate_id(14, "snap"),
                    session.project_id.clone(),
                    Some(session.session_id.clone()),
                    String::new(),
                    String::new(),
                );

                let snapshot_state = SnapshotState::new(
                    None,   // created_by
                    vec![], // changes_by
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

                self.snapshot_repository.create_snapshot(snapshot).await?;
                data.last_snapshot_at = Some(current_time);
            }
        }
        Ok(())
    }

    async fn end_editing_session_if_conditions_met(
        &self,
        session: &mut ProjectEditingSession,
        data: &ManageProjectEditSessionTaskData,
    ) -> Result<(), Box<dyn Error>> {
        let client_count = session.get_client_count().await?;

        if let Some(clients_disconnected_at) = data.clients_disconnected_at {
            let current_time = Utc::now();
            let clients_disconnection_elapsed_time =
                (current_time - clients_disconnected_at).num_milliseconds();

            if clients_disconnection_elapsed_time > MAX_EMPTY_SESSION_DURATION && client_count == 0
            {
                session.end_session().await?;
            }
        }
        Ok(())
    }

    async fn complete_job_if_met_requirements(
        &self,
        session: &ProjectEditingSession,
        data: &ManageProjectEditSessionTaskData,
    ) -> Result<(), Box<dyn Error>> {
        let current_editing_session = session.active_editing_session().await?;
        if current_editing_session.as_ref() == Some(&data.session_id) {
            sleep(std::time::Duration::from_secs(5)).await;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct ManageProjectEditSessionTaskData {
    pub session_id: String,
    pub project_id: String,
    pub clients_count: Option<i32>,
    pub last_merged_at: Option<DateTime<Utc>>,
    pub last_snapshot_at: Option<DateTime<Utc>>,
    pub clients_disconnected_at: Option<DateTime<Utc>>,
}
