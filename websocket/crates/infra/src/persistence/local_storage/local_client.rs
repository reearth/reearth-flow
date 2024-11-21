use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use thiserror::Error;
use tokio::fs::{self, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};

use crate::persistence::StorageClient;

const MAX_VERSIONS: usize = 100;
const COMPACT_THRESHOLD: usize = 150;

#[derive(Error, Debug)]
pub enum LocalStorageError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Invalid timestamp")]
    InvalidTimestamp,
}

pub struct LocalClient {
    base_dir: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
struct VersionedData<T> {
    versions: BTreeMap<i64, T>,
}

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

impl LocalClient {
    pub async fn new<P: AsRef<std::path::Path>>(base_dir: P) -> Result<Self, LocalStorageError> {
        let base_dir = base_dir.as_ref().to_path_buf();

        if !base_dir.exists() {
            fs::create_dir_all(&base_dir).await?;
        }

        Ok(Self { base_dir })
    }

    fn get_file_path(&self, path: &str) -> PathBuf {
        let full_path = self.base_dir.join(path);

        if let Some(parent) = full_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).unwrap();
            }
        }

        full_path
    }

    async fn read_file<T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
    ) -> Result<VersionedData<T>, LocalStorageError> {
        let file_path = self.get_file_path(path);

        if !file_path.exists() {
            return Ok(VersionedData::new());
        }

        let file = OpenOptions::new().read(true).open(&file_path).await?;
        let mut reader = BufReader::new(file);
        let mut contents = Vec::new();
        reader.read_to_end(&mut contents).await?;
        let data = serde_json::from_slice(&contents)?;
        Ok(data)
    }

    async fn write_file<T: Serialize>(
        &self,
        path: &str,
        data: &VersionedData<T>,
    ) -> Result<(), LocalStorageError> {
        let file_path = self.get_file_path(path);
        let temp_path = file_path.with_extension("tmp");

        if let Some(parent) = file_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).await?;
            }
        }

        let temp_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&temp_path)
            .await?;

        let mut writer = BufWriter::new(temp_file);
        let serialized = serde_json::to_vec(data)?;
        writer.write_all(&serialized).await?;
        writer.flush().await?;

        fs::rename(&temp_path, &file_path).await?;
        Ok(())
    }

    async fn compact_if_needed<T: Serialize + for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        data: &mut VersionedData<T>,
    ) -> Result<bool, LocalStorageError> {
        if data.versions.len() > COMPACT_THRESHOLD {
            data.compact();
            self.write_file(path, data).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn delete(&self, path: &str) -> Result<(), LocalStorageError> {
        let file_path = self.get_file_path(path);
        fs::remove_file(&file_path).await?;
        Ok(())
    }
}

#[async_trait]
impl StorageClient for LocalClient {
    type Error = LocalStorageError;

    async fn upload<T: Serialize + for<'de> Deserialize<'de> + Clone + Send + Sync + 'static>(
        &self,
        path: &str,
        data: &T,
    ) -> Result<i64, Self::Error> {
        let mut versioned_data = self.read_file::<T>(path).await?;
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
                    .ok_or(LocalStorageError::InvalidTimestamp)?;
                let value_str = serde_json::to_string(value)?;
                Ok((dt, value_str))
            })
            .collect::<Result<_, LocalStorageError>>()?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct TestData {
        field1: String,
        field2: i32,
    }

    async fn create_test_client() -> (LocalClient, tempfile::TempDir) {
        let temp_dir = tempdir().unwrap();
        let client = LocalClient::new(temp_dir.path()).await.unwrap();
        (client, temp_dir)
    }

    #[tokio::test]
    async fn test_upload_and_get_latest() -> Result<(), LocalStorageError> {
        let (client, _temp_dir) = create_test_client().await;
        let test_data = TestData {
            field1: "test".to_string(),
            field2: 42,
        };

        // Test upload
        let _timestamp = client.upload("test.json", &test_data).await?;

        // Test get latest version
        let downloaded_data = client
            .get_latest_version::<TestData>("test.json")
            .await?
            .unwrap();
        assert_eq!(test_data, downloaded_data);

        Ok(())
    }

    #[tokio::test]
    async fn test_file_organization() -> Result<(), LocalStorageError> {
        let (client, temp_dir) = create_test_client().await;
        let test_data = TestData {
            field1: "test".to_string(),
            field2: 42,
        };

        // Test different paths
        client.upload("project1/data.json", &test_data).await?;
        client.upload("project2/data.json", &test_data).await?;
        client
            .upload("project1/subfolder/data.json", &test_data)
            .await?;

        // Verify file and directory creation
        assert!(temp_dir.path().join("project1").exists());
        assert!(temp_dir.path().join("project2").exists());
        assert!(temp_dir.path().join("project1/subfolder").exists());
        assert!(temp_dir.path().join("project1/data.json").exists());
        assert!(temp_dir.path().join("project2/data.json").exists());
        assert!(temp_dir
            .path()
            .join("project1/subfolder/data.json")
            .exists());

        Ok(())
    }

    #[tokio::test]
    async fn test_version_management() -> Result<(), LocalStorageError> {
        let (client, _temp_dir) = create_test_client().await;
        let path = "test_versions.json";

        // Create multiple versions
        let mut timestamps = Vec::new();
        for i in 0..5 {
            let test_data = TestData {
                field1: format!("version_{}", i),
                field2: i,
            };
            let timestamp = client.upload(path, &test_data).await?;
            timestamps.push(timestamp);
        }

        // Test list_versions
        let versions = client.list_versions(path, Some(3)).await?;
        assert_eq!(versions.len(), 3);

        // Test get_version_at
        let timestamp = DateTime::<Utc>::from_timestamp_millis(timestamps[2]).unwrap();
        let version_data = client
            .get_version_at::<TestData>(path, timestamp)
            .await?
            .unwrap();
        assert_eq!(version_data.field2, 2);

        Ok(())
    }

    #[tokio::test]
    async fn test_update_latest_version() -> Result<(), LocalStorageError> {
        let (client, _temp_dir) = create_test_client().await;
        let path = "test_update.json";

        let initial_data = TestData {
            field1: "initial".to_string(),
            field2: 1,
        };
        let updated_data = TestData {
            field1: "updated".to_string(),
            field2: 2,
        };

        // Upload initial version
        client.upload(path, &initial_data).await?;

        // Update latest version
        client.update_latest_version(path, &updated_data).await?;

        // Verify update
        let latest_data = client.get_latest_version::<TestData>(path).await?.unwrap();
        assert_eq!(latest_data, updated_data);

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_version() -> Result<(), LocalStorageError> {
        let (client, _temp_dir) = create_test_client().await;
        let path = "test_delete.json".to_string();

        // Create multiple versions
        let mut timestamps = Vec::new();
        for i in 0..3 {
            let test_data = TestData {
                field1: format!("version_{}", i),
                field2: i,
            };
            let timestamp = client.upload(&path, &test_data).await?;
            timestamps.push(timestamp);
        }

        // Delete middle version
        let timestamp = DateTime::<Utc>::from_timestamp_millis(timestamps[1]).unwrap();
        client.delete_version(&path, timestamp).await?;

        // Verify versions
        let versions = client.list_versions(&path, None).await?;
        assert_eq!(versions.len(), 2);

        Ok(())
    }

    #[tokio::test]
    async fn test_compaction() -> Result<(), LocalStorageError> {
        let (client, _temp_dir) = create_test_client().await;
        let path = "test_compact.json".to_string();

        // Create more than COMPACT_THRESHOLD versions
        for i in 0..COMPACT_THRESHOLD + 10 {
            let test_data = TestData {
                field1: format!("version_{}", i),
                field2: i as i32,
            };
            client.upload(&path, &test_data).await?;
        }

        // Verify compaction
        let versions = client.list_versions(&path, None).await?;
        assert!(versions.len() <= MAX_VERSIONS);

        Ok(())
    }

    #[tokio::test]
    async fn test_empty_file() -> Result<(), LocalStorageError> {
        let (client, _temp_dir) = create_test_client().await;
        let path = "nonexistent.json".to_string();

        // Try to get latest version from nonexistent file
        let result = client.get_latest_version::<TestData>(&path).await?;
        assert!(result.is_none());

        Ok(())
    }
}
