use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;

/// 存储仓储接口（用于GCS和Redis）
#[async_trait]
pub trait StorageRepository: Send + Sync {
    /// 获取数据
    async fn get(&self, key: &str) -> Result<Option<Bytes>>;

    /// 设置数据
    async fn set(&self, key: &str, value: Bytes) -> Result<()>;

    /// 删除数据
    async fn delete(&self, key: &str) -> Result<()>;

    /// 设置带过期时间的数据
    async fn set_with_ttl(&self, key: &str, value: Bytes, ttl_secs: u64) -> Result<()>;

    /// 批量获取
    async fn get_batch(&self, keys: &[String]) -> Result<Vec<Option<Bytes>>>;
}
