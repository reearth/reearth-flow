use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use dashmap::DashMap;
use tokio::sync::Mutex;
use tracing::{info, warn};
use yrs::sync::{Awareness, DefaultProtocol};
use yrs::updates::decoder::Decode;
use yrs::{Doc, Transact, Update};

use crate::application::usecases::kv::DocOps;
use crate::domain::repositories::broadcast_pool::{BroadcastGroupHandle, BroadcastGroupProvider};
use crate::domain::value_objects::broadcast::BroadcastConfig;
use crate::infrastructure::gcs::GcsStore;
use crate::infrastructure::redis::RedisStore;
use crate::infrastructure::websocket::{BroadcastGroup, CollaborativeStorage};
use crate::AwarenessRef;

#[derive(Debug, Clone)]
pub struct BroadcastGroupManager {
    store: Arc<GcsStore>,
    redis_store: Arc<RedisStore>,
    buffer_capacity: usize,
}

impl BroadcastGroupManager {
    pub fn new(store: Arc<GcsStore>, redis_store: Arc<RedisStore>) -> Self {
        Self {
            store,
            redis_store,
            buffer_capacity: 512,
        }
    }

    async fn create_group(&self, doc_id: &str) -> Result<Arc<BroadcastGroup>> {
        let doc = Doc::new();
        let mut txn = doc.transact_mut();
        self.store.load_doc_v2(doc_id, &mut txn).await?;
        drop(txn);

        let awareness: AwarenessRef = Arc::new(tokio::sync::RwLock::new(Awareness::new(doc)));

        let mut final_last_id = "0".to_string();

        let awareness_guard = awareness.write().await;
        let mut txn = awareness_guard.doc().transact_mut();

        match self.redis_store.read_all_stream_data(doc_id).await {
            Ok((updates, last_id)) => {
                for update_data in &updates {
                    if let Ok(update) = Update::decode_v1(update_data) {
                        if let Err(e) = txn.apply_update(update) {
                            warn!("Failed to apply Redis update: {}", e);
                        }
                    }
                }

                if let Some(last_id) = last_id {
                    final_last_id = last_id;
                }
            }
            Err(e) => {
                warn!(
                    "Failed to read updates from Redis stream for document '{}': {}",
                    doc_id, e
                );
            }
        }

        drop(txn);
        drop(awareness_guard);

        let group = Arc::new(
            BroadcastGroup::new(
                awareness,
                self.buffer_capacity,
                Arc::clone(&self.redis_store),
                Arc::clone(&self.store),
                BroadcastConfig {
                    storage_enabled: true,
                    doc_name: Some(doc_id.to_string()),
                },
            )
            .await?,
        );

        if final_last_id != "0" {
            let last_read_id = group.get_last_read_id();
            let mut last_id_guard = last_read_id.lock().await;
            *last_id_guard = final_last_id;
        }

        Ok(group)
    }
}

#[derive(Clone, Debug)]
pub struct BroadcastPool {
    manager: BroadcastGroupManager,
    groups: Arc<DashMap<String, Arc<BroadcastGroup>>>,
    storage: Arc<CollaborativeStorage>,
}

impl BroadcastPool {
    pub fn new(store: Arc<GcsStore>, redis_store: Arc<RedisStore>) -> Self {
        let storage = Arc::new(CollaborativeStorage::new(
            Arc::clone(&store),
            Arc::clone(&redis_store),
        ));
        let manager = BroadcastGroupManager::new(store, redis_store);
        Self {
            manager,
            groups: Arc::new(DashMap::new()),
            storage,
        }
    }

    pub fn get_store(&self) -> Arc<GcsStore> {
        self.storage.store()
    }

    pub fn get_redis_store(&self) -> Arc<RedisStore> {
        self.storage.redis_store()
    }

    async fn ensure_group(&self, doc_id: &str) -> Result<Arc<BroadcastGroup>> {
        if let Some(group) = self.groups.get(doc_id) {
            info!("Reusing existing BroadcastGroup for doc_id: {}", doc_id);
            return Ok(group.clone());
        }

        info!("Creating new BroadcastGroup for doc_id: {}", doc_id);
        let group = self.manager.create_group(doc_id).await?;

        self.groups.insert(doc_id.to_string(), group.clone());
        Ok(group)
    }

    async fn perform_cleanup(&self, doc_id: &str) {
        if let Some((_, group)) = self.groups.remove(doc_id) {
            let _ = group.shutdown().await;
            info!("Shutdown BroadcastGroup for doc_id: {}", doc_id);
        }
    }

    pub async fn get_group(&self, doc_id: &str) -> Result<Arc<BroadcastGroup>> {
        self.ensure_group(doc_id).await
    }

    pub async fn cleanup_group(&self, doc_id: &str) {
        self.perform_cleanup(doc_id).await;
    }

    pub fn get_cached_groups_count(&self) -> usize {
        self.groups.len()
    }
}

#[async_trait]
impl BroadcastGroupProvider for BroadcastPool {
    type Group = BroadcastGroup;

    async fn get_group(&self, doc_id: &str) -> Result<Arc<Self::Group>> {
        self.ensure_group(doc_id).await
    }

    async fn cleanup_group(&self, doc_id: &str) {
        self.perform_cleanup(doc_id).await;
    }
}

#[async_trait]
impl BroadcastGroupHandle for BroadcastGroup {
    async fn increment_connections_count(&self) {
        self.connections_count.increment();
    }

    async fn decrement_connections_count(&self) {
        self.connections_count.decrement();
    }

    async fn get_connections_count(&self) -> usize {
        self.connections_count.get()
    }

    async fn get_client_id(&self) -> String {
        self.awareness_ref.read().await.client_id().to_string()
    }

    fn get_doc_name(&self) -> &str {
        &self.doc_name
    }

    async fn cleanup_client_awareness(&self) -> Result<()> {
        let awareness = self.awareness().clone();
        let awareness_read = awareness.read().await;
        awareness_read.clean_local_state();
        Ok(())
    }

    async fn subscribe<Sink, Stream, E>(
        self: Arc<Self>,
        sink: Arc<Mutex<Sink>>,
        stream: Stream,
    ) -> crate::domain::value_objects::websocket::Subscription
    where
        Sink: futures_util::Sink<Bytes, Error = E> + Send + Sync + Unpin + 'static,
        Stream: futures_util::Stream<Item = std::result::Result<Bytes, E>>
            + Send
            + Sync
            + Unpin
            + 'static,
        E: std::error::Error + Send + Sync + 'static,
    {
        self.listen(sink, stream, DefaultProtocol).await
    }
}
