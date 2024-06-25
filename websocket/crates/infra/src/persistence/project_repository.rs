use async_trait::async_trait;

use crate::persistence::gcs::gcs_client::GcsClient;
use crate::persistence::redis::flow_project_lock::FlowProjectLock;
use crate::persistence::redis::redis_client::RedisClient;
use std::error::Error;
use std::sync::Arc;

use flow_websocket_domain::project::{Project, ProjectEditingSession};
use flow_websocket_domain::repository::{
    ProjectEditingSessionRepository, ProjectRepository, ProjectSnapshotRepository,
};
use flow_websocket_domain::snapshot::ProjectSnapshot;

pub struct ProjectRedisRepository {
    redis_client: Arc<RedisClient>,
    global_lock: FlowProjectLock,
}

impl ProjectRedisRepository {
    pub fn new(redis_client: Arc<RedisClient>) -> Self {
        let global_lock = FlowProjectLock::new(redis_client.connection());
        Self {
            redis_client,
            global_lock,
        }
    }
}

#[async_trait]
impl ProjectRepository for ProjectRedisRepository {
    async fn get_project(&self, project_id: &str) -> Result<Option<Project>, Box<dyn Error>> {
        let key = format!("project:{}", project_id);
        self.redis_client.get(&key).await
    }
}

#[async_trait]
impl ProjectEditingSessionRepository for ProjectRedisRepository {
    async fn create_session(&self, session: ProjectEditingSession) -> Result<(), Box<dyn Error>> {
        let key = format!("session:{}", session.session_id.as_ref().unwrap());
        self.redis_client.set(key, &session).await?;
        Ok(())
    }

    async fn get_active_session(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectEditingSession>, Box<dyn Error>> {
        let key = format!("project:{}:active_session", project_id);
        self.redis_client.get(&key).await
    }

    async fn update_session(&self, session: ProjectEditingSession) -> Result<(), Box<dyn Error>> {
        let key = format!("session:{}", session.session_id.as_ref().unwrap());
        self.redis_client.set(key, &session).await?;
        Ok(())
    }
}

#[async_trait]
impl ProjectSnapshotRepository for ProjectRedisRepository {
    async fn create_snapshot(&self, snapshot: ProjectSnapshot) -> Result<(), Box<dyn Error>> {
        let key = format!("snapshot:{}", snapshot.id);
        self.redis_client.set(key, &snapshot).await?;
        Ok(())
    }

    async fn get_latest_snapshot(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectSnapshot>, Box<dyn Error>> {
        let key = format!("project:{}:latest_snapshot", project_id);
        self.redis_client.get(&key).await
    }

    async fn get_latest_snapshot_state(&self, project_id: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        let key = format!("project:{}:latest_snapshot_state", project_id);
        self.redis_client.get(&key).await
    }
}

/// A `ProjectGcsRepository` is a thin wrapper of `GcsClient`.
pub struct ProjectGcsRepository {
    client: GcsClient,
}

impl ProjectGcsRepository {
    /// Returns the `ProjectGcsRepository`.
    fn new(client: GcsClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ProjectSnapshotRepository for ProjectGcsRepository {
    /// Create a snapshot.
    async fn create_snapshot(&self, snapshot: ProjectSnapshot) -> Result<(), Box<dyn Error>> {
        let path = format!("snapshot/{}", snapshot.id);
        self.client.upload(path, &snapshot);
        Ok(())
    }

    /// Get the latest snapshot.
    async fn get_latest_snapshot(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectSnapshot>, Box<dyn Error>> {
        let path = format!("snapshot/{}:latest_snapshot", project_id);
        self.client.download(path).await
    }

    /// Get the state of the latest snapshot.
    async fn get_latest_snapshot_state(&self, project_id: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        let path = format!("snapshot/{}:latest_snapshot_state", project_id);
        self.client.download(path).await
    }
}
