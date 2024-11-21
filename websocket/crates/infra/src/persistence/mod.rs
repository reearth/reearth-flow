use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub mod editing_session;
pub mod event_handler;
pub mod gcs;
pub mod local_storage;
pub mod project_repository;
pub mod redis;
pub mod repository;

#[cfg(feature = "gcs-storage")]
pub use project_repository::gcs::ProjectGcsRepository;
#[cfg(feature = "local-storage")]
pub use project_repository::local::ProjectLocalRepository;

#[async_trait]
pub trait StorageClient {
    type Error;

    async fn upload<T: Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static>(
        &self,
        path: &str,
        data: &T,
    ) -> Result<i64, Self::Error>;

    async fn get_latest_version<T: for<'de> Deserialize<'de> + Clone + Send + 'static>(
        &self,
        path: &str,
    ) -> Result<Option<T>, Self::Error>;

    async fn get_version_at<T: for<'de> Deserialize<'de> + Clone + Send + 'static>(
        &self,
        path: &str,
        timestamp: DateTime<Utc>,
    ) -> Result<Option<T>, Self::Error>;

    async fn list_versions(
        &self,
        path: &str,
        limit: Option<usize>,
    ) -> Result<Vec<(DateTime<Utc>, String)>, Self::Error>;

    async fn update_latest_version<
        T: Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static,
    >(
        &self,
        path: &str,
        data: &T,
    ) -> Result<(), Self::Error>;

    async fn delete_version(&self, path: &str, timestamp: DateTime<Utc>)
        -> Result<(), Self::Error>;
}
