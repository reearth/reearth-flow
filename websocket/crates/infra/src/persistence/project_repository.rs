use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;

use flow_websocket_domain::project::{Project, ProjectEditingSession};
use flow_websocket_domain::repository::{ProjectRepository, ProjectEditingSessionRepository};

use crate::persistence::redis::redis_client::RedisClient;

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

    async fn get_active_session(&self, project_id: &str) -> Result<Option<ProjectEditingSession>, Box<dyn Error>> {
        let key = format!("project:{}:active_session", project_id);
        self.redis_client.get(&key).await
    }

    async fn update_session(&self, session: ProjectEditingSession) -> Result<(), Box<dyn Error>> {
        let key = format!("session:{}", session.session_id.as_ref().unwrap());
        self.redis_client.set(key, &session).await?;
        Ok(())
    }
}
