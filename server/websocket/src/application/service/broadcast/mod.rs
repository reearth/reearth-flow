use crate::domain::entity::broadcast::{BroadcastConfig, BroadcastGroup};
use crate::domain::entity::sub::Subscription;
use crate::domain::repository::kv;
use crate::domain::repository::redis;
use crate::domain::repository::AwarenessRepository;
use crate::domain::repository::BroadcastRepository;
use crate::domain::repository::WebSocketRepository;
use crate::domain::value_objects::document_name::DocumentName;
use crate::domain::value_objects::instance_id::InstanceId;
use anyhow::Result;
use bytes::Bytes;
use std::sync::Arc;
use tokio::sync::oneshot;
use tokio::sync::{Mutex, RwLock};
use tracing::warn;
use yrs::sync::{Awareness, Message, SyncMessage};
use yrs::updates::encoder::Encode;
use yrs::{ReadTxn, Transact};

/// Application service for managing broadcast groups with Y.js support
pub struct BroadcastGroupService<S, R, B, A, W>
where
    S: kv::KVStore + Send + Sync + 'static,
    R: redis::RedisRepository + Send + Sync + 'static,
    B: BroadcastRepository + Send + Sync + 'static,
    A: AwarenessRepository + Send + Sync + 'static,
    W: WebSocketRepository + Send + Sync + 'static,
{
    broadcast_repo: Arc<B>,
    storage_repo: Arc<S>,
    redis_repo: Arc<R>,
    awareness_repo: Arc<A>,
    websocket_repo: Arc<W>,
    config: BroadcastConfig,
}

impl<S, R, B, A, W> BroadcastGroupService<S, R, B, A, W>
where
    S: kv::KVStore + Send + Sync + 'static,
    R: redis::RedisRepository + Send + Sync + 'static,
    B: BroadcastRepository + Send + Sync + 'static,
    A: AwarenessRepository + Send + Sync + 'static,
    W: WebSocketRepository + Send + Sync + 'static,
{
    pub fn new(
        broadcast_repo: Arc<B>,
        storage_repo: Arc<S>,
        redis_repo: Arc<R>,
        awareness_repo: Arc<A>,
        websocket_repo: Arc<W>,
    ) -> Self {
        Self::with_config(
            broadcast_repo,
            storage_repo,
            redis_repo,
            awareness_repo,
            websocket_repo,
            BroadcastConfig::default(),
        )
    }

    pub fn with_config(
        broadcast_repo: Arc<B>,
        storage_repo: Arc<S>,
        redis_repo: Arc<R>,
        awareness_repo: Arc<A>,
        websocket_repo: Arc<W>,
        config: BroadcastConfig,
    ) -> Self {
        Self {
            broadcast_repo,
            storage_repo,
            redis_repo,
            awareness_repo,
            websocket_repo,
            config,
        }
    }

    /// Create or get a broadcast group for a document
    pub async fn get_or_create_group(
        &self,
        document_name: DocumentName,
    ) -> Result<Arc<BroadcastGroup>> {
        // Try to get existing group first
        if let Some(group) = self.broadcast_repo.get_group(&document_name).await? {
            return Ok(group);
        }

        let instance_id = InstanceId::new();

        // Create new group if it doesn't exist
        self.broadcast_repo
            .create_group(document_name, instance_id)
            .await
    }

    /// Handle connection increment for a group
    pub async fn increment_connections(&self, group: &BroadcastGroup) -> Result<usize> {
        let count = group.increment_connections();
        self.redis_repo.register_doc_instance(
            &group.document_name().as_str(),
            &group.instance_id().as_str(),
            60,
        );
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

    /// Load Y.js awareness for a document
    pub async fn load_awareness(
        &self,
        document_name: &DocumentName,
    ) -> Result<Arc<RwLock<Awareness>>> {
        self.awareness_repo.load_awareness(document_name).await
    }

    /// Save awareness state
    pub async fn save_awareness_state(
        &self,
        document_name: &DocumentName,
        awareness: &Awareness,
    ) -> Result<()> {
        self.awareness_repo
            .save_awareness_state(document_name, awareness, self.redis_repo.as_ref())
            .await
    }

    /// Get awareness update for broadcasting
    pub async fn get_awareness_update(
        &self,
        document_name: &DocumentName,
    ) -> Result<Option<Bytes>> {
        self.awareness_repo
            .get_awareness_update(document_name)
            .await
    }

    /// Create WebSocket subscription for Y.js protocol
    pub async fn create_websocket_subscription(
        &self,
        document_name: &DocumentName,
        sink: Arc<Mutex<W::Sink>>,
        stream: W::Stream,
        user_token: Option<String>,
    ) -> Result<Subscription> {
        self.websocket_repo
            .create_subscription(document_name, sink, stream, user_token)
            .await
    }

    /// Handle Y.js protocol message
    pub async fn handle_protocol_message(
        &self,
        document_name: &DocumentName,
        message: yrs::sync::Message,
    ) -> Result<Option<yrs::sync::Message>> {
        self.websocket_repo
            .handle_protocol_message(document_name, message)
            .await
    }

    /// Create or get a broadcast group with Y.js support
    pub async fn get_or_create_group_with_awareness(
        &self,
        document_name: DocumentName,
    ) -> Result<(Arc<BroadcastGroup>, Arc<RwLock<Awareness>>)> {
        // Get or create the broadcast group
        let group = self.get_or_create_group(document_name.clone()).await?;

        // Load awareness for the document
        let awareness = self.load_awareness(&document_name).await?;

        Ok((group, awareness))
    }

    /// Start background tasks for a broadcast group
    pub async fn start_background_tasks(
        &self,
        group: Arc<BroadcastGroup>,
        awareness: Arc<RwLock<Awareness>>,
    ) -> Result<()> {
        let document_name = group.document_name().clone();
        let instance_id = group.instance_id().clone();
        let config = group.config().clone();

        // Start awareness updater task
        let (_, awareness_shutdown_rx) = oneshot::channel();
        let _ = self.spawn_awareness_updater(
            document_name.clone(),
            config.awareness_update_interval_ms,
            awareness_shutdown_rx,
        );

        // Start Redis subscriber task
        let (_, redis_shutdown_rx) = oneshot::channel();
        let _ = self.spawn_redis_subscriber(
            document_name.clone(),
            instance_id.clone(),
            group.last_read_id().clone(),
            redis_shutdown_rx,
        );

        // Start heartbeat task
        let (_, heartbeat_shutdown_rx) = oneshot::channel();
        let _ = self.spawn_heartbeat_task(
            document_name.clone(),
            config.heartbeat_interval_ms,
            heartbeat_shutdown_rx,
        );

        // Start sync task
        let (_, sync_shutdown_rx) = oneshot::channel();
        let _ = self.spawn_sync_task(
            document_name,
            awareness,
            config.sync_interval_ms,
            sync_shutdown_rx,
        );

        // Store task handles in the group (this would require making group mutable)
        // For now, we'll return the handles and let the caller manage them
        // In a real implementation, you might want to store these in a separate task manager

        Ok(())
    }

    async fn stop_background_tasks(&self, group: Arc<BroadcastGroup>) -> Result<()> {
        if let Ok(mut group) = Arc::try_unwrap(group) {
            group.shutdown().await?;
        }
        Ok(())
    }

    /// Spawn awareness updater background task
    fn spawn_awareness_updater(
        &self,
        document_name: DocumentName,
        interval_ms: u64,
        mut shutdown_rx: oneshot::Receiver<()>,
    ) -> tokio::task::JoinHandle<()> {
        let awareness_repo = self.awareness_repo.clone();
        let broadcast_repo = self.broadcast_repo.clone();

        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(tokio::time::Duration::from_millis(interval_ms));

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        // Get awareness update and broadcast it
                        if let Ok(Some(update)) = awareness_repo.get_awareness_update(&document_name).await {
                            let _ = broadcast_repo.broadcast_message(&document_name, update).await;
                        }
                    }
                    _ = &mut shutdown_rx => {
                        break;
                    }
                }
            }
        })
    }

    /// Spawn Redis subscriber background task
    fn spawn_redis_subscriber(
        &self,
        document_name: DocumentName,
        instance_id: InstanceId,
        last_read_id: Arc<tokio::sync::Mutex<String>>,
        mut shutdown_rx: tokio::sync::oneshot::Receiver<()>,
    ) -> tokio::task::JoinHandle<()> {
        let redis_repo = self.redis_repo.clone();
        let broadcast_repo = self.broadcast_repo.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        // Read updates from Redis stream and broadcast them
                        if let Ok(updates) = redis_repo.read_and_filter(
                            document_name.as_str(),
                            100,
                            instance_id.as_str(),
                            &last_read_id,
                        ).await {
                            for update in updates {
                                let _ = broadcast_repo.broadcast_message(&document_name, update).await;
                            }
                        }
                    }
                    _ = &mut shutdown_rx => {
                        break;
                    }
                }
            }
        })
    }

    /// Spawn heartbeat background task
    fn spawn_heartbeat_task(
        &self,
        document_name: DocumentName,
        interval_ms: u64,
        mut shutdown_rx: tokio::sync::oneshot::Receiver<()>,
    ) -> tokio::task::JoinHandle<()> {
        let broadcast_repo = self.broadcast_repo.clone();

        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(tokio::time::Duration::from_millis(interval_ms));

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        // Send heartbeat message
                        let heartbeat = Bytes::from("heartbeat");
                        let _ = broadcast_repo.broadcast_message(&document_name, heartbeat).await;
                    }
                    _ = &mut shutdown_rx => {
                        break;
                    }
                }
            }
        })
    }

    /// Spawn sync background task
    fn spawn_sync_task(
        &self,
        document_name: DocumentName,
        awareness: Arc<RwLock<Awareness>>,
        interval_ms: u64,
        mut shutdown_rx: tokio::sync::oneshot::Receiver<()>,
    ) -> tokio::task::JoinHandle<()> {
        let broadcast_repo = self.broadcast_repo.clone();
        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(tokio::time::Duration::from_millis(interval_ms));

            loop {
                tokio::select! {
                    _ = &mut shutdown_rx => {
                        break;
                    },
                    _ = interval.tick() => {
                        let awareness = awareness.read().await;
                        let txn = awareness.doc().transact();
                        let state_vector = txn.state_vector();

                        let sync_msg = Message::Sync(SyncMessage::SyncStep1(state_vector));
                        let encoded_msg = sync_msg.encode_v1();

                        let msg = Bytes::from(encoded_msg);
                        if let Err(e) = broadcast_repo.broadcast_message(&document_name, msg).await {
                            warn!("Failed to send periodic sync message: {}", e);
                            break;
                        }
                    }
                }
            }
        })
    }
}
