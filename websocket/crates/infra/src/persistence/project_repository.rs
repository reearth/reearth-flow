use anyhow::{Context, Result};
use std::sync::Arc;

use async_trait::async_trait;

use crate::persistence::gcs::gcs_client::GcsClient;
use crate::persistence::redis::redis_client::RedisClient;

use flow_websocket_domain::project::{Project, ProjectEditingSession};
use flow_websocket_domain::repository::{
    ProjectEditingSessionRepository, ProjectRepository, ProjectSnapshotRepository,
};
use flow_websocket_domain::snapshot::ProjectSnapshot;

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
    async fn get_project(&self, project_id: &str) -> Result<Option<Project>> {
        let key = format!("project:{}", project_id);
        self.redis_client
            .get(&key)
            .await
            .context("Failed to get project from Redis")
    }
}

#[async_trait]
impl ProjectEditingSessionRepository for ProjectRedisRepository {
    async fn create_session(&self, session: ProjectEditingSession) -> Result<()> {
        let key = format!("session:{}", session.session_id.as_ref().unwrap());
        self.redis_client
            .set(key, &session)
            .await
            .context("Failed to create session in Redis")?;
        Ok(())
    }

    async fn get_active_session(&self, project_id: &str) -> Result<Option<ProjectEditingSession>> {
        let key = format!("project:{}:active_session", project_id);
        self.redis_client
            .get(&key)
            .await
            .context("Failed to get active session from Redis")
    }

    async fn update_session(&self, session: ProjectEditingSession) -> Result<()> {
        let key = format!("session:{}", session.session_id.as_ref().unwrap());
        self.redis_client
            .set(key, &session)
            .await
            .context("Failed to update session in Redis")?;
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
impl ProjectSnapshotRepository for ProjectGcsRepository {
    async fn create_snapshot(&self, snapshot: ProjectSnapshot) -> Result<()> {
        let path = format!("snapshot/{}", snapshot.id);
        self.client
            .upload(path, &snapshot)
            .await
            .context("Failed to create snapshot in GCS")?;
        Ok(())
    }

    async fn get_latest_snapshot(&self, project_id: &str) -> Result<Option<ProjectSnapshot>> {
        let path = format!("snapshot/{}:latest_snapshot", project_id);
        self.client
            .download(path)
            .await
            .context("Failed to get latest snapshot from GCS")
    }

    async fn get_latest_snapshot_state(&self, project_id: &str) -> Result<Vec<u8>> {
        let path = format!("snapshot/{}:latest_snapshot_state", project_id);
        self.client
            .download(path)
            .await
            .context("Failed to get latest snapshot state from GCS")
    }
}
