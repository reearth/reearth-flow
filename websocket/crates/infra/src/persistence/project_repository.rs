use crate::persistence::gcs::gcs_client::{GcsClient, GcsError};
use crate::persistence::redis::redis_client::{RedisClient, RedisClientError};
use async_trait::async_trait;
use flow_websocket_domain::project::{Project, ProjectEditingSession};
use flow_websocket_domain::repository::{
    ProjectEditingSessionRepository, ProjectRepository, ProjectSnapshotRepository,
};
use flow_websocket_domain::snapshot::ProjectSnapshot;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProjectRepositoryError {
    #[error("Redis error: {0}")]
    Redis(#[from] RedisClientError),
    #[error("GCS error: {0}")]
    Gcs(#[from] GcsError),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
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
}
