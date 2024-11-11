use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Trait defining storage operations for persisting and retrieving data
#[async_trait]
pub trait StorageClient {
    /// The error type returned by storage operations
    type Error: std::error::Error + Send + Sync + 'static;

    /// Uploads data to storage at the specified path
    async fn upload<T: Serialize + Send + Sync + 'static>(
        &self,
        path: String,
        data: &T,
    ) -> Result<(), Self::Error>;

    /// Downloads and deserializes data from the specified path
    async fn download<T: for<'de> Deserialize<'de> + Send + 'static>(
        &self,
        path: String,
    ) -> Result<T, Self::Error>;

    /// Deletes data at the specified path
    async fn delete(&self, path: String) -> Result<(), Self::Error>;

    /// Uploads a new version of data and returns the versioned path
    async fn upload_versioned<T: Serialize + Send + Sync + 'static>(
        &self,
        path: String,
        data: &T,
    ) -> Result<String, Self::Error>;

    /// Updates the latest version of data at the specified path
    async fn update_latest_versioned<T: Serialize + Send + Sync + 'static>(
        &self,
        path: String,
        data: &T,
    ) -> Result<(), Self::Error>;

    /// Gets the path to the latest version of data
    async fn get_latest_version(&self, path_prefix: &str) -> Result<Option<String>, Self::Error>;

    /// Gets the path to the version of data at a specific timestamp
    async fn get_version_at(
        &self,
        path_prefix: &str,
        timestamp: DateTime<Utc>,
    ) -> Result<Option<String>, Self::Error>;

    /// Lists versions of data with optional limit
    async fn list_versions(
        &self,
        path_prefix: &str,
        limit: Option<usize>,
    ) -> Result<Vec<(DateTime<Utc>, String)>, Self::Error>;

    /// Downloads and deserializes the latest version of data
    async fn download_latest<T: for<'de> Deserialize<'de> + Send + 'static>(
        &self,
        path_prefix: &str,
    ) -> Result<Option<T>, Self::Error>;
}

pub mod editing_session;
pub mod gcs;
pub mod local_storage;
pub mod project_repository;
pub mod redis;
pub mod repository;
