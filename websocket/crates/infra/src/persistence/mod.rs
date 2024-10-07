use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[async_trait]
pub trait StorageClient {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn upload<T: Serialize + Send + Sync + 'static>(
        &self,
        path: String,
        data: &T,
    ) -> Result<(), Self::Error>;

    async fn download<T: for<'de> Deserialize<'de> + Send + 'static>(
        &self,
        path: String,
    ) -> Result<T, Self::Error>;

    async fn delete(&self, path: String) -> Result<(), Self::Error>;

    async fn upload_versioned<T: Serialize + Send + Sync + 'static>(
        &self,
        path: String,
        data: &T,
    ) -> Result<String, Self::Error>;

    async fn update_versioned<T: Serialize + Send + Sync + 'static>(
        &self,
        path: String,
        data: &T,
    ) -> Result<(), Self::Error>;

    async fn get_latest_version(&self, path_prefix: &str) -> Result<Option<String>, Self::Error>;

    async fn get_version_at(
        &self,
        path_prefix: &str,
        timestamp: DateTime<Utc>,
    ) -> Result<Option<String>, Self::Error>;

    async fn list_versions(
        &self,
        path_prefix: &str,
        limit: Option<usize>,
    ) -> Result<Vec<(DateTime<Utc>, String)>, Self::Error>;

    async fn download_latest<T: for<'de> Deserialize<'de> + Send + 'static>(
        &self,
        path_prefix: &str,
    ) -> Result<Option<T>, Self::Error>;
}

pub mod gcs;
pub mod local_storage;
pub mod project_repository;
pub mod redis;
