use chrono::{DateTime, Utc};
use lru::LruCache;
use serde::Deserialize;
use serde::{de::DeserializeOwned, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::num::NonZero;
use std::path::{Path, PathBuf};
use tokio::fs::{self, OpenOptions};
use tokio::io::{self, AsyncReadExt};
use tokio::io::{AsyncWriteExt, BufReader, BufWriter};
use tokio::sync::Mutex;

pub struct LocalClient {
    base_path: PathBuf,
    file_locks: Mutex<HashMap<PathBuf, ()>>,
    cache: Mutex<LruCache<String, Vec<u8>>>,
}

#[derive(Serialize, Deserialize)]
struct VersionMetadata {
    latest_version: String,
    version_history: BTreeMap<i64, String>, // Timestamp to version path
}

impl LocalClient {
    pub async fn new<P: AsRef<Path>>(base_path: P) -> io::Result<Self> {
        let base_path = base_path.as_ref().to_path_buf();
        fs::create_dir_all(&base_path).await?;
        Ok(Self {
            base_path,
            file_locks: Mutex::new(HashMap::new()),
            cache: Mutex::new(LruCache::new(NonZero::new(100).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "Invalid cache size")
            })?)),
        })
    }

    fn get_full_path(&self, path: &str) -> PathBuf {
        let sanitized_path = Path::new(path)
            .components()
            .filter(|c| matches!(c, std::path::Component::Normal(_)))
            .collect::<PathBuf>();
        self.base_path.join(sanitized_path)
    }

    async fn lock_file(&self, path: &Path) {
        let mut locks = self.file_locks.lock().await;
        locks.entry(path.to_path_buf()).or_insert(());
    }

    async fn unlock_file(&self, path: &PathBuf) {
        let mut locks = self.file_locks.lock().await;
        locks.remove(path);
    }
}

impl LocalClient {
    pub async fn upload<T: Serialize + Send + Sync>(
        &self,
        path: String,
        data: &T,
        overwrite: bool,
    ) -> io::Result<()> {
        let full_path = self.get_full_path(&path);
        self.lock_file(&full_path).await;

        let result = async {
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent).await?;
            }
            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .append(!overwrite)
                .open(&full_path)
                .await?;
            let mut writer = BufWriter::new(file);
            let serialized = serde_json::to_vec(data)?;
            writer.write_all(&serialized).await?;
            writer.flush().await?;

            // Update cache only if the path already exists in the cache
            let mut cache = self.cache.lock().await;
            if cache.contains(&path) {
                let updated_content = fs::read(&full_path).await?;
                cache.put(path.clone(), updated_content);
            }

            Ok(())
        }
        .await;

        self.unlock_file(&full_path).await;
        result
    }

    pub async fn download<T: DeserializeOwned + Send>(&self, path: String) -> io::Result<T> {
        let full_path = self.get_full_path(&path);
        self.lock_file(&full_path).await;

        let result = async {
            // Check if the data is in the cache
            let mut cache = self.cache.lock().await;
            if let Some(cached_data) = cache.get(&path) {
                return Ok(serde_json::from_slice(cached_data)?);
            }

            // If not in cache, read from file
            let file = fs::File::open(&full_path).await?;
            let mut reader = BufReader::new(file);
            let mut contents = Vec::new();
            reader.read_to_end(&mut contents).await?;

            // Store in cache
            cache.put(path.clone(), contents.clone());

            let data: T = serde_json::from_slice(&contents)?;
            Ok(data)
        }
        .await;

        self.unlock_file(&full_path).await;
        result
    }

    pub async fn upload_versioned<T: Serialize + Send + Sync>(
        &self,
        path: String,
        data: &T,
    ) -> io::Result<String> {
        let timestamp = Utc::now().timestamp_millis();
        let versioned_path = format!("{}_v{}", path, timestamp);

        // Upload the data
        self.upload(versioned_path.clone(), data, true).await?;

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

        self.upload(metadata_path, &metadata, true).await?;

        Ok(versioned_path)
    }

    pub async fn update_versioned<T: Serialize + Send + Sync>(
        &self,
        path: String,
        data: &T,
    ) -> io::Result<()> {
        let metadata_path = format!("{}_metadata", path);
        let metadata = self.download::<VersionMetadata>(metadata_path).await?;

        self.upload(metadata.latest_version, data, true).await
    }

    pub async fn get_latest_version(&self, path_prefix: &str) -> io::Result<Option<String>> {
        let metadata_path = format!("{}_metadata", path_prefix);
        match self.download::<VersionMetadata>(metadata_path).await {
            Ok(metadata) => Ok(Some(metadata.latest_version)),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn get_version_at(
        &self,
        path_prefix: &str,
        timestamp: DateTime<Utc>,
    ) -> io::Result<Option<String>> {
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
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn list_versions(
        &self,
        path_prefix: &str,
        limit: Option<usize>,
    ) -> io::Result<Vec<(DateTime<Utc>, String)>> {
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
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(vec![]),
            Err(e) => Err(e),
        }
    }

    pub async fn download_latest<T: DeserializeOwned + Send>(
        &self,
        path_prefix: &str,
    ) -> io::Result<Option<T>> {
        let latest_version = self.get_latest_version(path_prefix).await?;

        match latest_version {
            Some(version_path) => {
                let data = self.download(version_path).await?;
                Ok(Some(data))
            }
            None => Ok(None), // No versions found
        }
    }

    pub async fn delete(&self, path: &str) -> io::Result<()> {
        let full_path = self.get_full_path(path);
        fs::remove_file(full_path).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use serde::{Deserialize, Serialize};
    use tempfile::tempdir;

    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct TestData {
        field1: String,
        field2: i32,
    }

    #[tokio::test]
    async fn test_upload_and_download() -> io::Result<()> {
        let temp_dir = tempdir()?;
        let base_path = temp_dir.path().to_path_buf();
        let client = LocalClient::new(base_path).await?;

        let test_data = TestData {
            field1: "test".to_string(),
            field2: 42,
        };

        // Test upload
        client
            .upload("test_file.json".to_string(), &test_data, true)
            .await?;

        // Test download
        let downloaded_data: TestData = client.download("test_file.json".to_string()).await?;

        assert_eq!(test_data, downloaded_data);

        Ok(())
    }

    #[tokio::test]
    async fn test_file_not_found() {
        let temp_dir = tempdir().unwrap();
        let base_path = temp_dir.path().to_path_buf();
        let client = LocalClient::new(base_path).await.unwrap();

        let result: io::Result<TestData> =
            client.download("non_existent_file.json".to_string()).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::NotFound);
    }

    #[tokio::test]
    async fn test_concurrent_access() -> io::Result<()> {
        let temp_dir = tempdir()?;
        let base_path = temp_dir.path().to_path_buf();
        let client = Arc::new(LocalClient::new(base_path).await?);

        let test_data = TestData {
            field1: "test".to_string(),
            field2: 42,
        };

        let test_data_clone = TestData {
            field1: "test".to_string(),
            field2: 42,
        };

        let upload_task = tokio::spawn({
            let client = client.clone();
            async move {
                client
                    .upload("concurrent_test.json".to_string(), &test_data, true)
                    .await
            }
        });

        let download_task = tokio::spawn({
            let client = client.clone();
            async move {
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                client
                    .download::<TestData>("concurrent_test.json".to_string())
                    .await
            }
        });

        upload_task.await??;
        let downloaded_data = download_task.await??;

        assert_eq!(test_data_clone, downloaded_data);

        Ok(())
    }

    #[tokio::test]
    async fn test_upload_and_download_versioned() -> io::Result<()> {
        let temp_dir = tempdir()?;
        let base_path = temp_dir.path().to_path_buf();
        let client = LocalClient::new(base_path).await?;

        let test_data1 = TestData {
            field1: "version1".to_string(),
            field2: 1,
        };
        let test_data2 = TestData {
            field1: "version2".to_string(),
            field2: 2,
        };

        // Upload first version
        let version1_path = client
            .upload_versioned("test_file".to_string(), &test_data1)
            .await?;

        // Upload second version
        let version2_path = client
            .upload_versioned("test_file".to_string(), &test_data2)
            .await?;

        // Download latest version
        let latest_data: TestData = client.download_latest("test_file").await?.unwrap();
        assert_eq!(latest_data, test_data2);

        // Download specific versions
        let data1: TestData = client.download(version1_path).await?;
        let data2: TestData = client.download(version2_path).await?;
        assert_eq!(data1, test_data1);
        assert_eq!(data2, test_data2);

        Ok(())
    }

    #[tokio::test]
    async fn test_update_versioned() -> io::Result<()> {
        let temp_dir = tempdir()?;
        let base_path = temp_dir.path().to_path_buf();
        let client = LocalClient::new(base_path).await?;

        let test_data1 = TestData {
            field1: "initial".to_string(),
            field2: 1,
        };
        let test_data2 = TestData {
            field1: "updated".to_string(),
            field2: 2,
        };

        // Upload initial version
        client
            .upload_versioned("test_file".to_string(), &test_data1)
            .await?;

        // Update the version
        client
            .update_versioned("test_file".to_string(), &test_data2)
            .await?;

        // Download latest version
        let latest_data: TestData = client.download_latest("test_file").await?.unwrap();
        assert_eq!(latest_data, test_data2);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_version_at() -> io::Result<()> {
        let temp_dir = tempdir()?;
        let base_path = temp_dir.path().to_path_buf();
        let client = LocalClient::new(base_path).await?;

        let test_data1 = TestData {
            field1: "version1".to_string(),
            field2: 1,
        };
        let test_data2 = TestData {
            field1: "version2".to_string(),
            field2: 2,
        };

        // Upload first version
        client
            .upload_versioned("test_file".to_string(), &test_data1)
            .await?;

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        let mid_time = Utc::now();
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        // Upload second version
        client
            .upload_versioned("test_file".to_string(), &test_data2)
            .await?;

        // Get version at mid_time
        let version_path = client.get_version_at("test_file", mid_time).await?.unwrap();
        let data: TestData = client.download(version_path).await?;
        assert_eq!(data, test_data1);

        Ok(())
    }

    #[tokio::test]
    async fn test_list_versions() -> io::Result<()> {
        let temp_dir = tempdir()?;
        let base_path = temp_dir.path().to_path_buf();
        let client = LocalClient::new(base_path).await?;

        let test_data = TestData {
            field1: "test".to_string(),
            field2: 1,
        };

        // Upload multiple versions
        for i in 0..5 {
            let mut data = test_data.clone();
            data.field2 = i;
            client
                .upload_versioned("test_file".to_string(), &data)
                .await?;
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }

        // List all versions
        let versions = client.list_versions("test_file", None).await?;
        assert_eq!(versions.len(), 5);

        // List limited versions
        let limited_versions = client.list_versions("test_file", Some(3)).await?;
        assert_eq!(limited_versions.len(), 3);

        Ok(())
    }
}
