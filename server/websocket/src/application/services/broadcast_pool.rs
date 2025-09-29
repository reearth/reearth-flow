use crate::application::kv::DocOps;
use crate::infrastructure::gcs::GcsStore;
use crate::infrastructure::redis::RedisStore;
use crate::infrastructure::websocket::{BroadcastGroup, CollaborativeStorage};
use crate::AwarenessRef;
use anyhow::Result;
use dashmap::DashMap;
use std::sync::Arc;
use tracing::{info, warn};
use yrs::sync::Awareness;
use yrs::updates::decoder::Decode;
use yrs::{Doc, Transact, Update};

use crate::domain::value_objects::broadcast::BroadcastConfig;

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

    pub async fn get_group(&self, doc_id: &str) -> Result<Arc<BroadcastGroup>> {
        if let Some(group) = self.groups.get(doc_id) {
            info!("Reusing existing BroadcastGroup for doc_id: {}", doc_id);
            return Ok(group.clone());
        }

        info!("Creating new BroadcastGroup for doc_id: {}", doc_id);
        let group = self.manager.create_group(doc_id).await?;

        self.groups.insert(doc_id.to_string(), group.clone());
        Ok(group)
    }

    pub fn get_existing_group(&self, doc_id: &str) -> Option<Arc<BroadcastGroup>> {
        self.groups.get(doc_id).map(|group| group.clone())
    }

    pub async fn cleanup_group(&self, doc_id: &str) {
        if let Some((_, group)) = self.groups.remove(doc_id) {
            let _ = group.shutdown().await;
            info!("Shutdown BroadcastGroup for doc_id: {}", doc_id);
        }
    }

    pub fn get_cached_groups_count(&self) -> usize {
        self.groups.len()
    }

    pub async fn flush_to_gcs(&self, doc_id: &str) -> Result<()> {
        self.storage.flush_to_gcs(doc_id).await
    }

    pub async fn save_snapshot(&self, doc_id: &str) -> Result<()> {
        self.storage.save_snapshot(doc_id).await
    }
}
