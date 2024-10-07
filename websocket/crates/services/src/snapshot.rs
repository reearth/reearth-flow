use async_trait::async_trait;
use chrono::Utc;
use flow_websocket_domain::repository::ProjectSnapshotRepository;
use flow_websocket_domain::snapshot::{Metadata, ObjectDelete, SnapshotInfo};
use flow_websocket_domain::types::snapshot::ProjectSnapshot;
use std::sync::Arc;
use uuid::Uuid;

use crate::types::CreateSnapshotData;

pub struct SnapshotService<R> {
    snapshot_repository: Arc<R>,
}

impl<R> SnapshotService<R>
where
    R: ProjectSnapshotRepository + Send + Sync,
{
    pub fn new(snapshot_repository: Arc<R>) -> Self {
        Self {
            snapshot_repository,
        }
    }

    pub async fn create_snapshot_with_data(
        &self,
        data: CreateSnapshotData,
    ) -> Result<ProjectSnapshot, R::Error> {
        let snapshot_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let metadata = Metadata::new(
            snapshot_id.clone(),
            data.project_id.clone(),
            data.session_id,
            data.name.unwrap_or_default(),
            String::new(),
        );

        let state = SnapshotInfo::new(
            data.created_by,
            data.changes_by,
            data.tenant,
            ObjectDelete {
                deleted: false,
                delete_after: None,
            },
            Some(now),
            Some(now),
        );

        let snapshot = ProjectSnapshot::new(metadata, state);

        self.snapshot_repository
            .create_snapshot(snapshot.clone())
            .await?;

        Ok(snapshot)
    }
}

#[async_trait]
impl<R> ProjectSnapshotRepository for SnapshotService<R>
where
    R: ProjectSnapshotRepository + Send + Sync,
{
    type Error = R::Error;

    async fn create_snapshot(&self, snapshot: ProjectSnapshot) -> Result<(), Self::Error> {
        self.snapshot_repository.create_snapshot(snapshot).await
    }

    async fn get_latest_snapshot(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectSnapshot>, Self::Error> {
        self.snapshot_repository
            .get_latest_snapshot(project_id)
            .await
    }

    async fn get_latest_snapshot_state(&self, project_id: &str) -> Result<Vec<u8>, Self::Error> {
        self.snapshot_repository
            .get_latest_snapshot_state(project_id)
            .await
    }

    async fn update_latest_snapshot(&self, snapshot: ProjectSnapshot) -> Result<(), Self::Error> {
        self.snapshot_repository
            .update_latest_snapshot(snapshot)
            .await
    }
}
