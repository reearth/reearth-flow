use crate::domain::entity::broadcast::{BroadcastGroup, BroadcastConfig};
use crate::domain::value_objects::document_name::DocumentName;
use crate::domain::value_objects::instance_id::InstanceId;
use crate::infrastructure::repositories::{
    AwarenessRepositoryImpl, BroadcastRepositoryImpl, WebSocketRepositoryImpl,
};
use crate::application::service::broadcast::BroadcastGroupService;
use crate::interface::ws::broadcast_handler::{BroadcastWebSocketHandler, BroadcastWebSocketHandlerFactory};
use anyhow::Result;
use std::sync::Arc;

/// DDD-compliant BroadcastGroup facade that coordinates all layers
/// 
/// This provides a clean interface that maintains compatibility with existing code
/// while leveraging the new DDD architecture underneath.
pub struct BroadcastGroupDDD {
    service: Arc<BroadcastGroupService<
        Arc<crate::storage::gcs::GcsStore>,
        Arc<crate::storage::redis::RedisStore>,
        Arc<BroadcastRepositoryImpl>,
        Arc<AwarenessRepositoryImpl>,
        Arc<WebSocketRepositoryImpl>,
    >>,
    handler_factory: BroadcastWebSocketHandlerFactory,
    document_name: DocumentName,
    instance_id: InstanceId,
}

impl BroadcastGroupDDD {
    /// Create a new DDD-compliant broadcast group
    pub async fn new(
        document_name: DocumentName,
        instance_id: InstanceId,
        gcs_store: Arc<crate::storage::gcs::GcsStore>,
        redis_store: Arc<crate::storage::redis::RedisStore>,
        config: BroadcastConfig,
    ) -> Result<Self> {
        // Create repository implementations
        let broadcast_repo = Arc::new(BroadcastRepositoryImpl::new(config.buffer_capacity));
        let awareness_repo = Arc::new(AwarenessRepositoryImpl::new(gcs_store.clone()));
        let websocket_repo = Arc::new(WebSocketRepositoryImpl::new(
            broadcast_repo.clone(),
            awareness_repo.clone(),
        ));

        // Create the application service
        let service = Arc::new(BroadcastGroupService::with_config(
            broadcast_repo,
            gcs_store,
            redis_store,
            awareness_repo,
            websocket_repo,
            config.clone(),
        ));

        // Create handler factory
        let handler_factory = BroadcastWebSocketHandlerFactory::new(config);

        Ok(Self {
            service,
            handler_factory,
            document_name,
            instance_id,
        })
    }

    /// Get the document name
    pub fn document_name(&self) -> &DocumentName {
        &self.document_name
    }

    /// Get the instance ID
    pub fn instance_id(&self) -> &InstanceId {
        &self.instance_id
    }

    /// Get or create a broadcast group using the DDD service
    pub async fn get_or_create_group(&self) -> Result<Arc<BroadcastGroup>> {
        self.service.get_or_create_group(self.document_name.clone()).await
    }

    /// Subscribe to document updates
    pub async fn subscribe_to_updates(&self) -> Result<tokio::sync::broadcast::Receiver<bytes::Bytes>> {
        self.service.subscribe_to_group(&self.document_name).await
    }

    /// Broadcast a message to all subscribers
    pub async fn broadcast_message(&self, message: bytes::Bytes) -> Result<()> {
        self.service.broadcast_message(&self.document_name, message).await
    }

    /// Handle WebSocket connection (simplified interface)
    pub async fn handle_websocket_connection(
        &self,
        websocket_stream: tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        user_token: Option<String>,
    ) -> Result<()> {
        // Use the simplified handler from the interface layer
        let handler = crate::interface::ws::broadcast_handler::SimpleBroadcastHandler::new(
            self.service.broadcast_repo.clone(),
            self.service.awareness_repo.clone(),
        );

        handler.handle_simple_connection(
            self.document_name.clone(),
            websocket_stream,
            user_token,
        ).await
    }

    /// Get connection count for the group
    pub async fn connection_count(&self) -> Result<usize> {
        if let Some(group) = self.service.get_group(&self.document_name).await? {
            Ok(group.connection_count())
        } else {
            Ok(0)
        }
    }

    /// Increment connection count
    pub async fn increment_connections(&self) -> Result<usize> {
        let group = self.get_or_create_group().await?;
        self.service.increment_connections(&group).await
    }

    /// Decrement connection count
    pub async fn decrement_connections(&self) -> Result<usize> {
        if let Some(group) = self.service.get_group(&self.document_name).await? {
            self.service.decrement_connections(&group).await
        } else {
            Ok(0)
        }
    }

    /// Save document snapshot
    pub async fn save_snapshot(&self, data: &[u8]) -> Result<()> {
        self.service.save_snapshot(self.document_name.clone(), data).await
            .map_err(|e| anyhow::anyhow!("Failed to save snapshot: {:?}", e))
    }

    /// Load document from storage
    pub async fn load_document(&self) -> Result<Option<Vec<u8>>> {
        self.service.load_document(self.document_name.clone()).await
            .map_err(|e| anyhow::anyhow!("Failed to load document: {:?}", e))
    }

    /// Start all background tasks for the group
    pub async fn start_background_tasks(&self) -> Result<()> {
        let (group, awareness) = self.service
            .get_or_create_group_with_awareness(self.document_name.clone())
            .await?;

        self.service.start_background_tasks(group, awareness).await
    }

    /// Shutdown the broadcast group and clean up resources
    pub async fn shutdown(&self) -> Result<()> {
        // Remove the group from the repository
        self.service.remove_group(&self.document_name).await?;
        Ok(())
    }
}

/// Example usage of the DDD-compliant broadcast group
#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::gcs::GcsStore;
    use crate::storage::redis::RedisStore;

    #[tokio::test]
    async fn test_ddd_broadcast_group() -> Result<()> {
        // Create storage dependencies
        let gcs_store = Arc::new(GcsStore::new("test-bucket".to_string()));
        let redis_store = Arc::new(RedisStore::new("redis://localhost:6379").await?);

        // Create document name and instance ID
        let document_name = DocumentName::new("test-doc".to_string())?;
        let instance_id = InstanceId::new();

        // Create DDD broadcast group
        let broadcast_group = BroadcastGroupDDD::new(
            document_name,
            instance_id,
            gcs_store,
            redis_store,
            BroadcastConfig::default(),
        ).await?;

        // Test basic operations
        let group = broadcast_group.get_or_create_group().await?;
        assert_eq!(group.connection_count(), 0);

        let count = broadcast_group.increment_connections().await?;
        assert_eq!(count, 1);

        let count = broadcast_group.decrement_connections().await?;
        assert_eq!(count, 0);

        // Test message broadcasting
        let message = bytes::Bytes::from("test message");
        broadcast_group.broadcast_message(message).await?;

        // Cleanup
        broadcast_group.shutdown().await?;

        Ok(())
    }
}

/// Factory for creating DDD-compliant broadcast groups
pub struct BroadcastGroupDDDFactory {
    gcs_store: Arc<crate::storage::gcs::GcsStore>,
    redis_store: Arc<crate::storage::redis::RedisStore>,
    config: BroadcastConfig,
}

impl BroadcastGroupDDDFactory {
    pub fn new(
        gcs_store: Arc<crate::storage::gcs::GcsStore>,
        redis_store: Arc<crate::storage::redis::RedisStore>,
        config: BroadcastConfig,
    ) -> Self {
        Self {
            gcs_store,
            redis_store,
            config,
        }
    }

    pub async fn create_group(
        &self,
        document_name: DocumentName,
        instance_id: InstanceId,
    ) -> Result<BroadcastGroupDDD> {
        BroadcastGroupDDD::new(
            document_name,
            instance_id,
            self.gcs_store.clone(),
            self.redis_store.clone(),
            self.config.clone(),
        ).await
    }
}
