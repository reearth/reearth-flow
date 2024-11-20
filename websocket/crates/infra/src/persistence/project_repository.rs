use crate::generate_id;
#[cfg(feature = "gcs-storage")]
use crate::persistence::gcs::gcs_client::{GcsClient, GcsError};

#[cfg(feature = "local-storage")]
pub use self::local::ProjectLocalRepository;
use super::editing_session::ProjectEditingSession;
#[cfg(feature = "local-storage")]
use super::local_storage::LocalStorageError;
use super::repository::{
    ProjectEditingSessionImpl, ProjectImpl, ProjectSnapshotImpl, WorkspaceImpl,
};
use super::StorageClient;
#[cfg(feature = "local-storage")]
use crate::persistence::local_storage::LocalClient;
use crate::types::project::Project;
use crate::types::snapshot::ProjectSnapshot;
use async_trait::async_trait;
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use redis::AsyncCommands;
use serde_json;
use std::io;
#[cfg(feature = "local-storage")]
use std::path::PathBuf;
#[cfg(feature = "local-storage")]
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProjectRepositoryError {
    #[cfg(feature = "gcs-storage")]
    #[error(transparent)]
    Gcs(#[from] GcsError),
    #[cfg(feature = "local-storage")]
    #[error(transparent)]
    Local(#[from] LocalStorageError),
    #[error(transparent)]
    Serialization(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Session ID not found")]
    SessionIdNotFound,
    #[error(transparent)]
    Redis(#[from] redis::RedisError),
    #[error(transparent)]
    Pool(#[from] bb8::RunError<redis::RedisError>),
}

#[derive(Clone)]
pub struct ProjectRedisRepository {
    redis_pool: Pool<RedisConnectionManager>,
}

impl ProjectRedisRepository {
    pub fn new(redis_pool: Pool<RedisConnectionManager>) -> Self {
        Self { redis_pool }
    }
}

#[async_trait]
impl ProjectEditingSessionImpl for ProjectRedisRepository {
    type Error = ProjectRepositoryError;

    async fn create_session(
        &self,
        mut session: ProjectEditingSession,
    ) -> Result<String, Self::Error> {
        let mut conn = self.redis_pool.get().await?;

        let session_id = session
            .session_id
            .get_or_insert_with(|| generate_id!("editor-session:"))
            .clone();

        let session_key = format!("session:{}", session_id);
        let active_session_key = format!("project:{}:active_session", session.project_id);

        let session_json = serde_json::to_string(&session)?;
        let _: () = conn.set(&session_key, session_json).await?;
        let _: () = conn.set(&active_session_key, &session_id).await?;

        Ok(session_id)
    }

    async fn get_active_session(
        &self,
        project_id: &str,
    ) -> Result<Option<ProjectEditingSession>, Self::Error> {
        let mut conn = self.redis_pool.get().await?;

        let active_session_key = format!("project:{}:active_session", project_id);
        let session_id: Option<String> = conn.get(&active_session_key).await?;

        if let Some(session_id) = session_id {
            let session_key = format!("session:{}", session_id);
            let session: Option<String> = conn.get(&session_key).await?;
            Ok(session.map(|s| serde_json::from_str(&s)).transpose()?)
        } else {
            Ok(None)
        }
    }

    async fn update_session(&self, session: ProjectEditingSession) -> Result<(), Self::Error> {
        let mut conn = self.redis_pool.get().await?;

        let session_id = session
            .session_id
            .as_ref()
            .ok_or(ProjectRepositoryError::SessionIdNotFound)?;

        let key = format!("session:{}", session_id);
        let session_json = serde_json::to_string(&session)?;
        let _: () = conn.set(&key, session_json).await?;

        let active_session_key = format!("project:{}:active_session", session.project_id);
        let _: () = conn.set(&active_session_key, session_id).await?;

        Ok(())
    }

    async fn delete_session(&self, project_id: &str) -> Result<(), Self::Error> {
        let mut conn = self.redis_pool.get().await?;
        let active_session_key = format!("project:{}:active_session", project_id);
        let _: () = conn.del(&active_session_key).await?;
        Ok(())
    }
}

#[cfg(feature = "gcs-storage")]
pub(crate) mod gcs {

    use crate::types::workspace::Workspace;

    use super::*;

    #[derive(Clone)]
    pub struct ProjectGcsRepository {
        client: GcsClient,
    }

    impl ProjectGcsRepository {
        pub async fn new(bucket: String) -> Result<Self, GcsError> {
            let client = GcsClient::new(bucket).await?;
            Ok(Self { client })
        }
    }

    #[async_trait]
    impl ProjectSnapshotImpl for ProjectGcsRepository {
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

        async fn update_latest_snapshot(
            &self,
            snapshot: ProjectSnapshot,
        ) -> Result<(), Self::Error> {
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

    #[async_trait]
    impl WorkspaceImpl for ProjectGcsRepository {
        type Error = ProjectRepositoryError;

        async fn get_workspace(
            &self,
            workspace_id: &str,
        ) -> Result<Option<Workspace>, Self::Error> {
            let path = format!("workspace/{}", workspace_id);
            let workspace = self.client.download::<Workspace>(path).await?;
            Ok(Some(workspace))
        }

        async fn list_workspace_projects_ids(
            &self,
            workspace_id: &str,
        ) -> Result<Vec<String>, Self::Error> {
            let path = format!("workspace/{}", workspace_id);
            let workspace = self.client.download::<Workspace>(path).await?;
            let project_ids = workspace.projects;
            Ok(project_ids)
        }

        async fn create_workspace(&self, workspace: Workspace) -> Result<(), Self::Error> {
            let path = format!("workspace/{}", workspace.id);
            self.client.upload(path, &workspace).await?;
            Ok(())
        }

        async fn update_workspace(&self, workspace: Workspace) -> Result<(), Self::Error> {
            let path = format!("workspace/{}", workspace.id);
            self.client.upload(path, &workspace).await?;
            Ok(())
        }

        async fn delete_workspace(&self, workspace_id: &str) -> Result<(), Self::Error> {
            let path = format!("workspace/{}", workspace_id);
            self.client.delete(path).await?;
            Ok(())
        }
    }
    #[async_trait]
    impl ProjectImpl for ProjectGcsRepository {
        type Error = ProjectRepositoryError;

        async fn create_project(&self, project: Project) -> Result<(), Self::Error> {
            let path = format!("project/{}", project.id);
            self.client.upload(path, &project).await?;
            Ok(())
        }

        async fn delete_project(&self, project_id: &str) -> Result<(), Self::Error> {
            let path = format!("project/{}", project_id);
            self.client.delete(path).await?;
            Ok(())
        }

        async fn update_project(&self, project: Project) -> Result<(), Self::Error> {
            let path = format!("project/{}", project.id);
            self.client.upload(path, &project).await?;
            Ok(())
        }
    }
}

#[cfg(feature = "local-storage")]
pub(crate) mod local {
    use crate::types::workspace::Workspace;

    use super::*;

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
    impl ProjectSnapshotImpl for ProjectLocalRepository {
        type Error = ProjectRepositoryError;

        async fn create_snapshot(&self, snapshot: ProjectSnapshot) -> Result<(), Self::Error> {
            let path = format!("snapshots/{}", snapshot.metadata.project_id);
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

        async fn update_latest_snapshot(
            &self,
            snapshot: ProjectSnapshot,
        ) -> Result<(), Self::Error> {
            let path = format!("snapshots/{}", snapshot.metadata.project_id);
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

    #[async_trait]
    impl ProjectImpl for ProjectLocalRepository {
        type Error = ProjectRepositoryError;

        async fn create_project(&self, project: Project) -> Result<(), Self::Error> {
            let path = format!("project/{}", project.id);
            self.client.upload(path, &project).await?;
            Ok(())
        }

        async fn delete_project(&self, project_id: &str) -> Result<(), Self::Error> {
            let path = format!("project/{}", project_id);
            self.client.delete(path).await?;
            Ok(())
        }

        async fn update_project(&self, project: Project) -> Result<(), Self::Error> {
            let path = format!("project/{}", project.id);
            self.client.upload(path, &project).await?;
            Ok(())
        }
    }

    #[async_trait]
    impl WorkspaceImpl for ProjectLocalRepository {
        type Error = ProjectRepositoryError;

        async fn get_workspace(
            &self,
            workspace_id: &str,
        ) -> Result<Option<Workspace>, Self::Error> {
            let path = format!("workspace/{}", workspace_id);
            let workspace = self.client.download::<Workspace>(path).await?;
            Ok(Some(workspace))
        }

        async fn list_workspace_projects_ids(
            &self,
            workspace_id: &str,
        ) -> Result<Vec<String>, Self::Error> {
            let path = format!("workspace/{}", workspace_id);
            let workspace = self.client.download::<Workspace>(path).await?;
            Ok(workspace.projects)
        }

        async fn create_workspace(&self, workspace: Workspace) -> Result<(), Self::Error> {
            let path = format!("workspace/{}", workspace.id);
            self.client.upload(path, &workspace).await?;
            Ok(())
        }

        async fn update_workspace(&self, workspace: Workspace) -> Result<(), Self::Error> {
            let path = format!("workspace/{}", workspace.id);
            self.client.upload(path, &workspace).await?;
            Ok(())
        }

        async fn delete_workspace(&self, workspace_id: &str) -> Result<(), Self::Error> {
            let path = format!("workspace/{}", workspace_id);
            self.client.delete(path).await?;
            Ok(())
        }
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
