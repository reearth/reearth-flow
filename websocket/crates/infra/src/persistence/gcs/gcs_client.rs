use async_trait::async_trait;
use chrono::{DateTime, Utc};
use google_cloud_storage::client::{Client, ClientConfig};
use google_cloud_storage::http::objects::delete::DeleteObjectRequest;
use google_cloud_storage::http::objects::download::Range;
use google_cloud_storage::http::objects::get::GetObjectRequest;
use google_cloud_storage::http::objects::upload::{Media, UploadObjectRequest, UploadType};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::Debug;
use thiserror::Error;

use crate::persistence::StorageClient;

#[derive(Error, Debug)]
pub enum GcsError {
    #[error(transparent)]
    Auth(#[from] google_cloud_storage::client::google_cloud_auth::error::Error),
    #[error(transparent)]
    Http(#[from] google_cloud_storage::http::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("UTF-8 conversion error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

#[derive(Clone)]
pub struct GcsClient {
    client: Client,
    bucket: String,
}

#[derive(Serialize, Deserialize)]
struct VersionMetadata {
    latest_version: String,
    version_history: BTreeMap<i64, String>, // Timestamp to version path
}

#[async_trait]
impl StorageClient for GcsClient {
    type Error = GcsError;
    async fn upload<T: Serialize + Send + Sync + 'static>(
        &self,
        path: String,
        data: &T,
    ) -> Result<(), GcsError> {
        let upload_type = UploadType::Simple(Media::new(path));
        let bytes = serde_json::to_string(data)?;
        let _uploaded = self
            .client
            .upload_object(
                &UploadObjectRequest {
                    bucket: self.bucket.clone(),
                    ..Default::default()
                },
                bytes,
                &upload_type,
            )
            .await?;
        Ok(())
    }

    async fn download<T: for<'de> Deserialize<'de> + Send + 'static>(
        &self,
        path: String,
    ) -> Result<T, GcsError> {
        let bytes = self
            .client
            .download_object(
                &GetObjectRequest {
                    bucket: self.bucket.clone(),
                    object: path,
                    ..Default::default()
                },
                &Range::default(),
            )
            .await?;
        let src = String::from_utf8(bytes)?;
        let data = serde_json::from_str(&src)?;
        Ok(data)
    }

    async fn delete(&self, path: String) -> Result<(), GcsError> {
        self.client
            .delete_object(&DeleteObjectRequest {
                bucket: self.bucket.clone(),
                object: path,
                ..Default::default()
            })
            .await?;
        Ok(())
    }

    async fn upload_versioned<T: Serialize + Send + Sync + 'static>(
        &self,
        path: String,
        data: &T,
    ) -> Result<String, GcsError> {
        let timestamp = Utc::now().timestamp_millis();
        let versioned_path = format!("{}_v{}", path, timestamp);

        // Upload the data
        self.upload(versioned_path.clone(), data).await?;

        // Update metadata
        let metadata_path = format!("{}_metadata", path);
        let mut metadata = match self
            .download::<VersionMetadata>(metadata_path.clone())
            .await
        {
            Ok(existing_metadata) => existing_metadata,
            Err(_) => VersionMetadata {
                latest_version: versioned_path.clone(),
                version_history: BTreeMap::new(),
            },
        };

        metadata.latest_version = versioned_path.clone();
        metadata
            .version_history
            .insert(timestamp, versioned_path.clone());

        // Limit version history to last 100 versions
        if metadata.version_history.len() > 100 {
            while metadata.version_history.len() > 100 {
                if let Some(oldest) = metadata.version_history.keys().next().cloned() {
                    metadata.version_history.remove(&oldest);
                } else {
                    break;
                }
            }
        }

        self.upload(metadata_path, &metadata).await?;

        Ok(versioned_path)
    }

    async fn update_latest_versioned<T: Serialize + Send + Sync + 'static>(
        &self,
        path: String,
        data: &T,
    ) -> Result<(), GcsError> {
        // Get the metadata to find the latest version
        let metadata_path = format!("{}_metadata", path);
        let metadata = self.download::<VersionMetadata>(metadata_path).await?;

        // Update the data at the latest version path
        self.upload(metadata.latest_version, data).await?;

        Ok(())
    }

    async fn get_latest_version(&self, path_prefix: &str) -> Result<Option<String>, GcsError> {
        let metadata_path = format!("{}_metadata", path_prefix);
        match self.download::<VersionMetadata>(metadata_path).await {
            Ok(metadata) => Ok(Some(metadata.latest_version)),
            Err(_) => Ok(None),
        }
    }

    async fn get_version_at(
        &self,
        path_prefix: &str,
        timestamp: DateTime<Utc>,
    ) -> Result<Option<String>, GcsError> {
        let metadata_path = format!("{}_metadata", path_prefix);
        match self.download::<VersionMetadata>(metadata_path).await {
            Ok(metadata) => {
                let target_timestamp = timestamp.timestamp_millis();
                Ok(metadata
                    .version_history
                    .range(..=target_timestamp)
                    .next_back()
                    .map(|(_, path)| path.clone()))
            }
            Err(_) => Ok(None),
        }
    }

    async fn list_versions(
        &self,
        path_prefix: &str,
        limit: Option<usize>,
    ) -> Result<Vec<(DateTime<Utc>, String)>, GcsError> {
        let metadata_path = format!("{}_metadata", path_prefix);
        match self.download::<VersionMetadata>(metadata_path).await {
            Ok(metadata) => {
                let total_versions = metadata.version_history.len();
                let skip_count = total_versions.saturating_sub(limit.unwrap_or(total_versions));
                Ok(metadata
                    .version_history
                    .iter()
                    .skip(skip_count)
                    .filter_map(|(&timestamp, path)| {
                        DateTime::<Utc>::from_timestamp_millis(timestamp)
                            .map(|dt| (dt, path.clone()))
                    })
                    .collect())
            }
            Err(_) => Ok(vec![]),
        }
    }

    async fn download_latest<T: for<'de> Deserialize<'de> + Send + 'static>(
        &self,
        path_prefix: &str,
    ) -> Result<Option<T>, GcsError> {
        let latest_version = self.get_latest_version(path_prefix).await?;

        match latest_version {
            Some(version_path) => {
                let data = self.download(version_path).await?;
                Ok(Some(data))
            }
            None => Ok(None), // No versions found
        }
    }
}

impl GcsClient {
    pub async fn new(bucket: String) -> Result<Self, GcsError> {
        let config = ClientConfig::default().with_auth().await?;
        let client = Client::new(config);
        Ok(GcsClient { client, bucket })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;
    use mockall::predicate::*;

    mock! {
        pub GcsClientMock {}
        #[async_trait]
        impl StorageClient for GcsClientMock {
            type Error = GcsError;
            async fn upload<T: Serialize + Send + Sync + 'static>(&self, path: String, data: &T) -> Result<(), GcsError>;
            async fn download<T: for<'de> Deserialize<'de> + Send + 'static>(&self, path: String) -> Result<T, GcsError>;
            async fn delete(&self, path: String) -> Result<(), GcsError>;
            async fn upload_versioned<T: Serialize + Send + Sync + 'static>(&self, path: String, data: &T) -> Result<String, GcsError>;
            async fn update_latest_versioned<T: Serialize + Send + Sync + 'static>(&self, path: String, data: &T) -> Result<(), GcsError>;
            async fn get_latest_version(&self, path_prefix: &str) -> Result<Option<String>, GcsError>;
            async fn get_version_at(&self, path_prefix: &str, timestamp: DateTime<Utc>) -> Result<Option<String>, GcsError>;
            async fn list_versions(&self, path_prefix: &str, limit: Option<usize>) -> Result<Vec<(DateTime<Utc>, String)>, GcsError>;
            async fn download_latest<T: for<'de> Deserialize<'de> + Send + 'static>(&self, path_prefix: &str) -> Result<Option<T>, GcsError>;
        }
    }

    #[tokio::test]
    async fn test_upload_versioned() {
        let mut mock = MockGcsClientMock::new();
        mock.expect_upload_versioned()
            .with(eq("test_path".to_string()), always())
            .returning(|path: String, _: &String| Ok(format!("{}_v1234567890", path)));

        let result = mock
            .upload_versioned("test_path".to_string(), &"test_data".to_string())
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test_path_v1234567890".to_string());
    }

    #[tokio::test]
    async fn test_update_versioned() {
        let mut mock = MockGcsClientMock::new();
        mock.expect_update_latest_versioned()
            .with(eq("test_path".to_string()), always())
            .returning(|_: String, _: &String| Ok(()));

        let result = mock
            .update_latest_versioned("test_path".to_string(), &"updated_data".to_string())
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_latest_version() {
        let mut mock = MockGcsClientMock::new();
        mock.expect_get_latest_version()
            .with(eq("test_path"))
            .returning(|path: &str| Ok(Some(format!("{}_v1234567890", path))));

        let result = mock.get_latest_version("test_path").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("test_path_v1234567890".to_string()));
    }

    #[tokio::test]
    async fn test_download_latest() {
        let mut mock = MockGcsClientMock::new();
        mock.expect_download_latest::<String>()
            .with(eq("test_path"))
            .returning(|_: &str| Ok(Some("test_data".to_string())));

        let result: Result<Option<String>, GcsError> = mock.download_latest("test_path").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("test_data".to_string()));
    }
}
