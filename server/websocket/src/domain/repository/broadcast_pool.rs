use std::sync::Arc;

use anyhow::Result as AnyResult;
use async_trait::async_trait;
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::Mutex;

use crate::domain::services::websocket::Subscription;

#[async_trait]
pub trait BroadcastGroupHandle: Send + Sync {
    async fn increment_connections_count(&self);
    async fn decrement_connections_count(&self);
    async fn get_connections_count(&self) -> usize;
    async fn cleanup_client_awareness(&self) -> AnyResult<()>;

    async fn subscribe<Sink, Stream, E>(
        self: Arc<Self>,
        sink: Arc<Mutex<Sink>>,
        stream: Stream,
    ) -> Subscription
    where
        Sink: SinkExt<Bytes, Error = E> + Send + Sync + Unpin + 'static,
        Stream: StreamExt<Item = std::result::Result<Bytes, E>> + Send + Sync + Unpin + 'static,
        E: std::error::Error + Send + Sync + 'static;
}

#[async_trait]
pub trait BroadcastGroupProvider: Send + Sync {
    type Group: BroadcastGroupHandle + ?Sized;

    async fn get_group(&self, doc_id: &str) -> AnyResult<Arc<Self::Group>>;
    async fn cleanup_group(&self, doc_id: &str);
}
