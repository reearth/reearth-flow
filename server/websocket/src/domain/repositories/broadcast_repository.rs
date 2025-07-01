use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use tokio::sync::mpsc;

/// 广播仓储接口（用于消息发布订阅）
#[async_trait]
pub trait BroadcastRepository: Send + Sync {
    /// 发布消息
    async fn publish(&self, channel: &str, message: Bytes) -> Result<()>;

    /// 订阅频道
    async fn subscribe(&self, channel: &str) -> Result<mpsc::Receiver<Bytes>>;

    /// 取消订阅
    async fn unsubscribe(&self, channel: &str) -> Result<()>;

    /// 获取频道订阅者数量
    async fn subscriber_count(&self, channel: &str) -> Result<usize>;
}
