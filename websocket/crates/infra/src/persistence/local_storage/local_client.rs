use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs::{self, OpenOptions};
use tokio::io::{self, AsyncReadExt};
use tokio::io::{AsyncWriteExt, BufReader, BufWriter};
use tokio::sync::Mutex;

pub struct LocalClient {
    base_path: PathBuf,
    file_locks: Mutex<HashMap<PathBuf, ()>>,
}

impl LocalClient {
    pub async fn new<P: AsRef<Path>>(base_path: P) -> io::Result<Self> {
        let base_path = base_path.as_ref().to_path_buf();
        fs::create_dir_all(&base_path).await?;
        Ok(Self {
            base_path,
            file_locks: Mutex::new(HashMap::new()),
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
                .append(true)
                .open(&full_path)
                .await?;
            let mut writer = BufWriter::new(file);
            let serialized = serde_json::to_vec(data)?;
            writer.write_all(&serialized).await?;
            writer.flush().await?;
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
            let file = fs::File::open(&full_path).await?;
            let mut reader = BufReader::new(file);
            let mut contents = Vec::new();
            reader.read_to_end(&mut contents).await?;
            let data: T = serde_json::from_slice(&contents)?;
            Ok(data)
        }
        .await;

        self.unlock_file(&full_path).await;
        result
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use serde::{Deserialize, Serialize};
    use tempfile::tempdir;

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
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
            .upload("test_file.json".to_string(), &test_data)
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
                    .upload("concurrent_test.json".to_string(), &test_data)
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
}
