use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;

#[async_trait]
pub trait StorageRepository: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<Bytes>>;

    async fn set(&self, key: &str, value: Bytes) -> Result<()>;

    async fn delete(&self, key: &str) -> Result<()>;

    async fn set_with_ttl(&self, key: &str, value: Bytes, ttl_secs: u64) -> Result<()>;

    async fn get_batch(&self, keys: &[String]) -> Result<Vec<Option<Bytes>>>;
}
