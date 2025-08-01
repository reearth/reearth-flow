use crate::domain::entity::BroadcastGroup;
use crate::domain::value_objects::document_name::DocumentName;
use crate::domain::value_objects::instance_id::InstanceId;
use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use std::sync::Arc;

/// Repository interface for broadcast group operations
#[async_trait]
pub trait BroadcastRepository: Send + Sync {
    /// Create a new broadcast group
    async fn create_group(
        &self,
        document_name: DocumentName,
        instance_id: InstanceId,
    ) -> Result<Arc<BroadcastGroup>>;

    /// Get an existing broadcast group
    async fn get_group(&self, document_name: &DocumentName) -> Result<Option<Arc<BroadcastGroup>>>;

    /// Remove a broadcast group
    async fn remove_group(&self, document_name: &DocumentName) -> Result<()>;

    /// Broadcast message to all subscribers in a group
    async fn broadcast_message(&self, document_name: &DocumentName, message: Bytes) -> Result<()>;

    /// Subscribe to messages for a document
    async fn subscribe(
        &self,
        document_name: &DocumentName,
    ) -> Result<tokio::sync::broadcast::Receiver<Bytes>>;
}

/// Repository interface for WebSocket subscription management
#[async_trait]
pub trait WebSocketRepository: Send + Sync {
    type Sink: futures_util::Sink<yrs::sync::Message> + Send + Sync + Unpin;
    type Stream: futures_util::Stream<Item = Result<yrs::sync::Message, yrs::sync::Error>>
        + Send
        + Sync
        + Unpin;

    /// Create WebSocket subscription for Y.js protocol
    async fn create_subscription(
        &self,
        document_name: &DocumentName,
        sink: Arc<tokio::sync::Mutex<Self::Sink>>,
        stream: Self::Stream,
        user_token: Option<String>,
    ) -> Result<crate::Subscription>;

    /// Handle Y.js protocol message
    async fn handle_protocol_message(
        &self,
        document_name: &DocumentName,
        message: yrs::sync::Message,
    ) -> Result<Option<yrs::sync::Message>>;
}
