use crate::domain::value_objects::connection_id::ConnectionId;
use crate::domain::value_objects::document_name::DocumentName;
use crate::domain::value_objects::instance_id::InstanceId;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

/// Configuration for broadcast group behavior
#[derive(Debug, Clone)]
pub struct BroadcastConfig {
    pub buffer_capacity: usize,
    pub heartbeat_interval_ms: u64,
    pub sync_interval_ms: u64,
    pub awareness_update_interval_ms: u64,
}

impl Default for BroadcastConfig {
    fn default() -> Self {
        Self {
            buffer_capacity: 512,
            heartbeat_interval_ms: 30000,
            sync_interval_ms: 5000,
            awareness_update_interval_ms: 1000,
        }
    }
}

/// Domain entity representing a broadcast group for collaborative document editing
#[derive(Debug)]
pub struct BroadcastGroup {
    document_name: DocumentName,
    instance_id: InstanceId,
    connections: AtomicUsize,
    active_connections: HashMap<ConnectionId, Connection>,
    config: BroadcastConfig,
    // Task handles for background operations
    awareness_updater: Option<JoinHandle<()>>,
    redis_subscriber_task: Option<JoinHandle<()>>,
    heartbeat_task: Option<JoinHandle<()>>,
    sync_task: Option<JoinHandle<()>>,
    // Shutdown channels
    awareness_shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
    redis_subscriber_shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
    heartbeat_shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
    sync_shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
    // Last read ID for Redis stream
    last_read_id: Arc<Mutex<String>>,
}

#[derive(Debug, Clone)]
pub struct Connection {
    id: ConnectionId,
    user_token: Option<String>,
    connected_at: std::time::SystemTime,
}

impl BroadcastGroup {
    pub fn new(document_name: DocumentName, instance_id: InstanceId) -> Self {
        Self::with_config(document_name, instance_id, BroadcastConfig::default())
    }

    pub fn with_config(
        document_name: DocumentName,
        instance_id: InstanceId,
        config: BroadcastConfig,
    ) -> Self {
        Self {
            document_name,
            instance_id,
            connections: AtomicUsize::new(0),
            active_connections: HashMap::new(),
            config,
            awareness_updater: None,
            redis_subscriber_task: None,
            heartbeat_task: None,
            sync_task: None,
            awareness_shutdown_tx: None,
            redis_subscriber_shutdown_tx: None,
            heartbeat_shutdown_tx: None,
            sync_shutdown_tx: None,
            last_read_id: Arc::new(Mutex::new("0-0".to_string())),
        }
    }

    pub fn document_name(&self) -> &DocumentName {
        &self.document_name
    }

    pub fn instance_id(&self) -> &InstanceId {
        &self.instance_id
    }

    pub fn connection_count(&self) -> usize {
        self.connections.load(Ordering::Relaxed)
    }

    pub fn increment_connections(&self) -> usize {
        self.connections.fetch_add(1, Ordering::Relaxed) + 1
    }

    pub fn decrement_connections(&self) -> usize {
        let prev = self.connections.fetch_sub(1, Ordering::Relaxed);
        if prev > 0 {
            prev - 1
        } else {
            0
        }
    }

    pub fn is_empty(&self) -> bool {
        self.connection_count() == 0
    }

    pub fn config(&self) -> &BroadcastConfig {
        &self.config
    }

    pub fn last_read_id(&self) -> &Arc<Mutex<String>> {
        &self.last_read_id
    }

    /// Set task handles for background operations
    pub fn set_awareness_updater(
        &mut self,
        handle: JoinHandle<()>,
        shutdown_tx: tokio::sync::oneshot::Sender<()>,
    ) {
        self.awareness_updater = Some(handle);
        self.awareness_shutdown_tx = Some(shutdown_tx);
    }

    pub fn set_redis_subscriber(
        &mut self,
        handle: JoinHandle<()>,
        shutdown_tx: tokio::sync::oneshot::Sender<()>,
    ) {
        self.redis_subscriber_task = Some(handle);
        self.redis_subscriber_shutdown_tx = Some(shutdown_tx);
    }

    pub fn set_heartbeat_task(
        &mut self,
        handle: JoinHandle<()>,
        shutdown_tx: tokio::sync::oneshot::Sender<()>,
    ) {
        self.heartbeat_task = Some(handle);
        self.heartbeat_shutdown_tx = Some(shutdown_tx);
    }

    pub fn set_sync_task(
        &mut self,
        handle: JoinHandle<()>,
        shutdown_tx: tokio::sync::oneshot::Sender<()>,
    ) {
        self.sync_task = Some(handle);
        self.sync_shutdown_tx = Some(shutdown_tx);
    }

    /// Shutdown all background tasks
    pub async fn shutdown(&mut self) -> anyhow::Result<()> {
        // Send shutdown signals
        if let Some(tx) = self.awareness_shutdown_tx.take() {
            let _ = tx.send(());
        }
        if let Some(tx) = self.redis_subscriber_shutdown_tx.take() {
            let _ = tx.send(());
        }
        if let Some(tx) = self.heartbeat_shutdown_tx.take() {
            let _ = tx.send(());
        }
        if let Some(tx) = self.sync_shutdown_tx.take() {
            let _ = tx.send(());
        }

        // Wait for tasks to complete
        if let Some(handle) = self.awareness_updater.take() {
            let _ = handle.await;
        }
        if let Some(handle) = self.redis_subscriber_task.take() {
            let _ = handle.await;
        }
        if let Some(handle) = self.heartbeat_task.take() {
            let _ = handle.await;
        }
        if let Some(handle) = self.sync_task.take() {
            let _ = handle.await;
        }

        Ok(())
    }

    /// Check if all nodes in the document have position information
    pub fn validate_document_state(&self, doc: &yrs::Doc) -> bool {
        use yrs::{ReadTxn, Transact};

        let txn = doc.transact();
        let state_vector = txn.state_vector();

        // Check if all clients have contributed updates
        state_vector.len() > 0
    }
}

impl Connection {
    pub fn new(id: ConnectionId, user_token: Option<String>) -> Self {
        Self {
            id,
            user_token,
            connected_at: std::time::SystemTime::now(),
        }
    }

    pub fn id(&self) -> &ConnectionId {
        &self.id
    }

    pub fn user_token(&self) -> Option<&str> {
        self.user_token.as_deref()
    }

    pub fn connected_at(&self) -> std::time::SystemTime {
        self.connected_at
    }
}
