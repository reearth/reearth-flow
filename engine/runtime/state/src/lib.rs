use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    io::{Error, Result},
    path::{Path, PathBuf},
    sync::Arc,
};

use reearth_flow_common::str::remove_trailing_slash;
use reearth_flow_common::uri::Uri;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_storage::storage::Storage;

const CHUNK_SIZE: usize = 1000;

const ZSTD_LEVEL: i32 = 1;

#[derive(Debug, Clone)]
pub struct State {
    storage: Arc<Storage>,
    root: PathBuf,
    use_compression: bool,
}

impl State {
    pub fn new(root: &Uri, storage_resolver: &StorageResolver) -> Result<Self> {
        Self::new_internal(root, storage_resolver, false)
    }

    pub fn new_with_compression(root: &Uri, storage_resolver: &StorageResolver) -> Result<Self> {
        Self::new_internal(root, storage_resolver, true)
    }

    fn new_internal(
        root: &Uri,
        storage_resolver: &StorageResolver,
        use_compression: bool,
    ) -> Result<Self> {
        let storage = storage_resolver
            .resolve(root)
            .map_err(std::io::Error::other)?;
        Ok(Self {
            storage,
            root: Path::new(
                remove_trailing_slash(root.path().to_str().unwrap_or_default()).as_str(),
            )
            .to_path_buf(),
            use_compression,
        })
    }

    pub async fn save<T>(&self, obj: &T, id: &str) -> Result<()>
    where
        for<'de> T: Serialize + Deserialize<'de>,
    {
        let s = self.object_to_string(obj)?;
        let content = self.encode(s.as_bytes())?;
        let p = self.id_to_location(id, self.json_ext());
        self.storage
            .put(p.as_path(), content)
            .await
            .map_err(Error::other)
    }

    pub fn save_sync<T>(&self, obj: &T, id: &str) -> Result<()>
    where
        for<'de> T: Serialize + Deserialize<'de>,
    {
        let s = self.object_to_string(obj)?;
        let content = self.encode(s.as_bytes())?;
        let p = self.id_to_location(id, self.json_ext());
        self.storage
            .put_sync(p.as_path(), content)
            .map_err(Error::other)
    }

    pub async fn append<T>(&self, obj: &T, id: &str) -> Result<()>
    where
        for<'de> T: Serialize + Deserialize<'de>,
    {
        let s = self.object_to_string(obj)? + "\n";
        let content = self.encode(s.as_bytes())?;
        let p = self.id_to_location(id, self.jsonl_ext());
        self.storage
            .append(p.as_path(), content)
            .await
            .map_err(Error::other)
    }

    pub async fn append_strings(&self, all: &[String], id: &str) -> Result<()> {
        if all.is_empty() {
            return Ok(());
        }
        let p = self.id_to_location(id, self.jsonl_ext());
        for chunk in all.chunks(CHUNK_SIZE) {
            let s = chunk.join("\n") + "\n";
            let content = self.encode(s.as_bytes())?;
            self.storage
                .append(p.as_path(), content)
                .await
                .map_err(Error::other)?
        }
        Ok(())
    }

    pub fn append_sync<T>(&self, obj: &T, id: &str) -> Result<()>
    where
        for<'de> T: Serialize + Deserialize<'de>,
    {
        let s = self.object_to_string(obj)? + "\n";
        let content = self.encode(s.as_bytes())?;
        let p = self.id_to_location(id, self.jsonl_ext());
        self.storage
            .append_sync(p.as_path(), content)
            .map_err(Error::other)
    }

    pub async fn get<T>(&self, id: &str) -> Result<T>
    where
        for<'de> T: Deserialize<'de>,
    {
        let result = self
            .storage
            .get(self.id_to_location(id, self.json_ext()).as_path())
            .await?;
        let bytes = result.bytes().await?;
        let data = self.decode(bytes.as_ref())?;
        let s = std::str::from_utf8(&data).map_err(Error::other)?;
        self.string_to_object::<T>(s)
    }

    pub async fn delete(&self, id: &str) -> Result<()> {
        self.storage
            .delete(self.id_to_location(id, self.json_ext()).as_path())
            .await
            .map_err(Error::other)
    }

    pub fn id_to_location(&self, id: &str, ext: &str) -> PathBuf {
        PathBuf::new()
            .join(self.root.clone())
            .join(format!("{id}.{ext}"))
    }

    pub fn string_to_object<T>(&self, s: &str) -> Result<T>
    where
        for<'de> T: Deserialize<'de>,
    {
        serde_json::from_str(s).map_err(Error::other)
    }

    pub fn object_to_string<T: Serialize>(&self, obj: &T) -> Result<String> {
        serde_json::to_string(obj).map_err(Error::other)
    }

    fn json_ext(&self) -> &'static str {
        if self.use_compression {
            "json.zst"
        } else {
            "json"
        }
    }

    fn jsonl_ext(&self) -> &'static str {
        if self.use_compression {
            "jsonl.zst"
        } else {
            "jsonl"
        }
    }

    fn encode(&self, bytes: &[u8]) -> Result<bytes::Bytes> {
        if self.use_compression {
            let compressed = zstd::stream::encode_all(bytes, ZSTD_LEVEL).map_err(Error::other)?;
            Ok(bytes::Bytes::from(compressed))
        } else {
            Ok(bytes::Bytes::copy_from_slice(bytes))
        }
    }

    fn decode<'a>(&self, bytes: &'a [u8]) -> Result<Cow<'a, [u8]>> {
        if self.use_compression {
            let v = zstd::stream::decode_all(bytes).map_err(Error::other)?;
            Ok(Cow::Owned(v))
        } else {
            Ok(Cow::Borrowed(bytes))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Data {
        x: i32,
    }

    #[tokio::test]
    async fn test_write_and_read() {
        let storage_resolver = Arc::new(StorageResolver::new());

        let state = State::new(&Uri::for_test("ram:///workflows"), &storage_resolver).unwrap();
        let data = Data { x: 42 };
        state.save(&data, "test").await.unwrap();
        let result: Data = state.get("test").await.unwrap();
        assert_eq!(result, data);
    }

    #[tokio::test]
    async fn test_write_and_read_zstd() {
        let storage_resolver = Arc::new(StorageResolver::new());

        let state =
            State::new_with_compression(&Uri::for_test("ram:///workflows"), &storage_resolver)
                .unwrap();
        let data = Data { x: 42 };
        state.save(&data, "test").await.unwrap();
        let result: Data = state.get("test").await.unwrap();
        assert_eq!(result, data);
    }
}
