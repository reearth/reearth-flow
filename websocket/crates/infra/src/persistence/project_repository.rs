use crate::persistence::gcs::gcs_client::{GcsClient, GcsError};
use crate::persistence::redis::redis_client::{RedisClient, RedisClientError};
use async_trait::async_trait;
use chrono::Utc;

use crate::persistence::local_storage::LocalClient;
use flow_websocket_domain::project::{Project, ProjectEditingSession};
use flow_websocket_domain::repository::{
    ProjectEditingSessionRepository, ProjectRepository, ProjectSnapshotRepository,
};
use flow_websocket_domain::snapshot::ProjectSnapshot;
use serde_json;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProjectRepositoryError {
    #[error(transparent)]
    Redis(#[from] RedisClientError),
    #[error(transparent)]
    Gcs(#[from] GcsError),
    #[error(transparent)]
    Serialization(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}
use flow_websocket_domain::snapshot::SnapshotState;

pub struct ProjectRedisRepository {
    redis_client: Arc<RedisClient>,
}

impl ProjectRedisRepository {
    pub fn new(redis_client: Arc<RedisClient>) -> Self {
        Self { redis_client }
    }
}

#[async_trait]
impl ProjectRepository<ProjectRepositoryError> for ProjectRedisRepository {
    async fn get_project(
        &self,
        project_id: &str,
    ) -> Result<Option<Project>, ProjectRepositoryError> {
        let key = format!("project:{}", project_id);
        let project = self.redis_client.get(&key).await?;
        Ok(project)
    }
}

#[async_trait]
impl ProjectEditingSessionRepository<ProjectRepositoryError> for ProjectRedisRepository {
    async fn create_session(
        &self,
        session: ProjectEditingSession,
    ) -> Result<(), ProjectRepositoryError> {
        let key = format!("session:{}", session.session_id.as_ref().unwrap());
        self.redis_client.set(key, &session).await?;
        Ok(())
    }

    async fn get_active_session(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectEditingSession>, ProjectRepositoryError> {
        let key = format!("project:{}:active_session", project_id);
        let session = self.redis_client.get(&key).await?;
        Ok(session)
    }

    async fn update_session(
        &self,
        session: ProjectEditingSession,
    ) -> Result<(), ProjectRepositoryError> {
        let key = format!("session:{}", session.session_id.as_ref().unwrap());
        self.redis_client.set(key, &session).await?;
        Ok(())
    }
}

pub struct ProjectGcsRepository {
    client: GcsClient,
}

impl ProjectGcsRepository {
    fn new(client: GcsClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ProjectSnapshotRepository<ProjectRepositoryError> for ProjectGcsRepository {
    async fn create_snapshot(
        &self,
        snapshot: ProjectSnapshot,
    ) -> Result<(), ProjectRepositoryError> {
        let path = format!("snapshot/{}", snapshot.metadata.id);
        self.client.upload(path, &snapshot).await?;
        Ok(())
    }

    async fn get_latest_snapshot(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectSnapshot>, ProjectRepositoryError> {
        let path = format!("snapshot/{}:latest_snapshot", project_id);
        let snapshot = self.client.download(path).await?;
        Ok(snapshot)
    }

    async fn get_latest_snapshot_state(
        &self,
        project_id: &str,
    ) -> Result<Vec<u8>, ProjectRepositoryError> {
        let path = format!("snapshot/{}:latest_snapshot_state", project_id);
        let state = self.client.download(path).await?;
        Ok(state)
    }

    async fn update_snapshot(
        &self,
        snapshot: ProjectSnapshot,
    ) -> Result<(), ProjectRepositoryError> {
        let snapshot_metadata = serde_json::to_vec(&snapshot.metadata)?;

        // Upload the serialized data to GCS, overwriting the existing file
        let path = format!("snapshot/{}", snapshot.metadata.project_id);
        self.client.upload(path.clone(), &snapshot_metadata).await?;

        // Update the latest snapshot reference
        let latest_path = format!("snapshot/{}:latest_snapshot", snapshot.metadata.project_id);
        self.client.upload(latest_path, &snapshot).await?;

        //Get the latest snapshot state
        let latest_snapshot_state_path = format!(
            "snapshot/{}:latest_snapshot_state",
            snapshot.metadata.project_id
        );

        let latest_snapshot_state: Vec<u8> = self
            .client
            .download(latest_snapshot_state_path.clone())
            .await?;

        // Update the latest snapshot state
        let mut latest_snapshot_state: SnapshotState =
            serde_json::from_slice(&latest_snapshot_state)?;
        latest_snapshot_state.updated_at = Some(Utc::now());
        latest_snapshot_state
            .changes_by
            .push(snapshot.state.created_by.unwrap_or_default());

        self.client
            .upload(latest_snapshot_state_path, &latest_snapshot_state)
            .await?;

        Ok(())
    }
}

pub struct ProjectLocalRepository {
    client: Arc<LocalClient>,
}

impl ProjectLocalRepository {
    pub async fn new(base_path: PathBuf) -> io::Result<Self> {
        Ok(Self {
            client: Arc::new(LocalClient::new(base_path).await?),
        })
    }
}

#[async_trait]
impl ProjectRepository<ProjectRepositoryError> for ProjectLocalRepository {
    async fn get_project(
        &self,
        project_id: &str,
    ) -> Result<Option<Project>, ProjectRepositoryError> {
        let path = format!("projects/{}", project_id);
        match self.client.download::<Project>(path).await {
            Ok(project) => Ok(Some(project)),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(ProjectRepositoryError::Io(e)),
        }
    }
}

#[async_trait]
impl ProjectSnapshotRepository<ProjectRepositoryError> for ProjectLocalRepository {
    async fn create_snapshot(
        &self,
        snapshot: ProjectSnapshot,
    ) -> Result<(), ProjectRepositoryError> {
        let path = format!("snapshots/{}", snapshot.metadata.id);
        self.client.upload(path, &snapshot, true).await?;

        // Update latest snapshot
        let latest_path = format!("latest_snapshots/{}", snapshot.metadata.project_id);
        self.client.upload(latest_path, &snapshot, true).await?;
        Ok(())
    }

    async fn get_latest_snapshot(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectSnapshot>, ProjectRepositoryError> {
        let path = format!("snapshot/{}:latest_snapshot", project_id);
        match self.client.download::<ProjectSnapshot>(path).await {
            Ok(snapshot) => Ok(Some(snapshot)),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(ProjectRepositoryError::Io(e)),
        }
    }

    async fn get_latest_snapshot_state(
        &self,
        project_id: &str,
    ) -> Result<Vec<u8>, ProjectRepositoryError> {
        let path = format!("latest_snapshots/{}", project_id);
        match self.client.download::<ProjectSnapshot>(path).await {
            Ok(snapshot) => Ok(serde_json::to_vec(&snapshot)?),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(vec![]),
            Err(e) => Err(ProjectRepositoryError::Io(e)),
        }
    }

    async fn update_snapshot(
        &self,
        snapshot: ProjectSnapshot,
    ) -> Result<(), ProjectRepositoryError> {
        //let state = serde_json::to_vec(&snapshot.state)?;
        let snapshot_metadata = serde_json::to_vec(&snapshot.metadata)?;

        // Upload the serialized data to GCS, overwriting the existing file
        let path = format!("snapshot/{}", snapshot.metadata.project_id);
        self.client
            .upload(path.clone(), &snapshot_metadata, true)
            .await?;

        // Update the latest snapshot reference
        let latest_path = format!("snapshot/{}:latest_snapshot", snapshot.metadata.project_id);
        self.client.upload(latest_path, &snapshot, true).await?;

        //Get the latest snapshot state
        let latest_snapshot_state_path = format!(
            "snapshot/{}:latest_snapshot_state",
            snapshot.metadata.project_id
        );

        let latest_snapshot_state: Vec<u8> = self
            .client
            .download(latest_snapshot_state_path.clone())
            .await?;

        // Update the latest snapshot state
        let mut latest_snapshot_state: SnapshotState =
            serde_json::from_slice(&latest_snapshot_state)?;
        latest_snapshot_state.updated_at = Some(Utc::now());
        latest_snapshot_state
            .changes_by
            .push(snapshot.state.created_by.unwrap_or_default());

        self.client
            .upload(latest_snapshot_state_path, &latest_snapshot_state, true)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flow_websocket_domain::snapshot::{Metadata, ObjectDelete, ObjectTenant};
    use tempfile::TempDir;
    use tokio::test;

    async fn setup() -> Result<(TempDir, ProjectLocalRepository), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let repo = ProjectLocalRepository::new(temp_dir.path().to_path_buf()).await?;
        Ok((temp_dir, repo))
    }

    #[test]
    async fn test_get_project_non_existent() -> Result<(), Box<dyn std::error::Error>> {
        let (_temp_dir, repo) = setup().await?;
        let project_id = "non_existent_project";
        assert!(repo.get_project(project_id).await?.is_none());
        Ok(())
    }

    #[test]
    async fn test_get_project_existing() -> Result<(), Box<dyn std::error::Error>> {
        let (_temp_dir, repo) = setup().await?;
        let project_id = "test_project";
        let project = Project {
            id: project_id.to_string(),
            workspace_id: "test_workspace".to_string(),
        };

        repo.client
            .upload(format!("projects/{}", project_id), &project, true)
            .await?;

        let retrieved_project = repo.get_project(project_id).await?.unwrap();
        assert_eq!(retrieved_project.id, project.id);
        assert_eq!(retrieved_project.workspace_id, project.workspace_id);
        Ok(())
    }

    fn create_test_snapshot(project_id: &str) -> ProjectSnapshot {
        let now = Utc::now();

        let metadata = Metadata::new(
            "snap_abc123".to_string(),
            project_id.to_string(),
            Some("session_abc123".to_string()),
            "Test Snapshot".to_string(),
            "/test/path".to_string(),
        );

        let state = SnapshotState::new(
            Some("test_user_abc".to_string()),
            vec!["test_user_abc".to_string()],
            ObjectTenant {
                id: "tenant_abc123".to_string(),
                key: "tenant_key_abc".to_string(),
            },
            ObjectDelete {
                deleted: false,
                delete_after: None,
            },
            Some(now),
            Some(now),
        );

        ProjectSnapshot { metadata, state }
    }

    #[test]
    async fn test_create_and_get_snapshot_metadata() -> Result<(), Box<dyn std::error::Error>> {
        let (_temp_dir, repo) = setup().await?;
        let project_id = "test_project";
        let snapshot = create_test_snapshot(project_id);
        println!("snapshot: {:?}", snapshot);
        repo.create_snapshot(snapshot.clone()).await.unwrap();
        println!("snapshot created");

        let retrieved_snapshot = match repo.get_latest_snapshot(project_id).await? {
            Some(snapshot) => {
                println!("retrieved_snapshot: {:?}", snapshot);
                snapshot
            }
            None => {
                println!("Error: No snapshot found for project_id: {}", project_id);
                return Err("No snapshot found".into());
            }
        };

        println!("retrieved_snapshot: {:?}", retrieved_snapshot);
        assert_eq!(retrieved_snapshot.metadata.id, snapshot.metadata.id);
        assert_eq!(
            retrieved_snapshot.metadata.project_id,
            snapshot.metadata.project_id
        );
        Ok(())
    }

    #[test]
    async fn test_create_and_get_snapshot_state() -> Result<(), Box<dyn std::error::Error>> {
        let (_temp_dir, repo) = setup().await?;
        let project_id = "test_project";
        let snapshot = create_test_snapshot(project_id);
        repo.create_snapshot(snapshot.clone()).await.unwrap();
        let retrieved_snapshot_state = repo.get_latest_snapshot_state(project_id).await?;
        assert!(!retrieved_snapshot_state.is_empty());
        Ok(())
    }
}
