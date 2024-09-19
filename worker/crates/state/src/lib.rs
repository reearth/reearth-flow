use serde::{Deserialize, Serialize};
use std::{
    io::{Error, ErrorKind, Result},
    path::{Path, PathBuf},
    sync::Arc,
};

use reearth_flow_common::str::remove_trailing_slash;
use reearth_flow_common::uri::Uri;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_storage::storage::Storage;

#[derive(Debug, Clone)]
pub struct State {
    storage: Arc<Storage>,
    root: PathBuf,
}

impl State {
    pub fn new(root: &Uri, storage_resolver: &StorageResolver) -> Result<Self> {
        let storage = storage_resolver
            .resolve(root)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(Self {
            storage,
            root: Path::new(
                remove_trailing_slash(root.path().to_str().unwrap_or_default()).as_str(),
            )
            .to_path_buf(),
        })
    }

    pub async fn save<T>(&self, obj: &T, id: &str) -> Result<()>
    where
        for<'de> T: Serialize + Deserialize<'de>,
    {
        let s = self.object_to_string(obj)?;
        let content = bytes::Bytes::from(s);
        let p = self.id_to_location(id, "json");
        self.storage
            .put(p.as_path(), content)
            .await
            .map_err(|e| Error::new(ErrorKind::Other, e))
    }

    pub fn save_sync<T>(&self, obj: &T, id: &str) -> Result<()>
    where
        for<'de> T: Serialize + Deserialize<'de>,
    {
        let s = self.object_to_string(obj)?;
        let content = bytes::Bytes::from(s);
        let p = self.id_to_location(id, "json");
        self.storage
            .put_sync(p.as_path(), content)
            .map_err(|e| Error::new(ErrorKind::Other, e))
    }

    pub async fn append<T>(&self, obj: &T, id: &str) -> Result<()>
    where
        for<'de> T: Serialize + Deserialize<'de>,
    {
        let s = self.object_to_string(obj)?;
        let content = bytes::Bytes::from(s + "\n");
        let p = self.id_to_location(id, "jsonl");
        self.storage
            .append(p.as_path(), content)
            .await
            .map_err(|e| Error::new(ErrorKind::Other, e))
    }

    pub fn append_sync<T>(&self, obj: &T, id: &str) -> Result<()>
    where
        for<'de> T: Serialize + Deserialize<'de>,
    {
        let s = self.object_to_string(obj)?;
        let content = bytes::Bytes::from(s + "\n");
        let p = self.id_to_location(id, "jsonl");
        self.storage
            .append_sync(p.as_path(), content)
            .map_err(|e| Error::new(ErrorKind::Other, e))
    }

    pub async fn get<T>(&self, id: &str) -> Result<T>
    where
        for<'de> T: Deserialize<'de>,
    {
        let result = self
            .storage
            .get(self.id_to_location(id, "json").as_path())
            .await?;
        let byte = result.bytes().await?;
        let content =
            String::from_utf8(byte.to_vec()).map_err(|e| Error::new(ErrorKind::Other, e))?;
        self.string_to_object(content.as_str())
    }

    pub async fn delete(&self, id: &str) -> Result<()> {
        self.storage
            .delete(self.id_to_location(id, "json").as_path())
            .await
            .map_err(|e| Error::new(ErrorKind::Other, e))
    }

    fn string_to_object<T>(&self, s: &str) -> Result<T>
    where
        for<'de> T: Deserialize<'de>,
    {
        serde_json::from_str(s).map_err(|err| Error::new(ErrorKind::Other, err))
    }

    fn id_to_location(&self, id: &str, ext: &str) -> PathBuf {
        PathBuf::new()
            .join(self.root.clone())
            .join(format!("{}.{}", id, ext))
    }

    fn object_to_string<T: Serialize>(&self, obj: &T) -> Result<String> {
        serde_json::to_string(obj).map_err(|err| Error::new(ErrorKind::Other, err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct Data {
        x: i32,
    }

    #[tokio::test]
    async fn test_write_and_read() {
        #[derive(Serialize, Deserialize)]
        struct Data {
            x: i32,
        }

        let storage_resolver = Arc::new(StorageResolver::new());

        let state = State::new(&Uri::for_test("ram:///workflows"), &storage_resolver).unwrap();
        let data = Data { x: 42 };
        state.save(&data, "test").await.unwrap();
        let result: Data = state.get("test").await.unwrap();
        assert_eq!(result.x, 42);
    }
}
