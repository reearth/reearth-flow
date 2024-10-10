use async_trait::async_trait;
use chrono::Utc;
use flow_websocket_domain::repository::ProjectSnapshotRepository;
use flow_websocket_domain::snapshot::{Metadata, ObjectDelete, SnapshotInfo};
use flow_websocket_domain::types::data::SnapshotData;
use flow_websocket_domain::types::snapshot::ProjectSnapshot;
use flow_websocket_infra::persistence::project_repository::ProjectRepositoryError;
use std::sync::Arc;
use uuid::Uuid;

use crate::types::CreateSnapshotData;

pub struct SnapshotService<R> {
    snapshot_repository: Arc<R>,
}

impl<R> SnapshotService<R>
where
    R: ProjectSnapshotRepository<Error = ProjectRepositoryError> + Send + Sync,
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

    async fn update_latest_snapshot_data(
        &self,
        project_id: &str,
        snapshot_data: SnapshotData,
    ) -> Result<(), Self::Error> {
        self.snapshot_repository
            .update_latest_snapshot_data(project_id, snapshot_data)
            .await
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;
    use mockall::predicate::*;

    mock! {
        SnapshotRepo {}
        #[async_trait]
        impl ProjectSnapshotRepository for SnapshotRepo {
            type Error = ProjectRepositoryError;

            async fn update_latest_snapshot_data(&self, project_id: &str, snapshot_data: SnapshotData) -> Result<(), ProjectRepositoryError>;
            async fn create_snapshot(&self, snapshot: ProjectSnapshot) -> Result<(), ProjectRepositoryError>;
            async fn get_latest_snapshot(&self, project_id: &str) -> Result<Option<ProjectSnapshot>, ProjectRepositoryError>;
            async fn get_latest_snapshot_state(&self, project_id: &str) -> Result<Vec<u8>, ProjectRepositoryError>;
            async fn update_latest_snapshot(&self, snapshot: ProjectSnapshot) -> Result<(), ProjectRepositoryError>;
        }
    }

    #[tokio::test]
    async fn test_create_snapshot_with_data() {
        let mut mock_repo = MockSnapshotRepo::new();

        mock_repo
            .expect_create_snapshot()
            .withf(|snapshot: &ProjectSnapshot| {
                snapshot.metadata.project_id == "project_123"
                    && snapshot.metadata.session_id == Some("session_456".to_string())
                    && snapshot.info.created_by == Some("user_789".to_string())
            })
            .times(1)
            .returning(|_| Ok(()));

        let service = SnapshotService::new(Arc::new(mock_repo));

        let create_data = CreateSnapshotData {
            project_id: "project_123".to_string(),
            session_id: Some("session_456".to_string()),
            name: Some("Test Snapshot".to_string()),
            created_by: Some("user_789".to_string()),
            changes_by: vec!["user_789".to_string()],
            tenant: flow_websocket_domain::snapshot::ObjectTenant::new(
                "tenant_123".to_string(),
                "tenant_key".to_string(),
            ),
            state: vec![1, 2, 3],
        };

        let result = service.create_snapshot_with_data(create_data).await;
        assert!(result.is_ok());

        let snapshot = result.unwrap();
        assert_eq!(snapshot.metadata.project_id, "project_123");
        assert_eq!(
            snapshot.metadata.session_id,
            Some("session_456".to_string())
        );
        assert_eq!(snapshot.metadata.name, "Test Snapshot");
        assert_eq!(snapshot.info.created_by, Some("user_789".to_string()));
    }

    #[tokio::test]
    async fn test_get_latest_snapshot() {
        let mut mock_repo = MockSnapshotRepo::new();

        let example_snapshot = ProjectSnapshot::new(
            Metadata::new(
                "snapshot_123".to_string(),
                "project_123".to_string(),
                Some("session_456".to_string()),
                "Latest Snapshot".to_string(),
                String::new(),
            ),
            SnapshotInfo::new(
                Some("user_789".to_string()),
                vec!["user_789".to_string()],
                flow_websocket_domain::snapshot::ObjectTenant::new(
                    "tenant_123".to_string(),
                    "tenant_key".to_string(),
                ),
                ObjectDelete {
                    deleted: false,
                    delete_after: None,
                },
                None,
                None,
            ),
        );

        mock_repo
            .expect_get_latest_snapshot()
            .with(eq("project_123"))
            .times(1)
            .returning(move |_| Ok(Some(example_snapshot.clone())));

        let service = SnapshotService::new(Arc::new(mock_repo));

        let result = service.get_latest_snapshot("project_123").await;
        assert!(result.is_ok());

        let snapshot = result.unwrap().unwrap();
        assert_eq!(snapshot.metadata.id, "snapshot_123");
        assert_eq!(snapshot.metadata.project_id, "project_123");
    }

    #[tokio::test]
    async fn test_update_snapshot_data() {
        let mut mock_repo = MockSnapshotRepo::new();

        mock_repo
            .expect_update_latest_snapshot_data()
            .with(
                eq("project_123"),
                function(|data: &SnapshotData| {
                    data.project_id == "project_123"
                        && data.state == vec![1, 2, 3]
                        && data.name == Some("Test Snapshot".to_string())
                        && data.created_by == Some("user_789".to_string())
                }),
            )
            .times(1)
            .returning(|_, _| Ok(()));

        let service = SnapshotService::new(Arc::new(mock_repo));

        let snapshot_data = SnapshotData::new(
            "project_123".to_string(),
            vec![1, 2, 3],
            Some("Test Snapshot".to_string()),
            Some("user_789".to_string()),
        );

        let result = service
            .update_latest_snapshot_data("project_123", snapshot_data)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_latest_snapshot_state() {
        let mut mock_repo = MockSnapshotRepo::new();

        let example_state = vec![1, 2, 3];

        mock_repo
            .expect_get_latest_snapshot_state()
            .with(eq("project_123"))
            .times(1)
            .returning(move |_| Ok(example_state.clone()));

        let service = SnapshotService::new(Arc::new(mock_repo));

        let result = service.get_latest_snapshot_state("project_123").await;
        assert!(result.is_ok());
        let state = result.unwrap();
        assert_eq!(state, vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn test_update_latest_snapshot() {
        let mut mock_repo = MockSnapshotRepo::new();

        mock_repo
            .expect_update_latest_snapshot()
            .withf(|snapshot: &ProjectSnapshot| snapshot.metadata.project_id == "project_123")
            .times(1)
            .returning(|_| Ok(()));

        let service = SnapshotService::new(Arc::new(mock_repo));

        let snapshot = ProjectSnapshot::new(
            Metadata::new(
                "snapshot_123".to_string(),
                "project_123".to_string(),
                Some("session_456".to_string()),
                "Updated Snapshot".to_string(),
                String::new(),
            ),
            SnapshotInfo::new(
                Some("user_789".to_string()),
                vec!["user_789".to_string()],
                flow_websocket_domain::snapshot::ObjectTenant::new(
                    "tenant_123".to_string(),
                    "tenant_key".to_string(),
                ),
                ObjectDelete {
                    deleted: false,
                    delete_after: None,
                },
                None,
                None,
            ),
        );

        let result = service.update_latest_snapshot(snapshot).await;
        assert!(result.is_ok());
    }
}
