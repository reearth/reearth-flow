use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use tokio::sync::mpsc;

#[async_trait]
pub trait BroadcastRepository: Send + Sync {
    async fn publish(&self, channel: &str, message: Bytes) -> Result<()>;

    async fn subscribe(&self, channel: &str) -> Result<mpsc::Receiver<Bytes>>;

    async fn unsubscribe(&self, channel: &str) -> Result<()>;

    async fn subscriber_count(&self, channel: &str) -> Result<usize>;
}
