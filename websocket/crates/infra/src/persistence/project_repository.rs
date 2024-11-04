use crate::persistence::gcs::gcs_client::{GcsClient, GcsError};
use crate::persistence::redis::redis_client::RedisClientError;
use async_trait::async_trait;
use flow_websocket_domain::generate_id;
use flow_websocket_domain::project::Project;

use crate::persistence::local_storage::LocalClient;
use flow_websocket_domain::editing_session::ProjectEditingSession;
use flow_websocket_domain::repository::{
    ProjectEditingSessionRepository, ProjectRepository, ProjectSnapshotRepository,
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
    #[error("Session ID not found")]
    SessionIdNotFound,
    #[error("{0}")]
    Custom(String),
}

#[derive(Clone)]
pub struct ProjectRedisRepository<R>
where
    R: RedisClientTrait + Send + Sync,
{
    redis_client: Arc<R>,
}

impl<R: RedisClientTrait + Send + Sync> ProjectRedisRepository<R> {
    pub fn new(redis_client: Arc<R>) -> Self {
        Self { redis_client }
    }
}

#[async_trait]
impl<R> ProjectRepository for ProjectRedisRepository<R>
where
    R: RedisClientTrait + Send + Sync,
{
    type Error = ProjectRepositoryError;

    async fn get_project(&self, project_id: &str) -> Result<Option<Project>, Self::Error> {
        let key = format!("project:{}", project_id);
        let project = self.redis_client.get(&key).await?;
        Ok(project)
    }
}

#[async_trait]
impl<R> ProjectEditingSessionRepository for ProjectRedisRepository<R>
where
    R: RedisClientTrait + Send + Sync,
{
    type Error = ProjectRepositoryError;

    /// crate session and set active session
    async fn create_session(
        &self,
        mut session: ProjectEditingSession,
    ) -> Result<String, Self::Error> {
        let session_id = session
            .session_id
            .get_or_insert_with(|| generate_id!("editor-session"))
            .clone();

        let session_key = format!("session:{}", session_id);
        let active_session_key = format!("project:{}:active_session", session.project_id);

        self.redis_client.set(&session_key, &session).await?;
        self.redis_client
            .set(&active_session_key, &session_id)
            .await?;

        Ok(session_id)
    }

    async fn get_active_session(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectEditingSession>, Self::Error> {
        let active_session_key = format!("project:{}:active_session", project_id);
        let session_id: Option<String> = self.redis_client.get(&active_session_key).await?;

        if let Some(session_id) = session_id {
            let session_key = format!("session:{}", session_id);
            let session: Option<ProjectEditingSession> =
                self.redis_client.get(&session_key).await?;
            Ok(session)
        } else {
            Ok(None)
        }
    }

    async fn update_session(&self, session: ProjectEditingSession) -> Result<(), Self::Error> {
        let session_id = session
            .session_id
            .as_ref()
            .ok_or(ProjectRepositoryError::SessionIdNotFound)?;
        let key = format!("session:{}", session_id);
        self.redis_client.set(&key, &session).await?;

        let active_session_key = format!("project:{}:active_session", session.project_id);
        self.redis_client
            .set(&active_session_key, session_id)
            .await?;

        Ok(())
    }

    async fn delete_session(&self, project_id: &str) -> Result<(), Self::Error> {
        let active_session_key = format!("project:{}:active_session", project_id);
        self.redis_client.delete_key(&active_session_key).await?;
        Ok(())
    }
}

#[derive(Clone)]
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

    async fn update_latest_snapshot(&self, snapshot: ProjectSnapshot) -> Result<(), Self::Error> {
        let latest_version = self
            .client
            .get_latest_version(&format!("snapshot/{}", snapshot.metadata.project_id))
            .await?;
        if let Some(_version) = latest_version {
            let path = format!("snapshot/{}", snapshot.metadata.project_id);
            self.client.update_latest_versioned(path, &snapshot).await?;
        } else {
            let path = format!("snapshot/{}", snapshot.metadata.project_id);
            self.client.upload_versioned(path, &snapshot).await?;
        }
        Ok(())
    }

    async fn delete_snapshot(&self, project_id: &str) -> Result<(), Self::Error> {
        let path = format!("snapshot/{}", project_id);
        self.client.delete(path).await?;
        Ok(())
    }

    async fn list_all_snapshots_versions(
        &self,
        project_id: &str,
    ) -> Result<Vec<String>, Self::Error> {
        let path = format!("snapshot/{}", project_id);
        let versions = self.client.list_versions(&path, None).await?;
        Ok(versions.iter().map(|(_, v)| v.clone()).collect())
    }
}

#[derive(Clone)]
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

    async fn update_latest_snapshot(&self, snapshot: ProjectSnapshot) -> Result<(), Self::Error> {
        let path = format!("snapshots/{}", snapshot.metadata.id);
        self.client.update_latest_versioned(path, &snapshot).await?;
        Ok(())
    }

    async fn delete_snapshot(&self, project_id: &str) -> Result<(), Self::Error> {
        let path = format!("snapshots/{}", project_id);
        self.client.delete(path).await?;
        Ok(())
    }

    async fn list_all_snapshots_versions(
        &self,
        project_id: &str,
    ) -> Result<Vec<String>, Self::Error> {
        let path = format!("snapshots/{}", project_id);
        let versions = self.client.list_versions(&path, None).await?;
        Ok(versions.iter().map(|(_, v)| v.clone()).collect())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use flow_websocket_domain::editing_session::ProjectEditingSession;
//     use flow_websocket_domain::snapshot::ObjectTenant;
//     use mockall::mock;

//     type XReadResult = Vec<(String, Vec<(String, String)>)>;
//     type RedisResult<T> = Result<T, RedisClientError>;

//     mock! {
//         RedisClient {}
//         #[async_trait]
//         impl RedisClientTrait for RedisClient {
//             fn redis_url(&self) -> &str;
//             async fn get<T: serde::de::DeserializeOwned + Send + Sync + 'static>(&self, key: &str) -> RedisResult<Option<T>>;
//             async fn set<T: serde::Serialize + Send + Sync + 'static>(&self, key: &str, value: &T) -> RedisResult<()>;
//             async fn get_client_count(&self) -> RedisResult<usize>;
//             async fn keys(&self, pattern: &str) -> RedisResult<Vec<String>>;
//             async fn xadd(&self, key: &str, id: &str, fields: &[(String, String)]) -> RedisResult<String>;
//             async fn xread(&self, key: &str, id: &str) -> RedisResult<XReadResult>;
//             async fn xtrim(&self, key: &str, max_len: usize) -> RedisResult<usize>;
//             async fn xdel(&self, key: &str, ids: &[String]) -> RedisResult<usize>;
//             fn connection(&self) -> &Arc<tokio::sync::Mutex<redis::aio::MultiplexedConnection>>;
//             async fn xread_map(&self, key: &str, id: &str) -> RedisResult<XReadResult>;
//         }
//     }

//     #[tokio::test]
//     async fn test_create_session() {
//         let mut mock_redis = MockRedisClient::new();
//         mock_redis
//             .expect_set()
//             .withf(|key: &str, _value: &ProjectEditingSession| key.starts_with("session:"))
//             .times(1)
//             .returning(|_, _| Ok(()));
//         mock_redis
//             .expect_set()
//             .withf(|key: &str, _value: &String| {
//                 key.starts_with("project:") && key.ends_with(":active_session")
//             })
//             .times(1)
//             .returning(|_, _| Ok(()));

//         let repo = ProjectRedisRepository::new(Arc::new(mock_redis));

//         let mut session = ProjectEditingSession::new(
//             "project_123".to_string(),
//             ObjectTenant::new("tenant_123".to_string(), "tenant_key".to_string()),
//         );
//         session.session_id = Some("session_456".to_string());

//         let result = repo.create_session(session).await;
//         assert!(result.is_ok());
//     }

//     #[tokio::test]
//     async fn test_get_active_session() {
//         let mut mock_redis = MockRedisClient::new();
//         mock_redis
//             .expect_get::<String>()
//             .times(1)
//             .returning(|_| Ok(Some("session_456".to_string())));
//         mock_redis
//             .expect_get::<ProjectEditingSession>()
//             .times(1)
//             .returning(|_| {
//                 Ok(Some(ProjectEditingSession::new(
//                     "project_123".to_string(),
//                     ObjectTenant::new("tenant_123".to_string(), "tenant_key".to_string()),
//                 )))
//             });

//         let repo = ProjectRedisRepository::new(Arc::new(mock_redis));

//         let result = repo.get_active_session("project_123").await;
//         assert!(result.is_ok());
//         let session = result.unwrap();
//         assert!(session.is_some());
//         let session = session.unwrap();
//         assert_eq!(session.project_id, "project_123");
//     }

//     #[tokio::test]
//     async fn test_update_session() {
//         let mut mock_redis = MockRedisClient::new();
//         mock_redis
//             .expect_set()
//             .withf(|key: &str, _value: &ProjectEditingSession| key.starts_with("session:"))
//             .times(1)
//             .returning(|_, _| Ok(()));
//         mock_redis
//             .expect_set()
//             .withf(|key: &str, _value: &String| {
//                 key.starts_with("project:") && key.ends_with(":active_session")
//             })
//             .times(1)
//             .returning(|_, _| Ok(()));

//         let repo = ProjectRedisRepository::new(Arc::new(mock_redis));

//         let mut session = ProjectEditingSession::new(
//             "project_123".to_string(),
//             ObjectTenant::new("tenant_123".to_string(), "tenant_key".to_string()),
//         );
//         session.session_id = Some("session_456".to_string());

//         let result = repo.update_session(session).await;
//         assert!(result.is_ok());
//     }

//     #[tokio::test]
//     async fn test_get_client_count() {
//         let mut mock_redis = MockRedisClient::new();
//         mock_redis
//             .expect_get_client_count()
//             .times(1)
//             .returning(|| Ok(5));

//         let repo = ProjectRedisRepository::new(Arc::new(mock_redis));

//         let result = repo.get_client_count().await;
//         assert!(result.is_ok());
//         assert_eq!(result.unwrap(), 5);
//     }
// }
