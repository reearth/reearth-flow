use crate::persistence::gcs::gcs_client::{GcsClient, GcsError};
use crate::persistence::redis::redis_client::{RedisClient, RedisClientError};
use async_trait::async_trait;
use flow_websocket_domain::projection::Project;
use flow_websocket_domain::types::data::SnapshotData;

use crate::persistence::local_storage::LocalClient;
use flow_websocket_domain::project::ProjectEditingSession;
use flow_websocket_domain::repository::{
    ProjectEditingSessionRepository, ProjectRepository, ProjectSnapshotRepository,
    SnapshotDataRepository,
};
use flow_websocket_domain::snapshot::ProjectSnapshot;
use serde_json;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;

use super::local_storage::LocalStorageError;
use super::redis::redis_client::RedisClientTrait;
use super::StorageClient;

#[derive(Error, Debug)]
pub enum ProjectRepositoryError {
    #[error(transparent)]
    Redis(#[from] RedisClientError),
    #[error(transparent)]
    Gcs(#[from] GcsError),
    #[error(transparent)]
    Local(#[from] LocalStorageError),
    #[error(transparent)]
    Serialization(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}

pub struct ProjectRedisRepository {
    redis_client: Arc<RedisClient>,
}

impl ProjectRedisRepository {
    pub fn new(redis_client: Arc<RedisClient>) -> Self {
        Self { redis_client }
    }
}

#[async_trait]
impl ProjectRepository for ProjectRedisRepository {
    type Error = ProjectRepositoryError;

    async fn get_project(&self, project_id: &str) -> Result<Option<Project>, Self::Error> {
        let key = format!("project:{}", project_id);
        let project = self.redis_client.get(&key).await?;
        Ok(project)
    }
}

#[async_trait]
impl ProjectEditingSessionRepository for ProjectRedisRepository {
    type Error = ProjectRepositoryError;

    async fn create_session(&self, session: ProjectEditingSession) -> Result<(), Self::Error> {
        let key = format!("session:{}", session.session_id.as_ref().unwrap());
        self.redis_client.set(key, &session).await?;
        Ok(())
    }

    async fn get_active_session(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectEditingSession>, Self::Error> {
        let key = format!("project:{}:active_session", project_id);
        let session = self.redis_client.get(&key).await?;
        Ok(session)
    }

    async fn update_session(&self, session: ProjectEditingSession) -> Result<(), Self::Error> {
        let key = format!("session:{}", session.session_id.as_ref().unwrap());
        self.redis_client.set(key, &session).await?;
        Ok(())
    }
}

impl ProjectRedisRepository {
    pub async fn get_client_count(&self) -> Result<usize, ProjectRepositoryError> {
        let count = self.redis_client.get_client_count().await?;
        Ok(count)
    }
}

pub struct ProjectGcsRepository {
    client: GcsClient,
}

impl ProjectGcsRepository {
    pub fn new(client: GcsClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ProjectSnapshotRepository for ProjectGcsRepository {
    type Error = ProjectRepositoryError;

    async fn create_snapshot(&self, snapshot: ProjectSnapshot) -> Result<(), Self::Error> {
        let path = format!("snapshot/{}", snapshot.metadata.project_id);
        self.client.upload_versioned(path, &snapshot).await?;
        Ok(())
    }

    async fn get_latest_snapshot(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectSnapshot>, Self::Error> {
        let path_prefix = format!("snapshot/{}", project_id);
        let snapshot = self.client.download_latest(&path_prefix).await?;
        Ok(snapshot)
    }

    async fn get_latest_snapshot_state(&self, project_id: &str) -> Result<Vec<u8>, Self::Error> {
        let snapshot_data = self.get_latest_snapshot_data(project_id).await?;
        if let Some(data) = snapshot_data {
            Ok(data.state)
        } else {
            Ok(Vec::new())
        }
    }

    async fn update_latest_snapshot(&self, snapshot: ProjectSnapshot) -> Result<(), Self::Error> {
        let latest_version = self
            .client
            .get_latest_version(&format!("snapshot/{}", snapshot.metadata.project_id))
            .await?;
        if let Some(_version) = latest_version {
            let path = format!("snapshot/{}", snapshot.metadata.project_id);
            self.client.update_versioned(path, &snapshot).await?;
        } else {
            let path = format!("snapshot/{}", snapshot.metadata.project_id);
            self.client.upload(path, &snapshot).await?;
        }

        Ok(())
    }

    async fn update_snapshot_data(
        &self,
        project_id: &str,
        snapshot_data: SnapshotData,
    ) -> Result<(), Self::Error> {
        let path = format!("snapshot_data/{}", project_id);
        self.client.upload(path, &snapshot_data).await?;
        Ok(())
    }
}
#[async_trait]
impl SnapshotDataRepository for ProjectGcsRepository {
    type Error = ProjectRepositoryError;

    async fn create_snapshot_data(&self, snapshot_data: SnapshotData) -> Result<(), Self::Error> {
        let path = format!("snapshot_data/{}", snapshot_data.project_id);
        self.client.upload_versioned(path, &snapshot_data).await?;
        Ok(())
    }

    async fn get_snapshot_data(
        &self,
        snapshot_id: &str,
    ) -> Result<Option<SnapshotData>, Self::Error> {
        let path = format!("snapshot_data/{}", snapshot_id);
        let snapshot_data = self.client.download(path).await?;
        Ok(snapshot_data)
    }

    async fn get_latest_snapshot_data(
        &self,
        project_id: &str,
    ) -> Result<Option<SnapshotData>, Self::Error> {
        let path_prefix = format!("snapshot_data/{}", project_id);
        let snapshot_data = self.client.download_latest(&path_prefix).await?;
        Ok(snapshot_data)
    }

    async fn update_snapshot_data(
        &self,
        snapshot_id: &str,
        snapshot_data: SnapshotData,
    ) -> Result<(), Self::Error> {
        let path = format!("snapshot_data/{}", snapshot_id);
        self.client.upload(path, &snapshot_data).await?;
        Ok(())
    }

    async fn delete_snapshot_data(&self, snapshot_id: &str) -> Result<(), Self::Error> {
        let path = format!("snapshot_data/{}", snapshot_id);
        self.client.delete(path).await?;
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
impl ProjectSnapshotRepository for ProjectLocalRepository {
    type Error = ProjectRepositoryError;

    async fn create_snapshot(&self, snapshot: ProjectSnapshot) -> Result<(), Self::Error> {
        let path = format!("snapshots/{}", snapshot.metadata.id);
        self.client.upload_versioned(path, &snapshot).await?;
        Ok(())
    }

    async fn get_latest_snapshot(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectSnapshot>, Self::Error> {
        let path = format!("snapshots/{}", project_id);
        let snapshot = self
            .client
            .download_latest::<ProjectSnapshot>(&path)
            .await?;
        Ok(snapshot)
    }

    async fn get_latest_snapshot_state(&self, project_id: &str) -> Result<Vec<u8>, Self::Error> {
        let snapshot = self.get_latest_snapshot_data(project_id).await?;
        if let Some(snapshot) = snapshot {
            Ok(snapshot.state)
        } else {
            Ok(Vec::new())
        }
    }

    async fn update_latest_snapshot(&self, snapshot: ProjectSnapshot) -> Result<(), Self::Error> {
        let path = format!("snapshots/{}", snapshot.metadata.id);
        self.client.update_versioned(path, &snapshot).await?;
        Ok(())
    }
}

#[async_trait]
impl SnapshotDataRepository for ProjectLocalRepository {
    type Error = ProjectRepositoryError;

    async fn create_snapshot_data(&self, snapshot_data: SnapshotData) -> Result<(), Self::Error> {
        let path = format!("snapshot_data/{}", snapshot_data.project_id);
        self.client.update_versioned(path, &snapshot_data).await?;
        Ok(())
    }

    async fn get_snapshot_data(
        &self,
        snapshot_id: &str,
    ) -> Result<Option<SnapshotData>, Self::Error> {
        let path = format!("snapshot_data/{}", snapshot_id);
        let snapshot_data = self.client.download(path).await?;
        Ok(snapshot_data)
    }

    async fn get_latest_snapshot_data(
        &self,
        project_id: &str,
    ) -> Result<Option<SnapshotData>, Self::Error> {
        let path = format!("snapshot_data/{}", project_id);
        let snapshot_data = self.client.download_latest(&path).await?;
        Ok(snapshot_data)
    }

    async fn update_snapshot_data(
        &self,
        snapshot_id: &str,
        snapshot_data: SnapshotData,
    ) -> Result<(), Self::Error> {
        let path = format!("snapshot_data/{}", snapshot_id);
        self.client.upload(path, &snapshot_data).await?;
        Ok(())
    }

    async fn delete_snapshot_data(&self, snapshot_id: &str) -> Result<(), Self::Error> {
        let path = format!("snapshot_data/{}", snapshot_id);
        self.client.delete(path).await?;
        Ok(())
    }
}
