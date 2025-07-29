use crate::domain::entity::BroadcastGroup;
use crate::domain::repository::{BroadcastRepository, DocumentStorageRepository, RedisStreamRepository};
use crate::domain::value_object::{DocumentName, InstanceId};
use anyhow::Result;
use bytes::Bytes;
use std::sync::Arc;

/// Application service for managing broadcast groups
pub struct BroadcastGroupService {
    broadcast_repo: Arc<dyn BroadcastRepository>,
    storage_repo: Arc<dyn DocumentStorageRepository>,
    redis_repo: Arc<dyn RedisStreamRepository>,
}

impl BroadcastGroupService {
    pub fn new(
        broadcast_repo: Arc<dyn BroadcastRepository>,
        storage_repo: Arc<dyn DocumentStorageRepository>,
        redis_repo: Arc<dyn RedisStreamRepository>,
    ) -> Self {
        Self {
            broadcast_repo,
            storage_repo,
            redis_repo,
        }
    }

    /// Create or get a broadcast group for a document
    pub async fn get_or_create_group(
        &self,
        document_name: DocumentName,
        instance_id: InstanceId,
    ) -> Result<Arc<BroadcastGroup>> {
        // Try to get existing group first
        if let Some(group) = self.broadcast_repo.get_group(&document_name).await? {
            return Ok(group);
        }

        // Create new group if it doesn't exist
        self.broadcast_repo.create_group(document_name, instance_id).await
    }

    /// Handle connection increment for a group
    pub async fn increment_connections(&self, group: &BroadcastGroup) -> Result<usize> {
        let count = group.increment_connections();
        Ok(count)
    }

    /// Handle connection decrement for a group
    pub async fn decrement_connections(&self, group: &BroadcastGroup) -> Result<usize> {
        let count = group.decrement_connections();
        
        // If no more connections, consider cleanup
        if count == 0 {
            // Could trigger cleanup logic here
            tracing::debug!("Group {} has no more connections", group.document_name());
        }
        
        Ok(count)
    }

    /// Subscribe to a broadcast group
    pub async fn subscribe_to_group(
        &self,
        document_name: &DocumentName,
    ) -> Result<tokio::sync::broadcast::Receiver<Bytes>> {
        // Get broadcast receiver
        self.broadcast_repo.subscribe(document_name).await
    }

    /// Broadcast a message to all subscribers of a document
    pub async fn broadcast_message(
        &self,
        document_name: &DocumentName,
        message: Bytes,
    ) -> Result<()> {
        self.broadcast_repo.broadcast_message(document_name, message).await
    }

    /// Save document snapshot
    pub async fn save_snapshot(
        &self,
        document_name: &DocumentName,
        data: &[u8],
    ) -> Result<()> {
        self.storage_repo.save_snapshot(document_name, data).await
    }

    /// Load document from storage
    pub async fn load_document(
        &self,
        document_name: &DocumentName,
    ) -> Result<Option<Vec<u8>>> {
        self.storage_repo.load_document(document_name).await
    }

    /// Add update to Redis stream
    pub async fn add_update_to_stream(
        &self,
        document_name: &DocumentName,
        update: &[u8],
    ) -> Result<String> {
        self.redis_repo.add_update(document_name, update).await
    }

    /// Read updates from Redis stream
    pub async fn read_updates_from_stream(
        &self,
        document_name: &DocumentName,
        last_id: &str,
    ) -> Result<Vec<(String, Vec<u8>)>> {
        self.redis_repo.read_updates(document_name, last_id).await
    }

}
