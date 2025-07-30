use crate::domain::entity::document_name::DocumentName;
use crate::domain::entity::instance_id::InstanceId;
use crate::domain::entity::BroadcastGroup;
use crate::domain::repository::kv;
use crate::domain::repository::redis;
use crate::domain::repository::BroadcastRepository;
use anyhow::Result;
use bytes::Bytes;
use std::sync::Arc;

/// Application service for managing broadcast groups
pub struct BroadcastGroupService<S, R, B>
where
    S: kv::KVStore + Send + Sync + 'static,
    R: redis::RedisRepository + Send + Sync + 'static,
    B: BroadcastRepository + Send + Sync + 'static,
{
    broadcast_repo: Arc<B>,
    storage_repo: Arc<S>,
    redis_repo: Arc<R>,
}

impl<S, R, B> BroadcastGroupService<S, R, B>
where
    S: kv::KVStore + Send + Sync + 'static,
    R: redis::RedisRepository + Send + Sync + 'static,
    B: BroadcastRepository + Send + Sync + 'static,
{
    pub fn new(broadcast_repo: Arc<B>, storage_repo: Arc<S>, redis_repo: Arc<R>) -> Self {
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
        self.broadcast_repo
            .create_group(document_name, instance_id)
            .await
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
            self.broadcast_repo
                .remove_group(&group.document_name())
                .await?;
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
        self.broadcast_repo
            .broadcast_message(document_name, message)
            .await
    }

    /// Save document snapshot
    pub async fn save_snapshot(
        &self,
        document_name: DocumentName,
        data: &[u8],
    ) -> Result<(), S::Error> {
        self.storage_repo
            .upsert(document_name.into_bytes().as_ref(), data)
            .await
    }

    /// Load document from storage
    pub async fn load_document(
        &self,
        document_name: DocumentName,
    ) -> Result<Option<S::Return>, S::Error> {
        self.storage_repo
            .get(document_name.into_bytes().as_ref())
            .await
    }

    /// Add update to Redis stream
    pub async fn add_update_to_stream(
        &self,
        document_name: &DocumentName,
        instance_id: &InstanceId,
        update: &[u8],
    ) -> Result<(), R::Error> {
        self.redis_repo
            .publish_update(document_name.as_str(), update, instance_id.as_str())
            .await
    }

    /// Read updates from Redis stream
    pub async fn read_updates_from_stream(
        &self,
        document_name: &DocumentName,
        instance_id: &InstanceId,
        count: usize,
        last_read_id: &Arc<tokio::sync::Mutex<String>>,
    ) -> Result<Vec<Bytes>, R::Error> {
        let updates = self
            .redis_repo
            .read_and_filter(
                document_name.as_str(),
                count,
                instance_id.as_str(),
                last_read_id,
            )
            .await?;

        Ok(updates)
    }
}
