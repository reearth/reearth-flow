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

/// Repository interface for document storage operations
#[async_trait]
pub trait DocumentStorageRepository: Send + Sync {
    /// Save document snapshot to storage
    async fn save_snapshot(&self, document_name: &DocumentName, data: &[u8]) -> Result<()>;

    /// Load document from storage
    async fn load_document(&self, document_name: &DocumentName) -> Result<Option<Vec<u8>>>;

    /// Flush updates to storage
    async fn flush_updates(
        &self,
        document_name: &DocumentName,
        updates: Vec<Vec<u8>>,
    ) -> Result<()>;
}

/// Repository interface for Redis stream operations
#[async_trait]
pub trait RedisStreamRepository: Send + Sync {
    /// Add update to Redis stream
    async fn add_update(&self, document_name: &DocumentName, update: &[u8]) -> Result<String>;

    /// Read updates from Redis stream
    async fn read_updates(
        &self,
        document_name: &DocumentName,
        last_id: &str,
    ) -> Result<Vec<(String, Vec<u8>)>>;

    /// Get last read ID for a document
    async fn get_last_read_id(&self, document_name: &DocumentName) -> Result<Option<String>>;

    /// Set last read ID for a document
    async fn set_last_read_id(&self, document_name: &DocumentName, id: &str) -> Result<()>;
}
