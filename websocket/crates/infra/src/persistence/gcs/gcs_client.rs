use chrono::{DateTime, Utc};
use google_cloud_storage::client::{Client, ClientConfig};
use google_cloud_storage::http::objects::delete::DeleteObjectRequest;
use google_cloud_storage::http::objects::download::Range;
use google_cloud_storage::http::objects::get::GetObjectRequest;
use google_cloud_storage::http::objects::upload::{Media, UploadObjectRequest, UploadType};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

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

impl GcsClient {
    pub async fn new(bucket: String) -> Result<Self, GcsError> {
        let config = ClientConfig::default().with_auth().await?;
        let client = Client::new(config);
        Ok(GcsClient { client, bucket })
    }

    pub async fn upload<T: Serialize>(&self, path: String, data: &T) -> Result<(), GcsError> {
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

    pub async fn download<T: for<'de> Deserialize<'de>>(
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

    pub async fn delete(&self, path: String) -> Result<(), GcsError> {
        self.client
            .delete_object(&DeleteObjectRequest {
                bucket: self.bucket.clone(),
                object: path,
                ..Default::default()
            })
            .await?;
        Ok(())
    }

    pub async fn upload_versioned<T: Serialize>(
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
            let oldest = *metadata.version_history.keys().next().unwrap();
            metadata.version_history.remove(&oldest);
        }

        self.upload(metadata_path, &metadata).await?;

        Ok(versioned_path)
    }

    pub async fn get_latest_version(&self, path_prefix: &str) -> Result<Option<String>, GcsError> {
        let metadata_path = format!("{}_metadata", path_prefix);
        match self.download::<VersionMetadata>(metadata_path).await {
            Ok(metadata) => Ok(Some(metadata.latest_version)),
            Err(_) => Ok(None),
        }
    }

    pub async fn get_version_at(
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

    pub async fn list_versions(
        &self,
        path_prefix: &str,
        limit: Option<usize>,
    ) -> Result<Vec<(DateTime<Utc>, String)>, GcsError> {
        let metadata_path = format!("{}_metadata", path_prefix);
        match self.download::<VersionMetadata>(metadata_path).await {
            Ok(metadata) => {
                let mut versions: Vec<_> = metadata
                    .version_history
                    .iter()
                    .rev()
                    .take(limit.unwrap_or(usize::MAX))
                    .map(|(&timestamp, path)| {
                        (
                            DateTime::<Utc>::from_timestamp_millis(timestamp).unwrap(),
                            path.clone(),
                        )
                    })
                    .collect();
                versions.sort_by_key(|&(timestamp, _)| timestamp);
                Ok(versions)
            }
            Err(_) => Ok(vec![]),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct VersionMetadata {
    latest_version: String,
    version_history: BTreeMap<i64, String>, // Timestamp to version path
}
