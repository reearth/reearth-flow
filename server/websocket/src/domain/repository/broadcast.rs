use crate::domain::entity::document_name::DocumentName;
use crate::domain::entity::instance_id::InstanceId;
use crate::domain::entity::BroadcastGroup;
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
