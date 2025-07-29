use crate::application::service::BroadcastGroupService;
use crate::domain::entity::BroadcastGroup;
use crate::domain::value_object::{DocumentName, InstanceId};
use crate::infrastructure::repositories::{
    BroadcastRepositoryImpl, DocumentStorageRepositoryImpl, RedisStreamRepositoryImpl,
};
use crate::storage::gcs::GcsStore;
use crate::storage::redis::RedisStore;
use crate::{AwarenessRef, Subscription};

use anyhow::Result;
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tokio::task::JoinHandle;
use tracing::{debug, error, warn};
use yrs::encoding::write::Write;
use yrs::sync::protocol::{MSG_SYNC, MSG_SYNC_UPDATE};
use yrs::updates::encoder::{Encode, Encoder, EncoderV1};
use yrs::{Doc, ReadTxn, Transact, Update};

use super::types::BroadcastConfig;

/// DDD-compliant BroadcastGroup that coordinates between domain, application, and infrastructure layers
pub struct BroadcastGroupDDD {
    // Domain entity
    domain_group: Arc<BroadcastGroup>,
    
    // Application service
    app_service: Arc<BroadcastGroupService>,
    
    // Infrastructure concerns (kept for compatibility with existing code)
    awareness_ref: AwarenessRef,
    sender: broadcast::Sender<Bytes>,
    doc_sub: yrs::Subscription,
    awareness_sub: yrs::Subscription,
    
    // Background tasks
    awareness_updater: Option<JoinHandle<()>>,
    awareness_shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
    redis_subscriber_task: Option<JoinHandle<()>>,
    redis_subscriber_shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
    heartbeat_task: Option<JoinHandle<()>>,
    heartbeat_shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
    sync_task: Option<JoinHandle<()>>,
    sync_shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl std::fmt::Debug for BroadcastGroupDDD {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BroadcastGroupDDD")
            .field("domain_group", &self.domain_group)
            .field("awareness_ref", &self.awareness_ref)
            .finish()
    }
}

impl BroadcastGroupDDD {
    pub async fn new(
        awareness: AwarenessRef,
        buffer_capacity: usize,
        redis_store: Arc<RedisStore>,
        storage: Arc<GcsStore>,
        config: BroadcastConfig,
    ) -> Result<Self> {
        // Extract document name from config or generate default
        let doc_name_str = config.doc_name.unwrap_or_else(|| "default".to_string());
        let document_name = DocumentName::new(doc_name_str)
            .map_err(|e| anyhow::anyhow!("Invalid document name: {}", e))?;
        let instance_id = InstanceId::new();

        // Create infrastructure repositories
        let broadcast_repo = Arc::new(BroadcastRepositoryImpl::new(buffer_capacity));
        let storage_repo = Arc::new(DocumentStorageRepositoryImpl::new(storage));
        let redis_repo = Arc::new(RedisStreamRepositoryImpl::new(redis_store));

        // Create application service
        let app_service = Arc::new(BroadcastGroupService::new(
            broadcast_repo.clone(),
            storage_repo,
            redis_repo,
        ));

        // Create or get domain group
        let domain_group = app_service
            .get_or_create_group(document_name.clone(), instance_id)
            .await?;

        // Create broadcast channel for Y.js integration
        let (sender, _) = broadcast::channel(buffer_capacity.max(512));
        let awareness_c = Arc::downgrade(&awareness);
        let mut lock = awareness.write().await;
        let sink = sender.clone();

        // Set up Y.js document subscription
        let doc_sub = {
            lock.doc_mut().observe_update_v1(move |_txn, u| {
                let mut encoder = EncoderV1::new();
                encoder.write_var(MSG_SYNC);
                encoder.write_var(MSG_SYNC_UPDATE);
                encoder.write_buf(&u.update);
                let msg = Bytes::from(encoder.to_vec());
                if let Err(e) = sink.send(msg) {
                    error!("broadcast channel closed: {}", e);
                }
            })?
        };

        // Set up awareness subscription
        let sink_awareness = sender.clone();
        let awareness_sub = {
            lock.observe_on_update(move |awareness, e| {
                if let Ok(update) = awareness.update_with_clients(e.all_changes()) {
                    let msg = Bytes::from(update);
                    if let Err(e) = sink_awareness.send(msg) {
                        error!("awareness broadcast channel closed: {}", e);
                    }
                }
            })
        };

        drop(lock);

        Ok(Self {
            domain_group,
            app_service,
            awareness_ref: awareness,
            sender,
            doc_sub,
            awareness_sub,
            awareness_updater: None,
            awareness_shutdown_tx: None,
            redis_subscriber_task: None,
            redis_subscriber_shutdown_tx: None,
            heartbeat_task: None,
            heartbeat_shutdown_tx: None,
            sync_task: None,
            sync_shutdown_tx: None,
        })
    }

    /// Increment connections using domain logic
    pub async fn increment_connections(&self) -> Result<()> {
        self.app_service.increment_connections(&self.domain_group).await?;
        Ok(())
    }

    /// Decrement connections using domain logic
    pub async fn decrement_connections(&self) -> usize {
        self.app_service
            .decrement_connections(&self.domain_group)
            .await
            .unwrap_or(0)
    }

    /// Get connection count from domain entity
    pub fn connection_count(&self) -> usize {
        self.domain_group.connection_count()
    }

    /// Get awareness reference (for compatibility)
    pub fn awareness(&self) -> &AwarenessRef {
        &self.awareness_ref
    }

    /// Get document name from domain entity
    pub fn get_doc_name(&self) -> String {
        self.domain_group.document_name().as_str().to_string()
    }

    /// Subscribe to the broadcast group
    pub async fn subscribe(
        self: Arc<Self>,
        sink: Arc<Mutex<impl SinkExt<Bytes, Error = anyhow::Error> + Send + Unpin + 'static>>,
        mut stream: impl StreamExt<Item = Result<Bytes, anyhow::Error>> + Send + Unpin + 'static,
        user_token: Option<String>,
    ) -> Subscription {
        let mut receiver = self.sender.subscribe();
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel();

        let task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Handle incoming messages from the stream
                    msg = stream.next() => {
                        match msg {
                            Some(Ok(bytes)) => {
                                debug!("Received message from stream: {} bytes", bytes.len());
                                // Process incoming message (Y.js sync, awareness updates, etc.)
                            }
                            Some(Err(e)) => {
                                error!("Stream error: {}", e);
                                break;
                            }
                            None => {
                                debug!("Stream ended");
                                break;
                            }
                        }
                    }
                    
                    // Handle broadcast messages
                    msg = receiver.recv() => {
                        match msg {
                            Ok(bytes) => {
                                let mut sink_guard = sink.lock().await;
                                if let Err(e) = sink_guard.send(bytes).await {
                                    error!("Failed to send to sink: {}", e);
                                    break;
                                }
                            }
                            Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                                debug!("Broadcast channel closed");
                                break;
                            }
                            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                                warn!("Broadcast receiver lagged");
                                continue;
                            }
                        }
                    }
                    
                    // Handle shutdown signal
                    _ = &mut shutdown_rx => {
                        debug!("Subscription shutdown requested");
                        break;
                    }
                }
            }
        });

        Subscription {
            task: Some(task),
            shutdown_tx: Some(shutdown_tx),
        }
    }

    /// Shutdown the broadcast group
    pub async fn shutdown(&self) -> Result<()> {
        // Shutdown background tasks
        if let Some(tx) = &self.awareness_shutdown_tx {
            let _ = tx.send(());
        }
        if let Some(tx) = &self.redis_subscriber_shutdown_tx {
            let _ = tx.send(());
        }
        if let Some(tx) = &self.heartbeat_shutdown_tx {
            let _ = tx.send(());
        }
        if let Some(tx) = &self.sync_shutdown_tx {
            let _ = tx.send(());
        }

        // Wait for tasks to complete
        // Note: In a real implementation, you'd want to store these tasks and await them
        
        Ok(())
    }
}

impl Drop for BroadcastGroupDDD {
    fn drop(&mut self) {
        // Cleanup background tasks
        if let Some(task) = self.awareness_updater.take() {
            task.abort();
        }
        if let Some(task) = self.redis_subscriber_task.take() {
            task.abort();
        }
        if let Some(task) = self.heartbeat_task.take() {
            task.abort();
        }
        if let Some(task) = self.sync_task.take() {
            task.abort();
        }
    }
}
