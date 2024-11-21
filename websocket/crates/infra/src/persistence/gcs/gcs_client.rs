#[cfg(feature = "gcs-storage")]
use async_trait::async_trait;
#[cfg(feature = "gcs-storage")]
use chrono::{DateTime, Utc};
#[cfg(feature = "gcs-storage")]
use google_cloud_storage::client::{Client, ClientConfig};

use google_cloud_storage::http::objects::{
    delete::DeleteObjectRequest,
    download::Range,
    get::GetObjectRequest,
    upload::{Media, UploadObjectRequest, UploadType},
};
#[cfg(feature = "gcs-storage")]
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

use crate::persistence::StorageClient;

const MAX_VERSIONS: usize = 100;
const COMPACT_THRESHOLD: usize = 150;

#[cfg(feature = "gcs-storage")]
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
    #[error("Invalid timestamp")]
    InvalidTimestamp,
}

#[cfg(feature = "gcs-storage")]
#[derive(Clone)]
pub struct GcsClient {
    client: Client,
    bucket: String,
}

#[cfg(feature = "gcs-storage")]
#[derive(Serialize, Deserialize, Debug)]
struct VersionedData<T> {
    versions: BTreeMap<i64, T>,
}

#[cfg(feature = "gcs-storage")]
impl<T> VersionedData<T> {
    fn new() -> Self {
        Self {
            versions: BTreeMap::new(),
        }
    }

    fn compact(&mut self) {
        if self.versions.len() > MAX_VERSIONS {
            let to_remove: Vec<_> = self
                .versions
                .keys()
                .take(self.versions.len() - MAX_VERSIONS)
                .cloned()
                .collect();
            for key in to_remove {
                self.versions.remove(&key);
            }
        }
    }
}

#[cfg(feature = "gcs-storage")]
#[async_trait]
impl StorageClient for GcsClient {
    type Error = GcsError;

    async fn upload<T: Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static>(
        &self,
        path: &str,
        data: &T,
    ) -> Result<i64, Self::Error> {
        let mut versioned_data = match self.read_file::<T>(path).await {
            Ok(data) => data,
            Err(_) => VersionedData::new(),
        };

        let timestamp = Utc::now().timestamp_millis();
        versioned_data.versions.insert(timestamp, data.clone());

        self.compact_if_needed(path, &mut versioned_data).await?;
        self.write_file(path, &versioned_data).await?;

        Ok(timestamp)
    }

    async fn get_latest_version<T: for<'de> Deserialize<'de> + Clone + Send + 'static>(
        &self,
        path: &str,
    ) -> Result<Option<T>, Self::Error> {
        let versioned_data = self.read_file::<T>(path).await?;
        Ok(versioned_data
            .versions
            .iter()
            .next_back()
            .map(|(_, v)| v.clone()))
    }

    async fn get_version_at<T: for<'de> Deserialize<'de> + Clone + Send + 'static>(
        &self,
        path: &str,
        timestamp: DateTime<Utc>,
    ) -> Result<Option<T>, Self::Error> {
        let versioned_data = self.read_file::<T>(path).await?;
        let target_timestamp = timestamp.timestamp_millis();
        Ok(versioned_data
            .versions
            .range(..=target_timestamp)
            .next_back()
            .map(|(_, v)| v.clone()))
    }

    async fn list_versions(
        &self,
        path: &str,
        limit: Option<usize>,
    ) -> Result<Vec<(DateTime<Utc>, String)>, Self::Error> {
        let versioned_data = self.read_file::<serde_json::Value>(path).await?;
        let versions: Vec<_> = versioned_data
            .versions
            .iter()
            .rev()
            .take(limit.unwrap_or(usize::MAX))
            .map(|(&timestamp, value)| {
                let dt = DateTime::<Utc>::from_timestamp_millis(timestamp)
                    .ok_or(GcsError::InvalidTimestamp)?;
                let value_str = serde_json::to_string(value)?;
                Ok((dt, value_str))
            })
            .collect::<Result<_, GcsError>>()?;

        Ok(versions)
    }

    async fn update_latest_version<
        T: Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static,
    >(
        &self,
        path: &str,
        data: &T,
    ) -> Result<(), Self::Error> {
        let mut versioned_data = self.read_file::<T>(path).await?;

        if let Some((&last_timestamp, _)) = versioned_data.versions.iter().next_back() {
            versioned_data.versions.remove(&last_timestamp);
        }

        let timestamp = Utc::now().timestamp_millis();
        versioned_data.versions.insert(timestamp, data.clone());

        self.write_file(path, &versioned_data).await?;
        Ok(())
    }

    async fn delete_version(
        &self,
        path: &str,
        timestamp: DateTime<Utc>,
    ) -> Result<(), Self::Error> {
        let mut versioned_data = self.read_file::<serde_json::Value>(path).await?;
        versioned_data
            .versions
            .remove(&timestamp.timestamp_millis());
        self.write_file(path, &versioned_data).await?;
        Ok(())
    }
}

#[cfg(feature = "gcs-storage")]
impl GcsClient {
    pub async fn new(bucket: String) -> Result<Self, GcsError> {
        let config = ClientConfig::default().with_auth().await?;
        let client = Client::new(config);
        Ok(GcsClient { client, bucket })
    }

    async fn read_file<T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
    ) -> Result<VersionedData<T>, GcsError> {
        let object_name = format!("{}/data.json", path);
        let bytes = self
            .client
            .download_object(
                &GetObjectRequest {
                    bucket: self.bucket.clone(),
                    object: object_name,
                    ..Default::default()
                },
                &Range::default(),
            )
            .await?;

        let src = String::from_utf8(bytes)?;
        let data = serde_json::from_str(&src)?;
        Ok(data)
    }

    async fn write_file<T: Serialize>(
        &self,
        path: &str,
        data: &VersionedData<T>,
    ) -> Result<(), GcsError> {
        let object_name = format!("{}/data.json", path);
        let bytes = serde_json::to_string(data)?;

        let upload_type = UploadType::Simple(Media::new(object_name.clone()));
        self.client
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

    async fn compact_if_needed<T: Serialize + for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        data: &mut VersionedData<T>,
    ) -> Result<bool, GcsError> {
        if data.versions.len() > COMPACT_THRESHOLD {
            data.compact();
            self.write_file(path, data).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn delete(&self, path: &str) -> Result<(), GcsError> {
        let object_name = format!("{}/data.json", path);
        self.client
            .delete_object(&DeleteObjectRequest {
                bucket: self.bucket.clone(),
                object: object_name,
                ..Default::default()
            })
            .await?;
        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use google_cloud_storage::http::objects::download::Range;
//     use google_cloud_storage::http::objects::get::GetObjectRequest;
//     use google_cloud_storage::http::objects::upload::{Media, UploadObjectRequest, UploadType};
//     use mockall::mock;
//     use mockall::predicate::*;

//     #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
//     struct TestData {
//         field1: String,
//         field2: i32,
//     }

//     mock! {
//         pub Client {
//             async fn download_object<'a>(
//                 &'a self,
//                 request: &'a GetObjectRequest,
//                 range: &'a Range<usize>
//             ) -> Result<Vec<u8>, google_cloud_storage::http::Error>;

//             async fn upload_object<'a>(
//                 &'a self,
//                 request: &'a UploadObjectRequest,
//                 data: String,
//                 upload_type: &'a UploadType
//             ) -> Result<(), google_cloud_storage::http::Error>;
//         }
//     }

//     struct MockGcsClient {
//         client: MockClient,
//         bucket: String,
//     }

//     impl MockGcsClient {
//         fn new() -> Self {
//             Self {
//                 client: MockClient::new(),
//                 bucket: "test-bucket".to_string(),
//             }
//         }
//     }

//     #[tokio::test]
//     async fn test_upload_and_get_latest() -> Result<(), GcsError> {
//         let mut mock = MockGcsClient::new();
//         let test_data = TestData {
//             field1: "test".to_string(),
//             field2: 42,
//         };

//         // Mock initial read
//         mock.client
//             .expect_download_object()
//             .with(always(), always())
//             .returning(|_, _| Ok(serde_json::to_vec(&VersionedData::<TestData>::new()).unwrap()));

//         // Mock write
//         mock.client
//             .expect_upload_object()
//             .with(always(), always(), always())
//             .returning(|_, _, _| Ok(()));

//         // Mock read for get_latest
//         mock.client
//             .expect_download_object()
//             .with(always(), always())
//             .returning(move |_, _| {
//                 let mut data = VersionedData::new();
//                 data.versions
//                     .insert(Utc::now().timestamp_millis(), test_data.clone());
//                 Ok(serde_json::to_vec(&data).unwrap())
//             });

//         let client = GcsClient {
//             client: mock.client,
//             bucket: mock.bucket,
//         };

//         // Test upload
//         client.upload(&test_data).await?;

//         // Test get latest version
//         let downloaded_data = client.get_latest_version::<TestData>().await?.unwrap();
//         assert_eq!(test_data, downloaded_data);

//         Ok(())
//     }

//     #[tokio::test]
//     async fn test_version_management() -> Result<(), GcsError> {
//         let mut mock = MockGcsClient::new();
//         let mut versioned_data = VersionedData::new();
//         let mut timestamps = Vec::new();

//         // Setup mock expectations for multiple uploads
//         for i in 0..5 {
//             let test_data = TestData {
//                 field1: format!("version_{}", i),
//                 field2: i,
//             };
//             let timestamp = Utc::now().timestamp_millis() + i * 1000;
//             timestamps.push(timestamp);
//             versioned_data.versions.insert(timestamp, test_data);

//             mock.client
//                 .expect_download_object()
//                 .returning(move |_, _| Ok(serde_json::to_vec(&versioned_data).unwrap()));

//             mock.client
//                 .expect_upload_object()
//                 .returning(|_, _, _| Ok(()));
//         }

//         let client = GcsClient {
//             client: mock.client,
//             bucket: mock.bucket,
//         };

//         // Test list_versions
//         let versions = client.list_versions(Some(3)).await?;
//         assert_eq!(versions.len(), 3);

//         // Test get_version_at
//         let timestamp = DateTime::<Utc>::from_timestamp_millis(timestamps[2]).unwrap();
//         let version_data = client.get_version_at::<TestData>(timestamp).await?.unwrap();
//         assert_eq!(version_data.field2, 2);

//         Ok(())
//     }

//     #[tokio::test]
//     async fn test_update_latest_version() -> Result<(), GcsError> {
//         let mut mock = MockGcsClient::new();
//         let initial_data = TestData {
//             field1: "initial".to_string(),
//             field2: 1,
//         };
//         let updated_data = TestData {
//             field1: "updated".to_string(),
//             field2: 2,
//         };

//         // Mock initial read
//         mock.client.expect_download_object().returning(move |_, _| {
//             let mut data = VersionedData::new();
//             data.versions
//                 .insert(Utc::now().timestamp_millis(), initial_data.clone());
//             Ok(serde_json::to_vec(&data).unwrap())
//         });

//         // Mock write for update
//         mock.client
//             .expect_upload_object()
//             .returning(|_, _, _| Ok(()));

//         // Mock read for verification
//         mock.client.expect_download_object().returning(move |_, _| {
//             let mut data = VersionedData::new();
//             data.versions
//                 .insert(Utc::now().timestamp_millis(), updated_data.clone());
//             Ok(serde_json::to_vec(&data).unwrap())
//         });

//         let client = GcsClient {
//             client: mock.client,
//             bucket: "test-bucket".to_string(),
//         };

//         // Update latest version
//         client.update_latest_version(&updated_data).await?;

//         // Verify update
//         let latest_data = client.get_latest_version::<TestData>().await?.unwrap();
//         assert_eq!(latest_data, updated_data);

//         Ok(())
//     }

//     #[tokio::test]
//     async fn test_delete_version() -> Result<(), GcsError> {
//         let mut mock = MockGcsClient::new();
//         let test_data = TestData {
//             field1: "test".to_string(),
//             field2: 42,
//         };
//         let timestamp = Utc::now().timestamp_millis();

//         // Mock initial read
//         mock.client.expect_download_object().returning(move |_, _| {
//             let mut data = VersionedData::new();
//             data.versions.insert(timestamp, test_data.clone());
//             Ok(serde_json::to_vec(&data).unwrap())
//         });

//         // Mock write for delete
//         mock.client
//             .expect_upload_object()
//             .returning(|_, _, _| Ok(()));

//         // Mock read for verification
//         mock.client.expect_download_object().returning(|_, _| {
//             let data = VersionedData::<TestData>::new();
//             Ok(serde_json::to_vec(&data).unwrap())
//         });

//         let client = GcsClient {
//             client: mock.client,
//             bucket: "test-bucket".to_string(),
//         };

//         // Delete version
//         let timestamp_dt = DateTime::<Utc>::from_timestamp_millis(timestamp).unwrap();
//         client.delete_version(timestamp_dt).await?;

//         // Verify deletion
//         let latest = client.get_latest_version::<TestData>().await?;
//         assert!(latest.is_none());

//         Ok(())
//     }

//     #[tokio::test]
//     async fn test_compaction() -> Result<(), GcsError> {
//         let mut mock = MockGcsClient::new();
//         let mut versioned_data = VersionedData::new();

//         // Setup initial data with more than COMPACT_THRESHOLD versions
//         for i in 0..COMPACT_THRESHOLD + 10 {
//             let test_data = TestData {
//                 field1: format!("version_{}", i),
//                 field2: i as i32,
//             };
//             versioned_data
//                 .versions
//                 .insert(Utc::now().timestamp_millis() + i as i64, test_data);
//         }

//         // Mock read/write operations
//         mock.client
//             .expect_download_object()
//             .returning(move |_, _| Ok(serde_json::to_vec(&versioned_data).unwrap()));

//         mock.client
//             .expect_upload_object()
//             .returning(|_, _, _| Ok(()));

//         let client = GcsClient {
//             client: mock.client,
//             bucket: "test-bucket".to_string(),
//         };

//         // Upload one more version to trigger compaction
//         let new_data = TestData {
//             field1: "new".to_string(),
//             field2: 999,
//         };
//         client.upload(&new_data).await?;

//         // Verify compaction
//         let versions = client.list_versions(None).await?;
//         assert!(versions.len() <= MAX_VERSIONS);

//         Ok(())
//     }
// }
